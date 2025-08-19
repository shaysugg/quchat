use rocket::{fairing::AdHoc, serde::json::Json};

use crate::base::{ApiResult, Error, SimpleError};

#[catch(401)]
pub fn unauthorized() -> ApiResult<()> {
    Err(Error::Unauthorized(Json(SimpleError {
        msg: "Unauthorised",
    })))
}

#[catch(500)]
pub fn internal_server() -> ApiResult<()> {
    Err(Error::Internal(()))
}

#[catch(404)]
pub fn notfound() -> ApiResult<()> {
    Err(Error::Logical(Json(SimpleError { msg: "Not Found" })))
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("catcher", |rocket| async {
        rocket.register("/", catchers![notfound, internal_server, unauthorized])
    })
}
