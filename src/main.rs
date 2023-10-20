mod http;
mod db;
mod config;

#[macro_use]
extern crate diesel;

extern crate dotenv;

use crate::config::env::load_config_from_env;
use crate::db::pool::build_db_pool;
use dotenv::dotenv;
use log::info;
use std::{env, collections::BTreeMap};

use hmac::{Hmac, Mac};
use jwt::{VerifyWithKey, SignWithKey};
use sha2::Sha256;

use actix_web::{
    get,
    App, HttpServer, Result, web::{self, Redirect},
    dev::ServiceRequest,
    Error,
    middleware::{Logger, DefaultHeaders}, http::{header::ContentType, StatusCode}, Responder, HttpResponse
};

use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};
use http::error::MyError;

use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use utoipa_swagger_ui::SwaggerUi;
use actix_web_prom::PrometheusMetricsBuilder;
use std::collections::HashMap;
use gethostname::gethostname;

async fn validator(
    req: ServiceRequest,
    _credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    if req.path().contains("/swagger-ui") {
        return Ok(req);
    }

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

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok()
        .status(StatusCode::OK)
        .content_type(ContentType::plaintext())
        .body("Ok")
}

#[get("/health")]
async fn health_json() -> impl Responder {
    HttpResponse::Ok()
        .status(StatusCode::OK)
        .content_type(ContentType::json())
        .json("Ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    simple_logger::init_with_env().unwrap();

    let config = load_config_from_env();

    log::info!("JWT: {}", create_jwt());
    
    let pool = build_db_pool(config.database_url.to_string());

    struct SecurityAddon;

    impl Modify for SecurityAddon {
        fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
            let components = openapi.components.as_mut().unwrap();
            components.add_security_scheme(
                "token",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            )
        }
    }

    #[derive(OpenApi)]
    #[openapi(
        modifiers(&SecurityAddon),
        paths(
            http::controllers::quotes::list,
            http::controllers::quotes::item,
            http::controllers::quotes::add,
            http::controllers::quotes::update,
            http::controllers::quotes::delete,
        ),
        components(
            schemas(
                db::entities::quote::Quote,
                db::entities::quote::ApiPayloadQuote
            )
        )
    )]
    struct ApiDoc;

    let openapi = ApiDoc::openapi();

    info!("Start Server on port {}", config.http_listen_port);

    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(validator);

        let mut labels = HashMap::new();
        labels.insert(
            "host".to_string(),
            format!("{:?}", gethostname())
        );
        let prometheus = PrometheusMetricsBuilder::new(&config.prometheus_namespace)
            .endpoint(&config.prometheus_metrics_path)
            .const_labels(labels)
            .build()
            .unwrap();
    
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .wrap(prometheus.clone())
            .service(health)

            // logs
            .wrap(Logger::default())
            .wrap(Logger::new("%a %{User-Agent}i"))

            .service(
                web::scope("/api")
                        // api format
                        .wrap(DefaultHeaders::new().add(ContentType::json()))

                        // auth
                        .wrap(auth)
                        // routes
                        .service(http::controllers::quotes::list)
                        .service(http::controllers::quotes::item)
                        .service(http::controllers::quotes::delete)
                        .service(http::controllers::quotes::add)
                        .service(http::controllers::quotes::update)
                        .service(health_json)

            )

            .service(
                Redirect::new("/", "/swagger-ui/").using_status_code(StatusCode::MOVED_PERMANENTLY)
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind((
        config.http_listen_ip.as_str(),
        config.http_listen_port.to_string().parse::<u16>().unwrap()
    ))?
    .workers(
        config.http_server_num_worker
    )
    .max_connections(
        config.http_server_max_connexion
    )
    .server_hostname(
        config.http_server_hostname
    )
    .run()
    .await
}


#[actix_web::test]
async fn test_index_without_jwt() {
    use actix_web::test;
    use actix_web::http::StatusCode;

    dotenv().ok();
    let auth = HttpAuthentication::bearer(validator);
    let app = test::init_service(
        App::new()
            .wrap(auth)
            .service(health_json)
    ).await;
    let req = test::TestRequest::get().uri("/api/health")
        .insert_header(ContentType::json())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status() == StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_index_with_jwt() {
    use actix_web::test;

    dotenv().ok();
    let auth = HttpAuthentication::bearer(validator);

    let app = test::init_service(
        App::new()
            .wrap(auth)
            .service(health_json)
    ).await;
    let req = test::TestRequest::get().uri("/health")
        .insert_header(ContentType::json())
        .insert_header(("Authorization", format!("Bearer {}" ,create_jwt())))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let status = resp.status();
    let body = test::read_body(resp).await;
    println!("Out: {:?} Status: {}", std::str::from_utf8(&body), status.as_str());
    assert!(status.is_success());
}
