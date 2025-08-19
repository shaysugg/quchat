use chrono::Utc;
use rocket::fairing::AdHoc;
use rocket::futures::TryStreamExt;
use rocket::http::Status;
use rocket::response::stream::{Event, EventStream};
use rocket::serde::json::Json;
use rocket::tokio::sync::broadcast::{error::RecvError, Sender};
use rocket::{Shutdown, State};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::HashMap;
use std::str::FromStr;

use crate::authentication::UserId;
use crate::base::{ApiResult, ApiResultBuilder, RoomChange};
use crate::base::{Db, Identifiable, Rx, Tx};

#[derive(Serialize)]
pub struct RoomsStatus {
    pub online_users: Vec<String>,
}

impl RoomsStatus {
    pub fn new() -> Self {
        RoomsStatus {
            online_users: Vec::new(),
        }
    }
}

#[derive(Serialize)]
pub struct Room {
    id: String,
    name: String,
    creator_id: String,
    create_date: i64,
}
impl Room {
    pub fn uuid(&self) -> uuid::Uuid {
        uuid::Uuid::from_str(&self.id).unwrap()
    }
}

impl Identifiable for Room {
    fn id(&self) -> &str {
        self.id.as_str()
    }
}

#[derive(Debug, Serialize)]
pub struct RoomState {
    room_id: String,
    has_unread: bool,
}

#[derive(Deserialize)]
struct CreateRoomParam {
    name: String,
}

#[get("/status")]
fn rooms_status(rx: &State<Rx<RoomsStatus>>) -> Json<RoomsStatus> {
    let s = rx.0.try_recv().ok().unwrap_or(RoomsStatus::new());
    Json(s)
}

#[derive(Debug, Serialize, Clone)]
pub struct RoomsStateChange;

#[post("/online", data = "<id>")]
fn insert_online_user(
    id: String,
    rx: &State<Rx<RoomsStatus>>,
    tx: &State<Tx<RoomsStatus>>,
) -> Status {
    let mut status = rx.0.try_recv().ok().unwrap_or(RoomsStatus::new());
    status.online_users.push(id);
    let _ = tx.0.try_send(status);
    Status::Accepted
}

#[post("/offline", data = "<id>")]
fn remove_online_user(
    id: String,
    rx: &State<Rx<RoomsStatus>>,
    tx: &State<Tx<RoomsStatus>>,
) -> Status {
    let mut status = rx.0.try_recv().ok().unwrap_or(RoomsStatus::new());
    if let Some(i) = status.online_users.iter().position(|a| a == id.as_str()) {
        status.online_users.remove(i);
    }
    let _ = tx.0.try_send(status);
    Status::Accepted
}

#[get("/")]
async fn get_all(mut db: Connection<Db>) -> ApiResult<Vec<Room>> {
    let rooms = sqlx::query_as!(Room, "SELECT * FROM rooms ORDER BY create_date DESC")
        .fetch(&mut **db)
        .try_collect::<Vec<_>>()
        .await;

    ApiResultBuilder::from(rooms, "Failed to fetch rooms")
}

#[get("/<id>")]
async fn get_room(id: &str, mut db: Connection<Db>) -> ApiResult<Room> {
    let result = sqlx::query_as!(Room, "SELECT * FROM rooms WHERE (id)=($1)", id)
        .fetch_one(&mut **db)
        .await;

    ApiResultBuilder::from(result, "Failed to fetch rooms")
}

#[post("/", data = "<param>")]
async fn insert(
    mut db: Connection<Db>,
    user_id: UserId,
    param: Json<CreateRoomParam>,
) -> ApiResult<Room> {
    let room = Room {
        id: uuid::Uuid::new_v4().to_string(),
        name: param.name.clone(),
        creator_id: user_id.id,
        create_date: Utc::now().timestamp(),
    };
    let result = sqlx::query!(
        "INSERT INTO rooms (id, name, creator_id, create_date) VALUES ($1, $2, $3, $4)",
        room.id,
        room.name,
        room.creator_id,
        room.create_date
    )
    .execute(&mut **db)
    .await;
    match result {
        Ok(_) => ApiResultBuilder::data(room),
        Err(_) => ApiResultBuilder::err("Failed to create room"),
    }
}

#[get("/states/events?<room_ids>")]
async fn state_events(
    changes: &State<Sender<RoomChange>>,
    room_ids: String,
    mut shutdown: Shutdown,
) -> EventStream![] {
    let mut rx = changes.subscribe();
    let room_ids = room_ids
        .split(',')
        .map(|s| s.to_owned())
        .collect::<std::collections::HashSet<String>>();

    EventStream! {
        loop {
            let _ = rocket::tokio::select! {
                change = rx.recv() => {
                     match change {

                    Ok(change) if room_ids.contains(&change.message.room_id) => {
                        println!("change2 {:?}",change);
                        change
                    }
                    Ok(_) => continue,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                }},
                _ = &mut shutdown => break,
            };

            yield Event::empty();
        }
    }
}

