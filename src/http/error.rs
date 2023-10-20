use actix_web::{
    HttpResponse, error,
    http::{header::ContentType, StatusCode},
};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum MyError {
    #[display(fmt = "Bad request")]
    BadClientData,

    #[display(fmt = "Not found")]
    NotFount,

    #[display(fmt = "Server Unavailable")]
    ServerUnavailable,

    #[display(fmt = "Unauthorized")]
    Unauthorized,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::BadClientData => StatusCode::BAD_REQUEST,
            MyError::NotFount => StatusCode::NOT_FOUND,
            MyError::Unauthorized => StatusCode::UNAUTHORIZED,
            MyError::ServerUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}
