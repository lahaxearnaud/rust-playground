mod http_client;
mod http;

#[macro_use]
extern crate diesel;

extern crate dotenv;

mod db;

use actix_web::{
    get, post, web, App, HttpResponse, HttpServer, Responder, Result,
    middleware::Logger
};
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    user_id: u32,
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

            .service(http::controllers::quotes::list)
            .service(http::controllers::quotes::item)
            .service(http::controllers::quotes::delete)

            .service(echo)
            .service(user)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
