use chrono::Utc;
use rocket::{
    fairing::AdHoc,
    http::Status,
    response::stream::{Event, EventStream},
    serde::json::Json,
    tokio::sync::broadcast::{channel, error::RecvError, Sender},
    Shutdown, State,
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use crate::{authentication::UserId, base::*, serde_datetime::date_format};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Message {
    id: String,
    content: String,
    sender_id: String,
    room_id: String,
    create_date: i64,
}
#[derive(Debug, Clone, Serialize)]
struct RoomChange {
    message: Message,
}

impl Identifiable for Message {
    fn id(&self) -> &str {
        &self.id
    }
}
#[derive(Debug, Clone, Deserialize)]
struct SendParams<'r> {
    text: &'r str,
    room_id: String,
}

#[post("/send", data = "<params>")]
async fn send<'r>(
    params: Json<SendParams<'r>>,
    changes: &State<Sender<RoomChange>>,
    user_id: UserId,
    mut db: Connection<Db>,
) -> Status {
    let room_not_exists = sqlx::query!("SELECT * FROM rooms WHERE (id)=($1)", params.room_id)
        .fetch_one(&mut **db)
        .await
        .is_err();

    if room_not_exists {
        return Status::BadRequest;
    }

    let message = Message {
        id: uuid::Uuid::new_v4().to_string(),
        content: params.text.to_string(),
        sender_id: user_id.id,
        room_id: params.room_id.clone(),
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

    if insert_res.is_err() {
        return Status::BadRequest;
    }

    let change = RoomChange { message };

    match changes.send(change) {
        Ok(_) => Status::Ok,
        Err(_) => Status::BadRequest,
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

#[get("/<room_id>?<size>")]
async fn messages(
    room_id: String,
    size: Option<u32>,
    user_id: UserId,
    mut db: Connection<Db>,
) -> ApiResult<Vec<Message>> {
    let size = size.unwrap_or(20);
    let result = sqlx::query_as!(
        Message,
        "SELECT * FROM messages WHERE (room_id)=($1) ORDER BY create_date DESC LIMIT ($2)",
        room_id,
        size,
    )
    .fetch_all(&mut **db)
    .await;

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
