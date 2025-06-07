mod project {
    use actix_web::{App, HttpResponse, HttpServer, Responder, web};

    #[actix_web::test]
    async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _side_running_server = tokio::spawn(async move {
            HttpServer::new(|| {
                App::new()
                    .service(
                        web::resource("/method")
                            .route(web::get().to(|| async { HttpResponse::Ok().body("Get") }))
                            .route(web::post().to(|| async { HttpResponse::Ok().body("Post") }))
                            .route(web::delete().to(|| async { HttpResponse::Ok().body("Delete") }))
                            .route(web::put().to(|| async { HttpResponse::Ok().body("Put") })),
                    )
                    .service(hello)
                    .service(world)
            })
            .bind("127.0.0.1:65534")
            .unwrap()
            .run()
            .await
            .unwrap()
        });

        let rqs_client = reqwest::Client::new();
        let rqs = rqs_client
            .get("http://127.0.0.1:65534/hello")
            .send()
            .await?;

        assert_eq!(rqs.text().await?, "hello");

        // _side_running_server.await?;
        Ok(())
    }

    #[actix_web::get("/hello")]
    async fn hello() -> impl Responder {
        HttpResponse::Ok().body("hello")
    }

    #[actix_web::get("/world")]
    async fn world() -> impl Responder {
        HttpResponse::Ok().body("world")
    }
}
