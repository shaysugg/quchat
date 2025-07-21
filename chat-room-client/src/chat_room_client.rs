use std::{fmt::Display, str::FromStr};

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct Room {
    pub id: String,
    pub name: String,
    pub create_date: i32,
}
#[derive(Deserialize, Debug)]
pub struct Message {
    pub id: String,
    pub content: String,
    pub sender_id: String,
    pub room_id: String,
    pub create_date: i32,
}

pub struct User {
    pub id: String,
    pub name: String,
    pub token: String,
}

#[derive(Deserialize, Debug)]
pub struct UserProfile {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
struct RegisterBody<'r> {
    username: &'r str,
    password: &'r str,
}

#[derive(Deserialize)]
pub struct RegisterResponse {
    pub token: String,
}

#[derive(Deserialize)]
enum BaseRes<T> {
    #[serde(rename = "data")]
    Data(T),
    //TODO: temp
    #[serde(rename = "msg")]
    Error(String),
}

pub async fn register(client: &Client, username: &str, password: &str) -> Result<RegisterResponse> {
    let body = RegisterBody { username, password };
    let response = client
        .inner
        .post(URLs::register())
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::from(e))
        .inspect_err(|e| handle_unauthorized(e, client.unauthtorized_sender.clone()))?
        .json::<BaseRes<RegisterResponse>>()
        .await?;
    handle_result(response)
}

pub async fn whoami(client: &Client, token: &str) -> Result<UserProfile> {
    let response = client
        .inner
        .get(URLs::whoami())
        .bearer_auth(token)
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| Error::from(e))
        .inspect_err(|e| handle_unauthorized(e, client.unauthtorized_sender.clone()))?
        .json::<BaseRes<UserProfile>>()
        .await?;

    handle_result(response)
}

#[derive(Debug, Serialize)]
struct SignInBody<'r> {
    username: &'r str,
    password: &'r str,
}

#[derive(Deserialize)]
pub struct SignInResponse {
    pub token: String,
}

pub async fn sigin(client: &Client, username: &str, password: &str) -> Result<SignInResponse> {
    let body = SignInBody { username, password };
    let response = client
        .inner
        .post(URLs::signin())
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::from(e))
        .inspect_err(|e| handle_unauthorized(e, client.unauthtorized_sender.clone()))?
        .json::<BaseRes<SignInResponse>>()
        .await?;
    handle_result(response)
    // Ok(response)
}

#[derive(Deserialize)]
pub struct GetRoomsResponse {
    pub rooms: Vec<Room>,
}

pub async fn rooms(client: &Client, token: &str) -> Result<GetRoomsResponse> {
    let response = client
        .inner
        .get(URLs::rooms())
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| Error::from(e))
        .inspect_err(|e| handle_unauthorized(e, client.unauthtorized_sender.clone()))?
        .json::<BaseRes<Vec<Room>>>()
        .await?;

    handle_result(response).map(|r| GetRoomsResponse { rooms: r })
}

#[derive(Debug, Clone, Serialize)]
struct SendMessageParams<'r> {
    text: &'r str,
    room_id: &'r str,
}

pub async fn send_message(
    client: &Client,
    token: &str,
    message: &str,
    room_id: &str,
) -> Result<()> {
    let body = SendMessageParams {
        text: message,
        room_id,
    };
    let response = client
        .inner
        .post(URLs::send_messages())
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .map_err(|e| Error::from(e))
        .inspect_err(|e| handle_unauthorized(e, client.unauthtorized_sender.clone()))?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(Error::Logical(format!(
            "Unable to send message for {}",
            room_id
        )))
    }
}

pub async fn messages<'r>(
    client: &Client,
    token: &str,
    room_id: &'r str,
    sender: tokio::sync::mpsc::Sender<Message>,
) {
    use std::result::Result;
    let res = client
        .inner
        .get(URLs::messages(room_id))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let mut stream = res.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if let Ok(chunk) = chunk {
            match std::str::from_utf8(&chunk) {
                Ok(s) => {
                    if !(s.starts_with("data:")) {
                        continue;
                    }
                    //remove the initial data:
                    let mut string = String::from(s);
                    let string = string.split_off(5);
                    // println!("String {}", string);
                    let message: Result<Message, serde_json::Error> = serde_json::from_str(&string);
                    match message {
                        Ok(msg) => sender.send(msg).await.unwrap(),
                        Err(_) => println!("Unable to decode {}", string),
                    }
                }
                Err(e) => println!("Error: {}", e),
            };
        }
        // sender.send(chunk);
    }
}

