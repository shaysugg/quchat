use qu_chat_models::{Message, SendMessageParams};
use rocket::{
    fairing::AdHoc,
    response::stream::{Event, EventStream},
    serde::json::Json,
    tokio::sync::broadcast::{channel, error::RecvError, Sender},
    Shutdown, State,
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::{authentication::UserId, base::*};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct MessageDM {
    id: String,
    content: String,
    sender_id: String,
    room_id: String,
    create_date: i64,
}

#[derive(Debug, Serialize, Clone)]
#[allow(dead_code)]
struct RoomListChange {
    state_id: u128,
    room_id: String,
}

#[post("/send", data = "<params>")]
async fn send(
    params: Json<SendMessageParams>,
    changes: &State<Sender<RoomChange>>,
    user_id: UserId,
    mut db: Connection<Db>,
) -> ApiResult<String> {
    let room_not_exists = sqlx::query!("SELECT * FROM rooms WHERE (id)=($1)", params.room_id)
        .fetch_one(&mut **db)
        .await;

    let message = MessageDM {
        id: uuid::Uuid::new_v4().to_string(),
        content: params.text.to_string(),
        sender_id: user_id.id,
        room_id: params.room_id.to_string(),
        create_date: chrono::Utc::now().timestamp(),
    };

    let insert_res = sqlx::query!(
        "Insert INTO messages (id, content, room_id, sender_id, create_date) VALUES ($1, $2, $3, $4, $5)",
        message.id,
        message.content,
        message.room_id,
        message.sender_id,
        message.create_date
    )
    .execute(&mut **db)
    .await;

    let sender_id = message.sender_id;
    let name_result = sqlx::query!("SELECT name FROM users WHERE id = ($1)", sender_id)
        .fetch_one(&mut **db)
        .await
        .map(|r| r.name);

    let name = match (insert_res, name_result, room_not_exists) {
        (Ok(_), Ok(name), Ok(_)) => name,
        (_, _, Err(_)) => return ApiResultBuilder::err("Room doesn't exists."),
        (_, Err(_), _) => return ApiResultBuilder::err("Can't find sender name"),
        (Err(_), _, _) => return ApiResultBuilder::err("Can't send message."),
    };

    let change = RoomChange {
        message: Message {
            id: message.id,
            content: message.content,
            sender_id: sender_id,
            room_id: message.room_id,
            create_date: message.create_date,
            sender_name: name,
        },
    };

    let id = change.message.id.clone();
    match changes.send(change) {
        Ok(_) => ApiResultBuilder::data(id),
        Err(_) => Err(Error::Internal(())),
    }
}

#[get("/events/<room_id>")]
async fn events(
    changes: &State<Sender<RoomChange>>,
    room_id: String,
    mut shutdown: Shutdown,
) -> EventStream![] {
    let mut rx = changes.subscribe();
    EventStream! {
        loop {
            let change = rocket::tokio::select! {
                change = rx.recv() => match change {
                    Ok(change) if change.message.room_id == room_id => { change},
                    Ok(_) => continue,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut shutdown => break,
            };

            yield Event::json(&change.message);
        }
    }
}

// async fn unread(
//     changes: &State<Sender<RoomChange>>,

// )

#[get("/<room_id>?<size>")]
async fn messages(
    room_id: String,
    size: Option<u32>,
    _user_id: UserId,
    mut db: Connection<Db>,
) -> ApiResult<Vec<Message>> {
    let size = size.unwrap_or(20);
    let rows = sqlx::query(
        r#"
        SELECT messages.id, messages.content, messages.room_id, messages.sender_id, messages.create_date, users.name
        FROM messages
        INNER JOIN users ON messages.sender_id = users.id
        WHERE messages.room_id = ($1) ORDER BY create_date LIMIT ($2);
        "#
    )
    .bind(&room_id)
    .bind(size)
    .fetch_all(&mut **db)
    .await;

    let result = match rows {
        Ok(ref a) => Ok(a
            .iter()
            .map(|b| Message {
                id: b.get(0),
                content: b.get(1),
                sender_id: b.get(3),
                room_id: b.get(2),
                create_date: b.get(4),
                sender_name: b.get(5),
            })
            .collect::<Vec<Message>>()),
        Err(e) => Err(e),
    };

    ApiResultBuilder::from(result, "Unable to fetch messages")
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Messages Stage", |rocket| async {
        let (tx, rx) = channel::<RoomChange>(1024);
        rocket
            .mount("/messages", routes![events, send, messages])
            .manage(tx)
            .manage(rx)
    })
}
