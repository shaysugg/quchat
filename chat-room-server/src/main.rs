pub mod authentication;
pub mod base;
pub mod catchers;
pub mod jwt;
pub mod message;
pub mod rooms;
pub mod user;

use rocket_db_pools::Database;

#[macro_use]
extern crate rocket;

use crate::base::Db;

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(Db::init())
        .attach(rooms::stage())
        .attach(user::stage())
        .attach(authentication::stage())
        .attach(message::stage())
        .attach(catchers::stage())
}
