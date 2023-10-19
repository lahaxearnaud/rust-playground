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

async fn health() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("Ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    simple_logger::init_with_env().unwrap();

    let env_jwt_secret = env::var("JWT_SECRET");
    match env_jwt_secret {
        Ok(_) => (),
        Err(_) => panic!("JWT_SECRET env var must be defined"),
    }

    log::info!("JWT: {}", create_jwt());


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

    HttpServer::new(move || {
        let auth = HttpAuthentication::bearer(validator);

        let mut labels = HashMap::new();
        labels.insert(
            "host".to_string(),
            format!("{:?}", gethostname())
        );
        let prometheus = PrometheusMetricsBuilder::new(
            env::var("PROMETHEUS_NAMESPACE")
                .unwrap_or(
                    "rustplayground".to_string()
                ).as_ref()
        )
            .endpoint(
                env::var("PROMETHEUS_METRICS_PATH")
                    .unwrap_or(
                        "/metrics".to_string()
                    ).as_ref()
            )
            .const_labels(labels)
            .build()
            .unwrap();
    

        App::new()
            .wrap(prometheus.clone())
            .service(web::resource("/health").to(health))

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

            )

            .service(
                Redirect::new("/", "/swagger-ui/").using_status_code(StatusCode::MOVED_PERMANENTLY)
            )
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind((
        env::var("HTTP_LISTEN_IP")
            .unwrap_or(
                "127.0.0.1".to_string()
            ),
        env::var("HTTP_LISTEN_PORT")
            .unwrap_or(
                "8080".to_string()
            )
            .to_string()
            .parse::<u16>()
            .unwrap()
    ))?
    .workers(
        env::var("HTTP_SERVER_NUM_WORKERS")
                .unwrap_or(
                    "5".to_string()
                )
                .to_string()
                .parse::<usize>()
                .unwrap()
    )
    .max_connections(
        env::var("HTTP_SERVER_MAX_CONNEXION")
                .unwrap_or(
                    "5".to_string()
                )
                .to_string()
                .parse::<usize>()
                .unwrap()
    )
    .server_hostname(
        env::var("HTTP_SERVER_HOSTNAME")
                .unwrap_or(
                    "127.0.0.1".to_string()
                )
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
    use actix_web::test;

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