#[get("/states?<room_ids>")]
async fn rooms_state(
    mut db: Connection<Db>,
    user_id: UserId,
    room_ids: String,
) -> ApiResult<Vec<RoomState>> {
    let room_ids = room_ids.split(',').collect::<Vec<&str>>();
    let placeholders = std::iter::repeat("?")
        .take(room_ids.len())
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "SELECT room_id, last_seen FROM room_state WHERE room_id IN ({}) AND user_id = (?)",
        placeholders
    );

    let mut query_builder = sqlx::query(&query);
    for id in room_ids.iter() {
        query_builder = query_builder.bind(id);
    }

    let last_seen_res = query_builder
        .bind(&user_id.id)
        .fetch_all(&mut **db)
        .await
        .map(|res| {
            res.iter()
                .map(|row| (row.get(0), row.try_get(1).ok()))
                .collect::<HashMap<String, Option<i32>>>()
        });

    println!("last seen {:?}", last_seen_res);
    println!("last seen {:?}", last_seen_res);

    let query = format!(
        "SELECT room_id, MAX(create_date) FROM messages WHERE room_id IN ({}) AND sender_id != (?) GROUP BY room_id",
        placeholders
    );
    let mut query_builder = sqlx::query(&query);
    for id in room_ids.iter() {
        query_builder = query_builder.bind(id);
    }
    let last_message_result = query_builder
        .bind(&user_id.id)
        .fetch_all(&mut **db)
        .await
        .map(|res| {
            res.iter()
                .map(|row| (row.get(0), row.get(1)))
                .collect::<HashMap<String, i32>>()
        });

    println!("last message {:?}", last_message_result);

    let mut states = Vec::new();

    match (last_seen_res, last_message_result) {
        (Ok(last_seen), Ok(last_message)) => {
            for (room_id, message_timestamp) in last_message {
                let has_unread = if let Some(seen_timestamp) = last_seen.get(&room_id) {
                    message_timestamp > seen_timestamp.unwrap_or(0)
                } else {
                    true
                };

                states.push(RoomState {
                    room_id,
                    has_unread: has_unread,
                });
            }
            ApiResultBuilder::data(states)
        }
        (Ok(_), Err(_)) => ApiResultBuilder::err("Unable to get room states"),
        (Err(_), Ok(_)) => ApiResultBuilder::err("Unable to get room states"),
        (Err(_), Err(_)) => ApiResultBuilder::err("Unable to get room states"),
    }
}

#[post("/states/<room_id>")]
pub async fn update_room_state(
    mut db: Connection<Db>,
    user_id: UserId,
    room_id: String,
) -> ApiResult<String> {
    #[derive(Deserialize)]
    struct RoomStateDM {
        id: String,
        user_id: String,
        room_id: String,
        last_seen: Option<i64>,
    }

    let result = sqlx::query_as!(
        RoomStateDM,
        "SELECT * FROM room_state WHERE room_id = ($1) AND user_id = ($2)",
        user_id.id,
        room_id,
    )
    .fetch_optional(&mut **db)
    .await;

    //check if result has item
    let state = match result {
        Ok(res) => res,
        Err(_error) => return ApiResultBuilder::err("Unable to set state"),
    };

    let now = chrono::Utc::now().timestamp();
    // if has update
    if let Some(state) = state {
        let result = sqlx::query!(
            "UPDATE room_state SET last_seen = ($1) WHERE id = ($2)",
            now,
            state.id,
        )
        .execute(&mut **db)
        .await;

        ApiResultBuilder::from(
            result.map(|_| ("Successfully set state".to_string())),
            "Unable to set state",
        )
    } else {
        let id = uuid::Uuid::new_v4().to_string();
        let result = sqlx::query!(
            "INSERT INTO room_state (id, user_id, room_id, last_seen) VALUES ($1, $2, $3, $4)",
            id,
            user_id.id,
            room_id,
            now,
        )
        .execute(&mut **db)
        .await;

        ApiResultBuilder::from(
            result.map(|_| ("Successfully set state".to_string())),
            "Unable to set state",
        )
    }

    //if doesn't have then insert
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Rooms Stage", |rocket| async {
        let (tx, rx) = flume::bounded::<RoomsStatus>(32);
        rocket
            .mount(
                "/",
                routes![rooms_status, insert_online_user, remove_online_user,],
            )
            .mount(
                "/rooms",
                routes![
                    rooms_state,
                    get_all,
                    insert,
                    get_room,
                    state_events,
                    update_room_state
                ],
            )
            .manage(Tx(tx))
            .manage(Rx(rx))
    })
}
