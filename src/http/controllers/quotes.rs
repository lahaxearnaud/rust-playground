use crate::http::{self, error};
use crate::db::repositories::quote::QuoteRepository;
use crate::db::entities::quote::{Quote, ApiPayloadQuote};
use actix_web::http::StatusCode;
use actix_web::http::header::ContentType;
use validator::Validate;
use crate::db::pool::DbPool;
use actix_web::web::{Path, Json, self};
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
pub async fn list(pool: web::Data<DbPool>) -> actix_web::Result<HttpResponse, http::error::MyError> {
    let quote_repository = QuoteRepository;

    let quotes = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        return quote_repository.get_quotes(
            Some(1000),
            &mut conn
        );
    })
    .await;

    match quotes {
        Ok(_) => return Ok(HttpResponse::Ok().json(quotes.unwrap().unwrap())),
        Err(_) => return Err(http::error::MyError::NotFount),
    }
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
pub async fn item(path: Path<String>, pool: web::Data<DbPool>) -> Result<HttpResponse, http::error::MyError> {
    let quote_id = path.into_inner();
    let quote_repository = QuoteRepository;

    let quote = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        return quote_repository.get_quote(quote_id, &mut conn);
    })
    .await;

    match quote {
        Ok(_) => return Ok(HttpResponse::Ok().json(quote.unwrap().unwrap())),
        Err(_) => return Err(http::error::MyError::NotFount),
    }
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
pub async fn delete(path: Path<String>, pool: web::Data<DbPool>) -> impl Responder {
    let quote_id = path.into_inner();
    let quote_repository = QuoteRepository;

    let _ = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        return quote_repository.remove(quote_id, &mut conn);
    })
    .await;

    HttpResponse::Ok()
}

#[utoipa::path(
    path = "/api/quotes",
    request_body = ApiPayloadQuote,
    responses(
        (status = 201, description = "Quote created successfully", body = Quote),
        (status = 406, description = "Validation error", body = ValidationErrors),
        (status = 503, description = "Server error")
    ),
    security(
        ("token" = [])
    )
)]
#[post("/quotes")]
pub async fn add(quote_form: Json<ApiPayloadQuote>, pool: web::Data<DbPool>) -> Result<HttpResponse, http::error::MyError> {
    let quote_repository = QuoteRepository;

    let validation = quote_form.validate();

    if validation.is_err() {
        return Ok(HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .insert_header(ContentType::json())
            .json(validation.err()));
    }

    let id = Uuid::new_v4().to_string();
    let new_quote = Quote {
        id: id.clone(),
        author: quote_form.author.to_string(),
        quote: quote_form.quote.to_string()
    };
    let result_quote = new_quote.clone();

    let quote_insert = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        return quote_repository
            .insert(new_quote.clone(), &mut conn);
    })
    .await;

    match quote_insert {
        Ok(_) => return Ok(HttpResponse::Ok().json(result_quote)),
        Err(_) => return Err(http::error::MyError::ServerUnavailable),
    }
}

#[utoipa::path(
    path = "/api/quotes/{quote_id}",
    request_body = ApiPayloadQuote,
    responses(
        (status = 201, description = "Quote created successfully", body = Quote),
        (status = 406, description = "Validation error", body = ValidationErrors),
        (status = 404, description = "Quote not found"),
        (status = 503, description = "Server error")
    ),
    security(
        ("token" = [])
    )
)]
#[put("/quotes/{quote_id}")]
pub async fn update(quote_form: Json<ApiPayloadQuote>, path: Path<String>, pool: web::Data<crate::db::pool::DbPool>) -> Result<HttpResponse, http::error::MyError> {
    let quote_id = path.into_inner();

    let validation = quote_form.validate();

    if validation.is_err() {
        return Ok(HttpResponse::build(StatusCode::NOT_ACCEPTABLE)
            .insert_header(ContentType::json())
            .json(validation.err()));
    }

    let quote_repository = QuoteRepository;

    let quote_update = web::block(move || {
        let mut conn = pool.get().expect("couldn't get db connection from pool");

        let quote_promise = quote_repository.get_quote(quote_id, &mut conn);

        if quote_promise.is_err() {
            return Err(error::MyError::NotFount);
        }

        let mut db_quote = quote_promise.unwrap().clone();
        db_quote.quote = quote_form.quote.to_string();
        db_quote.author = quote_form.author.to_string();

        let update_promise = quote_repository.update(
            db_quote.clone(),
            &mut conn
        );

        match update_promise {
            Ok(_) => return Ok(update_promise.unwrap()),
            Err(_) => return Err(http::error::MyError::BadClientData),
        }        
    })
    .await;

    match quote_update {
        Ok(_) => return Ok(
            HttpResponse::Ok().json(
                quote_update.unwrap().unwrap()
            )
        ),
        Err(_) => return Err(http::error::MyError::ServerUnavailable),
    }
}

#[actix_web::test]
async fn test_get_list() {
    use actix_web::test;
    use dotenv::dotenv;
    use actix_web::App;

    dotenv().ok();
    let pool = crate::db::pool::build_db_pool();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(http::controllers::quotes::list)
    ).await;
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
    let pool = crate::db::pool::build_db_pool();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(http::controllers::quotes::item)
    ).await;
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

    let pool = crate::db::pool::build_db_pool();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
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

    let pool = crate::db::pool::build_db_pool();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(pool.clone()))
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
