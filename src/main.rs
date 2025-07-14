use actix_web::{HttpResponse, Responder};

mod course;
mod metextbook;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    actix_web::HttpServer::new(move || actix_web::App::new().service(hello))
        .bind("127.0.0.1:65534")?
        .run()
        .await?;

    Ok(())
}

#[actix_web::get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("hello")
}