pub async fn last_messages(client: &Client, room_id: &str, token: &str) -> Result<Vec<Message>> {
    let result = client
        .inner
        .get(URLs::last_messages(room_id))
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| Error::from(e))
        .inspect_err(|e| handle_unauthorized(e, client.unauthtorized_sender.clone()))?
        .json::<BaseRes<Vec<Message>>>()
        .await?;
    handle_result(result)
}

pub async fn signout(client: &Client, token: &str) -> Result<()> {
    let result = client
        .inner
        .post(URLs::signout())
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| Error::from(e))
        .inspect_err(|e| handle_unauthorized(e, client.unauthtorized_sender.clone()))?
        .json::<BaseRes<String>>()
        .await?;

    handle_result(result).map(|_| ())
}

fn handle_result<T>(res: BaseRes<T>) -> Result<T> {
    match res {
        BaseRes::Data(data) => Ok(data),
        BaseRes::Error(base_error) => Err(Error::Logical(base_error)),
    }
}

fn handle_unauthorized(error: &Error, sender: tokio::sync::mpsc::UnboundedSender<()>) {
    match error {
        Error::Unauthorized => {
            sender.send(()).unwrap();
        }
        _ => (),
    };
}

pub type Result<T> = std::result::Result<T, self::Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Unauthorized,
    TimedOut,
    Decoding,
    Other(String),
    Logical(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Error::Unauthorized => String::from("Unauthorized"),
            Error::TimedOut => String::from("Timed out"),
            Error::Other(str) => String::from(format!("Unknown Error {}", str)),
            Error::Logical(string) => string.to_string(),
            Error::Decoding => String::from("Unable to decode"),
        };
        write!(f, "{}", string)
    }
}

// Implement Error trait
impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::from(&error)
    }
}

impl From<&reqwest::Error> for Error {
    fn from(error: &reqwest::Error) -> Self {
        if error.is_decode() {
            return Self::Decoding;
        }

        if error.status() == Some(reqwest::StatusCode::UNAUTHORIZED) {
            return Self::Unauthorized;
        }

        if error.is_timeout() {
            return Self::TimedOut;
        }

        Self::Other(error.to_string())
    }
}

pub struct Client {
    pub inner: reqwest::Client,
    pub unauthtorized_sender: tokio::sync::mpsc::UnboundedSender<()>,
}

struct URLs;

impl URLs {
    fn base() -> String {
        String::from("http://127.0.0.1:8000")
    }

    fn register() -> String {
        format!("{}/auth/register", URLs::base())
    }

    fn signin() -> String {
        format!("{}/auth/signin", URLs::base())
    }

    fn rooms() -> String {
        format!("{}/rooms", URLs::base())
    }

    fn messages(room_id: &str) -> String {
        format!("{}/messages/events/{}", URLs::base(), room_id)
    }

    fn last_messages(room_id: &str) -> String {
        format!("{}/messages/{}", URLs::base(), room_id)
    }

    fn send_messages() -> String {
        format!("{}/messages/send", URLs::base())
    }

    fn whoami() -> String {
        format!("{}/users/whoami", URLs::base())
    }

    fn signout() -> String {
        format!("{}/auth/signout", URLs::base())
    }
}

#[cfg(test)]
mod tests {
    use crate::chat_room_client::*;

    #[tokio::test]

    async fn test_unauthenticate() {
        let rqwclient_builder = reqwest::Client::builder();
        let rqwclient = rqwclient_builder.build().unwrap();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        let client = Client {
            inner: rqwclient,
            unauthtorized_sender: tx,
        };

        let res = whoami(&client, "blah").await;
        print!("{:?}", res);
        let mut event_recved = false;
        while let Ok(_) = rx.try_recv() {
            event_recved = true;
        }
        assert!(event_recved);
    }
}
