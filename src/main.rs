use actix_web::{
    get, post, web, App, HttpResponse, HttpServer, Responder, Result, error,
    http::{header::ContentType, StatusCode},
    middleware::Logger
};
use serde::{Deserialize, Serialize};
use rand::seq::SliceRandom;

use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
enum MyError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "timeout")]
    Timeout,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::BadClientData => StatusCode::BAD_REQUEST,
            MyError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

#[derive(Deserialize)]
struct Info {
    user_id: u32,
}

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

#[get("/")]
async fn hello() -> Result<String, MyError> {
    let client = reqwest::Client::builder()
    .proxy(reqwest::Proxy::https("http://pnpxyu.boursorama.fr:3128").ok().unwrap())
    .build();

    if client.is_err() {
        log::warn!("Fail to create http client: {}", client.unwrap_err().to_string());
        return Err(MyError::BadClientData)
    }

    // Perform the actual execution of the network request
    let res = client.ok().unwrap()
        .get("https://dummyjson.com/quotes")
        .send()
        .await;

    if res.is_err() {
        let unwrapped_error = res.unwrap_err();
        if unwrapped_error.is_timeout() {
            log::warn!("Timeout on call api: {}", unwrapped_error.to_string());
            return Err(MyError::Timeout)
        }
        
        log::warn!("Fail to call api: {}", unwrapped_error.to_string());
        return Err(MyError::InternalError)
    }

    let quotes = res.ok().unwrap().json:: <QuotesResponse>().await;

    if quotes.is_err() {
        log::warn!("Fail to json decode: {}", quotes.unwrap_err().to_string());
        return Err(MyError::InternalError)
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

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

#[get("/users/{user_id}")]
async fn user(info: web::Path<Info>) -> Result<String> {

    Ok(format!("Welcome user_id {}!", info.user_id))
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_env().unwrap();

    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))
    
            .service(hello)
            .service(echo)
            .service(user)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}