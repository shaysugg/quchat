use crate::base::{ApiResult, Db};
use crate::{authentication::UserId, base::ApiResultBuilder};
use qu_chat_models::{Identifiable, UserProfile};
use rocket::fairing::AdHoc;
use rocket_db_pools::Connection;
use serde::Serialize;

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub secret: String,
}

impl Identifiable for User {
    fn id(&self) -> &str {
        &self.id
    }
}

#[get("/")]
async fn get_users(mut db: Connection<Db>) -> ApiResult<Vec<UserProfile>> {
    let result = sqlx::query!("SELECT id, name FROM users")
        .fetch_all(&mut **db)
        .await
        .map(|rows| {
            rows.into_iter()
                .map(|row| UserProfile {
                    id: row.id,
                    name: row.name,
                })
                .collect::<Vec<UserProfile>>()
        });

    ApiResultBuilder::from(result, "Failed to fetch users")
}

#[get("/<id>")]
async fn get_user(mut db: Connection<Db>, id: &str) -> ApiResult<UserProfile> {
    let result = sqlx::query!("SELECT * FROM users WHERE id = ($1)", id)
        .fetch_one(&mut **db)
        .await
        .map(|row| UserProfile {
            id: row.id,
            name: row.name,
        });

    ApiResultBuilder::from(result, "Failed to fetch user")
}

#[get("/whoami")]
async fn whoami(user_id: UserId, mut db: Connection<Db>) -> ApiResult<UserProfile> {
    let result = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ($1)", user_id.id)
        .fetch_one(&mut **db)
        .await
        .map(|row| UserProfile {
            id: row.id,
            name: row.name,
        });

    ApiResultBuilder::from(result, "Failed to fetch user profile")
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("Users Stage", |rocket| async {
        rocket.mount("/users", routes![get_users, get_user, whoami])
    })
}
