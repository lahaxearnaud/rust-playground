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
    Result
};
use uuid::Uuid;

#[utoipa::path(
    path = "/api/quotes",
    responses(
        (status = 200, description = "List current quote items", body = [Quote])
    ),
    security(
        ("token" = [])
    )
)]
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

#[utoipa::path(
    path = "/api/quotes/{quote_id}",
    responses(
        (status = 200, description = "Quote created successfully", body = Quote),
        (status = 404, description = "Quote not found")
    ),
    security(
        ("token" = [])
    )
)]
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

#[utoipa::path(
    path = "/api/quotes/{quote_id}",
    responses(
        (status = 200, description = "Quote deleted"),
    ),
    security(
        ("token" = [])
    )
)]
#[delete("/quotes/{quote_id}")]
pub async fn delete(path: Path<String>) -> impl Responder {
    let quote_id = path.into_inner();
    let quote_repository = QuoteRepository;
    let _ = quote_repository.remove(quote_id);

    HttpResponse::Ok()
}

#[utoipa::path(
    path = "/api/quotes",
    request_body = ApiPayloadQuote,
    responses(
        (status = 201, description = "Quote created successfully", body = Quote),
        (status = 406, description = "Validation error", body = ValidationErrors)
    ),
    security(
        ("token" = [])
    )
)]
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

#[utoipa::path(
    path = "/api/quotes/{quote_id}",
    request_body = ApiPayloadQuote,
    responses(
        (status = 201, description = "Quote created successfully", body = Quote),
        (status = 406, description = "Validation error", body = ValidationErrors),
        (status = 404, description = "Quote not found")
    ),
    security(
        ("token" = [])
    )
)]
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

#[actix_web::test]
async fn test_get_list() {
    use actix_web::test;
    use dotenv::dotenv;
    use actix_web::App;

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
async fn test_get_item() {
    use actix_web::test;
    use dotenv::dotenv;
    use actix_web::App;

    dotenv().ok();
    let app = test::init_service(App::new().service(http::controllers::quotes::item)).await;
    let req = test::TestRequest::get().uri("/quotes/072f58a7-4150-431e-3729-60aea434088e")
        .insert_header(ContentType::json())
        .to_request();
    let resp = test::call_service(&app, req).await;
    let success = resp.status().is_success();

    let body = test::read_body(resp).await;
    println!("Out: {:?}", std::str::from_utf8(&body));

    assert!(success);
}

#[actix_web::test]
async fn test_post() {
    use actix_web::test;
    use dotenv::dotenv;
    use actix_web::App;

    dotenv().ok();

    let app = test::init_service(
        App::new()
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


#[actix_web::test]
async fn test_put() {
    use actix_web::test;
    use dotenv::dotenv;
    use actix_web::App;

    dotenv().ok();

    let app = test::init_service(
        App::new()
        .service(http::controllers::quotes::update)
    ).await;
    let req = test::TestRequest::put().uri("/quotes/172f58a7-3729-431e-aa80-9189c808623c")
        .insert_header(ContentType::json())
        .set_json(ApiPayloadQuote{quote: "Nouvelle technique : on passe pour des cons, les autres se marrent, et on frappe. C’est nouveau. ".to_string(), author: "Tintin le beau".to_string()})
        .to_request();
    let resp = test::call_service(&app, req).await;
    let success = resp.status().is_success();

    let body = test::read_body(resp).await;
    println!("Out: {:?}", std::str::from_utf8(&body));

    assert!(success);
}
