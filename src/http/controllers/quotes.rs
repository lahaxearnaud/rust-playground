use crate::http;
use crate::db::repositories::quote::QuoteRepository;
use crate::db::entities::quote::{Quote, ApiPayloadQuote};
use actix_web::http::StatusCode;
use actix_web::http::header::ContentType;
use log::warn;
use validator::Validate;

use actix_web::web::{Path, Json};
use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::{
    get, delete, post, put,
    Result,
};
use uuid::Uuid;


#[get("/quotes")]
pub async fn list() -> impl Responder {
    let quote_repository = QuoteRepository;
    let response_promise = quote_repository.get_quotes(Some(1000));

    if response_promise.is_err() {
        warn!("{:?}", response_promise.unwrap_err());

        return Err(http::error::MyError::NotFount)
    }

    Ok(HttpResponse::Ok()
            .json(response_promise.unwrap()))
}

#[get("/quotes/{quote_id}")]
pub async fn item(path: Path<String>) -> Result<HttpResponse, http::error::MyError> {
    let quote_id = path.into_inner();
    let quote_repository = QuoteRepository;
    let response_promise = quote_repository.get_quote(quote_id);

    if response_promise.is_err() {
        warn!("{:?}", response_promise.unwrap_err());

        return Err(http::error::MyError::NotFount)
    }

    Ok(HttpResponse::Ok()
            .json(response_promise.unwrap()))
}

#[delete("/quotes/{quote_id}")]
pub async fn delete(path: Path<String>) -> impl Responder {
    let quote_id = path.into_inner();
    let quote_repository = QuoteRepository;
    let _ = quote_repository.remove(quote_id);

    HttpResponse::Ok()
}

#[post("/quotes")]
pub async fn add(quote_form: Json<ApiPayloadQuote>) -> Result<HttpResponse, http::error::MyError> {
    let quote_repository = QuoteRepository;

    let validation = quote_form.validate();

    if validation.is_err() {
        return Ok(HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .insert_header(ContentType::json())
            .json(validation.err()));
    }

    let new_quote = Quote {
        id: Uuid::new_v4().to_string(),
        author: quote_form.author.to_string(),
        quote: quote_form.quote.to_string()
    };

    let response_promise = quote_repository
        .insert(new_quote.clone());

    if response_promise.is_err() {
        warn!("{:?}", response_promise.unwrap_err());
        return Err(http::error::MyError::BadClientData)
    }

    Ok(HttpResponse::Ok().json(new_quote))
}

#[put("/quotes/{quote_id}")]
pub async fn update(quote_form: Json<ApiPayloadQuote>, path: Path<String>) -> Result<HttpResponse, http::error::MyError> {
    let quote_id = path.into_inner();

    let validation = quote_form.validate();

    if validation.is_err() {
        return Ok(HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .insert_header(ContentType::json())
            .json(validation.err()));
    }

    let quote_repository = QuoteRepository;
    let response_promise = quote_repository.get_quote(quote_id);

    if response_promise.is_err() {
        warn!("{:?}", response_promise.unwrap_err());

        return Err(http::error::MyError::NotFount)
    }

    let mut db_quote = response_promise.unwrap();

    db_quote.quote = quote_form.quote.to_string();
    db_quote.author = quote_form.author.to_string();

    let update_promise = quote_repository.update(db_quote.clone());

    if update_promise.is_err() {
        warn!("{:?}", update_promise.unwrap_err());

        return Err(http::error::MyError::NotFount)
    }

    Ok(HttpResponse::Ok().json(db_quote))
}
