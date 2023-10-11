
use crate::http_client;
use crate::http;
use crate::db::repositories::quote::QuoteRepository;

use actix_web::HttpResponse;
use actix_web::Responder;
use actix_web::{
    get, Result,
};
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;


#[derive(Serialize, Deserialize, Debug)]
struct Quote {
    id: u32,
    quote: String,
    author: String
}

#[derive(Serialize, Deserialize, Debug)]
struct QuotesResponse {
    quotes: Vec<Quote>,
    total: u32,
    skip: u32,
    limit: u32
}

#[get("/sqlite")]
pub async fn sqlite() -> impl Responder {
    let quote_repository = QuoteRepository;
    HttpResponse::Ok().json(quote_repository.get_quotes())
}

#[get("/")]
pub async fn hello() -> Result<String, http::error::MyError> {
    let client_promise = http_client::client::build_client();

    if client_promise.is_err() {
        log::warn!("Fail to create http client: {}", client_promise.unwrap_err().to_string());
        return Err(http::error::MyError::BadClientData)
    }

    // Perform the actual execution of the network request
    let res = client_promise
        .ok().unwrap()
        .get("https://dummyjson.com/quotes")
        .send()
        .await;

    if res.is_err() {
        let unwrapped_error = res.unwrap_err();
        if unwrapped_error.is_timeout() {
            log::warn!("Timeout on call api: {}", unwrapped_error.to_string());
            return Err(http::error::MyError::Timeout)
        }

        log::warn!("Fail to call api: {}", unwrapped_error.to_string());
        return Err(http::error::MyError::InternalError)
    }

    let quotes = res.ok().unwrap().json:: <QuotesResponse>().await;

    if quotes.is_err() {
        log::warn!("Fail to json decode: {}", quotes.unwrap_err().to_string());
        return Err(http::error::MyError::InternalError)
    }

    Ok(
        quotes
        .ok()
        .unwrap()
        .quotes
        .choose(&mut rand::thread_rng())
        .unwrap()
        .quote
        .to_string()
    )
}
