mod http_client;
mod http;

#[macro_use]
extern crate diesel;

extern crate dotenv;

mod db;

use actix_web::{
    App, HttpServer, Result,
    dev::ServiceRequest,
    Error,
    middleware::{Logger, DefaultHeaders}, http::header::ContentType
};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use http::error::MyError;


async fn validator(
    req: ServiceRequest,
    _credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token = _credentials.token();

    if !token.eq("foo") {
        return Err((Error::from(MyError::Unauthorized), req));
    }

    Ok(req)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_env().unwrap();

    HttpServer::new(|| {
        let auth = HttpAuthentication::bearer(validator);

        App::new()
            // auth
            .wrap(auth)

            // api format
            .wrap(DefaultHeaders::new().add(ContentType::json()))

            // logs
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))

            // routes
            .service(http::controllers::quotes::list)
            .service(http::controllers::quotes::item)
            .service(http::controllers::quotes::delete)
            .service(http::controllers::quotes::add)
            .service(http::controllers::quotes::update)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
