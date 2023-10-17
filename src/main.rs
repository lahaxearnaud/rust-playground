mod http;

#[macro_use]
extern crate diesel;

extern crate dotenv;

mod db;

use db::entities::quote::ApiPayloadQuote;
use dotenv::dotenv;
use std::{env, collections::BTreeMap};

use hmac::{Hmac, Mac};
use jwt::{VerifyWithKey, SignWithKey};
use sha2::Sha256;

use actix_web::{
    test,
    App, HttpServer, Result,
    dev::ServiceRequest,
    Error,
    middleware::{Logger, DefaultHeaders}, http::{header::ContentType, StatusCode}
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


#[actix_web::test]
async fn test_index_without_jwt() {
    dotenv().ok();
    let auth = HttpAuthentication::bearer(validator);
    let app = test::init_service(
        App::new()
            .wrap(auth)
            .service(http::controllers::quotes::list)
    ).await;
    let req = test::TestRequest::get().uri("/quotes")
        .insert_header(ContentType::json())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status() == StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_index_with_jwt() {
    dotenv().ok();
    let auth = HttpAuthentication::bearer(validator);

    let app = test::init_service(
        App::new()
            .wrap(auth)
            .service(http::controllers::quotes::list)
    ).await;
    let req = test::TestRequest::get().uri("/quotes")
        .insert_header(ContentType::json())
        .insert_header(("Authorization", format!("Bearer {}" ,create_jwt())))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let success = resp.status().is_success();

    let body = test::read_body(resp).await;
    println!("Out: {:?}", std::str::from_utf8(&body));

    assert!(success);
}

#[actix_web::test]
async fn test_index_get() {
    dotenv().ok();
    let app = test::init_service(App::new().service(http::controllers::quotes::list)).await;
    let req = test::TestRequest::get().uri("/quotes")
        .insert_header(ContentType::json())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let success = resp.status().is_success();

    let body = test::read_body(resp).await;
    println!("Out: {:?}", std::str::from_utf8(&body));

    assert!(success);
}

#[actix_web::test]
async fn test_index_post() {
    dotenv().ok();
    simple_logger::init_with_env().unwrap();

    let app = test::init_service(
        App::new()
        .wrap(Logger::default())
        .service(http::controllers::quotes::add)
    ).await;
    let req = test::TestRequest::post().uri("/quotes")
        .insert_header(ContentType::json())
        .set_json(ApiPayloadQuote{quote: "Il ne pas respirer la compote".to_string(), author: "Tintin le beau".to_string()})
        .to_request();
    let resp = test::call_service(&app, req).await;
    let success = resp.status().is_success();

    let body = test::read_body(resp).await;
    println!("Out: {:?}", std::str::from_utf8(&body));

    assert!(success);
}
