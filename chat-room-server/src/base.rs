use qu_chat_models::Message;
use rocket::response::Responder;
use rocket::serde::json::Json;
use serde::Serialize;

pub struct Tx<T>(pub flume::Sender<T>);
pub struct Rx<T>(pub flume::Receiver<T>);

#[derive(rocket_db_pools::Database)]
#[database("main")]
pub struct Db(pub sqlx::SqlitePool);

pub type ApiResult<T> = Result<Json<BaseRes<T>>, Error<'static>>;
pub struct ApiResultBuilder;
impl ApiResultBuilder {
    pub fn data<T>(data: T) -> ApiResult<T> {
        Ok(Json(BaseRes { data }))
    }

    pub fn err<T>(msg: &'static str) -> ApiResult<T> {
        Err(Error::Logical(Json(SimpleError { msg })))
    }

    pub fn from<T, E>(result: Result<T, E>, msg: &'static str) -> ApiResult<T> {
        match result {
            Ok(res) => ApiResultBuilder::data(res),
            Err(_) => ApiResultBuilder::err(msg),
        }
    }
}

#[derive(Serialize)]
pub struct BaseRes<T> {
    data: T,
}

#[derive(Serialize)]
pub struct SimpleError<'r> {
    pub msg: &'r str,
}

#[derive(Responder)]
pub enum Error<'r> {
    #[response(status = 400)]
    Logical(Json<SimpleError<'r>>),
    #[response(status = 401)]
    Unauthorized(Json<SimpleError<'r>>),
    #[response(status = 500)]
    Internal(()),
}

impl<'r> Error<'r> {
    pub fn logical(msg: &'r str) -> Error<'r> {
        Error::Logical(Json(SimpleError { msg }))
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct RoomChange {
    pub message: Message,
}
