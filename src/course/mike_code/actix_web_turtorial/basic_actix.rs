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
                    .service(user)
            })
            .bind("127.0.0.1:65534")
            .unwrap()
            .run()
            .await
            .unwrap()
        });

        let rqs_client = reqwest::Client::new();
        let rqs = rqs_client
            .get("http://127.0.0.1:65534/user/37")
            .send()
            .await?;

        assert_eq!(rqs.text().await?, "id is 37");

        // _side_running_server.await?;
        Ok(())
    }

    #[actix_web::get("/hello")]
    async fn hello() -> impl Responder {
        HttpResponse::Ok().body("hello")
    }

    #[actix_web::get("/user/{id}")]
    async fn user(path: web::Path<i32>) -> impl Responder {
        let id = path.into_inner();
        let msg = format!("id is {}", id);
        HttpResponse::Ok().body(msg)
    }
}
