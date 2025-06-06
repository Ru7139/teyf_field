mod project {
    use actix_web::{App, HttpResponse, HttpServer, web};

    #[actix_web::test]
    async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _side_running_server = tokio::spawn(async move {
            HttpServer::new(|| {
                App::new().service(
                    web::resource("/method")
                        .route(web::get().to(|| async { HttpResponse::Ok().body("Get") }))
                        .route(web::post().to(|| async { HttpResponse::Ok().body("Post") }))
                        .route(web::delete().to(|| async { HttpResponse::Ok().body("Delete") }))
                        .route(web::put().to(|| async { HttpResponse::Ok().body("Put") })),
                )
            })
            .bind("127.0.0.1:65534")
            .unwrap()
            .run()
            .await
            .unwrap()
        });

        // _side_running_server.await?;
        Ok(())
    }
}
