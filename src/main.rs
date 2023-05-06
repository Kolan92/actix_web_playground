use actix_web::{
    get, middleware, post, web, App, Either, Error, HttpResponse, HttpServer, Responder, Result,
};
use log::info;
use serde::Serialize;

#[derive(Serialize)]
struct MyObj {
    name: String,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[get("/users/{user_id}/{friend}")] // <- define path parameters
async fn with_query_string(path: web::Path<(u32, String)>) -> Result<String> {
    let (user_id, friend) = path.into_inner();
    Ok(format!("Welcome {}, user_id {}!", friend, user_id))
}

#[get("/json/{name}")]
async fn custom_json(name: web::Path<String>) -> Result<impl Responder> {
    let obj = MyObj {
        name: name.to_string(),
    };
    Ok(web::Json(obj))
}

type RegisterResult = Either<HttpResponse, Result<&'static str, Error>>;

//#[get("/either/{value}")]
async fn with_either(value: web::Path<bool>) -> RegisterResult {
    let value = value.into_inner();
    info!("Processing value: {value}");
    if value {
        Either::Right(Ok("Correct data"))
    } else {
        Either::Left(HttpResponse::BadRequest().body("Bad data"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .service(hello)
            .service(echo)
            .service(with_query_string)
            .service(custom_json)
            .route("/either/{value}", web::get().to(with_either))
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::{self};

    #[actix_web::test]
    async fn test_with_either_ok() {
        let value = web::Path::from(true);

        let resp = with_either(value).await;

        match resp {
            Either::Right(value) => assert!(value.is_ok()),
            Either::Left(_) => panic!("Expected ok result"),
        }
    }

    #[actix_web::test]
    async fn test_with_either_not_ok() {
        let value = web::Path::from(false);

        let resp = with_either(value).await;

        match resp {
            Either::Left(resp) => assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST),
            Either::Right(_) => panic!("Expected bad request"),
        }
    }
}
