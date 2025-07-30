use std::str::FromStr;

use chrono::Utc;
use rocket::fairing::AdHoc;
use rocket::futures::{TryFutureExt, TryStreamExt};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};

use rocket_db_pools::Connection;

use crate::base::{ApiResult, ApiResultBuilder};
use crate::{
    authentication::UserId,
    base::{Db, Error, Identifiable, Rx, Tx},
};

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

#[derive(Deserialize)]
struct CreateRoomParam {
    name: String,
}

#[get("/status")]
fn rooms_status(rx: &State<Rx<RoomsStatus>>) -> Json<RoomsStatus> {
    let s = rx.0.try_recv().ok().unwrap_or(RoomsStatus::new());
    Json(s)
}

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

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Rooms Stage", |rocket| async {
        let (tx, rx) = flume::bounded::<RoomsStatus>(32);
        rocket
            .mount(
                "/",
                routes![rooms_status, insert_online_user, remove_online_user],
            )
            .mount("/rooms", routes![get_all, insert, get_room])
            .manage(Tx(tx))
            .manage(Rx(rx))
    })
}
