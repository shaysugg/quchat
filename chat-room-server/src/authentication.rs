#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_macros)]
#![allow(unused_results)]
#![allow(unreachable_code)]
use qu_chat_models::{RegisterParams, RegisterResponse, SignInParams, SignInResponse};
use rocket::fairing::AdHoc;
use rocket::http::Status;

use rocket::outcome::try_outcome;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::serde::json::Json;
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::base::{ApiResult, ApiResultBuilder, Db};
use crate::jwt::*;
use crate::user::User;

static SECRET_KEY_MINE: &[u8] = b"hello";

pub struct UserId {
    pub id: String,
    pub token: String,
}

#[derive(Debug)]
pub enum ApiKeyError {
    Missing,
    Invalid,
    Expired,
    Db,
}

#[derive(Debug)]
pub enum UserGuardError {
    ApiKey(ApiKeyError),
    DB,
}

impl From<ApiKeyError> for UserGuardError {
    fn from(value: ApiKeyError) -> Self {
        UserGuardError::ApiKey(value)
    }
}

impl From<sqlx::Error> for UserGuardError {
    fn from(_: sqlx::Error) -> Self {
        UserGuardError::DB
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserId {
    type Error = ApiKeyError;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        //TODO: use my error
        let db = try_outcome!(req
            .guard::<Connection<Db>>()
            .await
            .map_error(|_| (Status::InternalServerError, ApiKeyError::Db)));

        let token = match req.headers().get_one("Authorization") {
            Some(token) => token,
            None => return Outcome::Error((Status::Unauthorized, ApiKeyError::Missing)),
        };

        let token = match token_from_barear(token) {
            Ok(token) => token,
            Err(_) => return Outcome::Error((Status::Unauthorized, ApiKeyError::Invalid)),
        };

        let token = match check_token_expired(token, db).await {
            Ok(token) => token,
            Err(err) => return Outcome::Error((Status::Unauthorized, err)),
        };

        match validate_jwt(&token, SECRET_KEY_MINE) {
            Ok(body) => Outcome::Success(UserId {
                id: body.user_id.to_string(),
                token: token,
            }),
            Err(_) => Outcome::Error((Status::Unauthorized, ApiKeyError::Invalid)),
        }
    }
}

#[post("/register", data = "<param>")]
async fn register<'r>(
    mut db: Connection<Db>,
    param: Json<RegisterParams>,
) -> ApiResult<RegisterResponse> {
    //write on db
    let id = uuid::Uuid::new_v4().to_string();
    let secret = hash(&param.password);
    let insert_result = sqlx::query!(
        "INSERT INTO users (id, name, secret) VALUES ($1, $2, $3)",
        id,
        param.username,
        secret,
    )
    .execute(&mut **db)
    .await;
    //TODO: if user exists

    match insert_result {
        Ok(_) => match generate_jwt(&id, SECRET_KEY_MINE) {
            Ok(token) => ApiResultBuilder::data(RegisterResponse { token: token }),
            Err(_) => ApiResultBuilder::err("Unable to create token"),
        },
        Err(err) => ApiResultBuilder::err("Unable to create user {:?}"),
    }
}

pub async fn check_token_expired(
    token: &str,
    mut db: Connection<Db>,
) -> Result<String, ApiKeyError> {
    #[derive(Serialize, Deserialize, Debug)]
    struct Record {
        id: i64,
        token: String,
    }
    //TODO internal server if db failed
    let res = sqlx::query_as!(
        Record,
        "SELECT * FROM token_blacklist WHERE token=($1)",
        token
    )
    .fetch_optional(&mut **db)
    .await
    .map_err(|e| ApiKeyError::Db)?;

    match res {
        Some(_) => Err(ApiKeyError::Expired),
        None => Ok(token.to_string()),
    }
}

fn hash(str: &str) -> String {
    hex::encode(Sha256::digest(&str).to_ascii_lowercase())
}

#[post("/signin", data = "<params>")]
async fn signin(mut db: Connection<Db>, params: Json<SignInParams>) -> ApiResult<SignInResponse> {
    let res = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE name = ($1)",
        params.username
    )
    .fetch_one(&mut **db)
    .await;

    let user: User;
    match res {
        Ok(usr) => user = usr,
        Err(_) => return ApiResultBuilder::err("User not found"),
    };

    if hash(&params.password) != user.secret {
        return ApiResultBuilder::err("Invalid password");
    };

    let token_res = generate_jwt(&user.id, SECRET_KEY_MINE);

    match token_res {
        Ok(token) => ApiResultBuilder::data(SignInResponse { token }),
        Err(_) => ApiResultBuilder::err("Unable to create token"),
    }
}

#[post("/signout")]
async fn signout(mut db: Connection<Db>, user_id: UserId) -> ApiResult<String> {
    let res = sqlx::query!(
        "INSERT into token_blacklist (token) VALUES ($1)",
        user_id.token
    )
    .execute(&mut **db)
    .await;

    match res {
        Ok(_) => ApiResultBuilder::data("Successfully signed out".to_string()),
        Err(_) => ApiResultBuilder::err("Unable to signed out"),
    }
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Authentication", |rocket| async {
        rocket.mount("/auth", routes![register, signin, signout])
    })
}
