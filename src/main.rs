mod http;

#[macro_use]
extern crate diesel;

extern crate dotenv;

mod db;

use dotenv::dotenv;
use std::{env, collections::BTreeMap};

use hmac::{Hmac, Mac};
use jwt::{VerifyWithKey, SignWithKey};
use sha2::Sha256;

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

    let key: Hmac<Sha256> = Hmac::new_from_slice(
        env::var("JWT_SECRET").unwrap_or("JWT_SECRET".to_string()).as_ref()
    ).unwrap();

    let verify_promise: Result<BTreeMap<String, String>, jwt::Error> = token.verify_with_key(&key);

    if verify_promise.is_err() {
        return Err((Error::from(MyError::Unauthorized), req));
    }

    Ok(req)
}


fn create_jwt() -> String {
    let key: Hmac<Sha256> = Hmac::new_from_slice(
        env::var("JWT_SECRET").unwrap_or("JWT_SECRET".to_string()).as_ref()
    ).unwrap();
    let mut claims = BTreeMap::new();
    claims.insert("audiance", "127.0.0.1");

    return claims.sign_with_key(&key).unwrap();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_env().unwrap();

    dotenv().ok();
    let env_jwt_secret = env::var("JWT_SECRET");
    match env_jwt_secret {
        Ok(_) => (),
        Err(_) => panic!("JWT_SECRET env var must be defined"),
    }

    log::info!("JWT: {}", create_jwt());


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
