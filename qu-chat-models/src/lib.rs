use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub trait Identifiable {
    fn id(&self) -> &str;
    fn uuid(&self) -> uuid::Uuid {
        uuid::Uuid::from_str(&self.id()).unwrap()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub sender_id: String,
    pub room_id: String,
    pub create_date: i64,
    pub sender_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub creator_id: String,
    pub create_date: i64,
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

#[derive(Debug, Deserialize, Serialize)]
pub struct RoomState {
    pub room_id: String,
    pub has_unread: bool,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterParams {
    pub username: String,
    pub password: String,
}
#[derive(Deserialize, Serialize)]
pub struct RegisterResponse {
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SignInParams {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize)]
pub struct SignInResponse {
    pub token: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreateRoomParam {
    pub name: String,
}

#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct SendMessageParams {
    pub text: String,
    pub room_id: String,
}
