mod project {
    use actix_web::{App, HttpResponse, HttpServer, web};

    #[actix_web::test]
    async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _side_running_server = tokio::spawn(async move {
            HttpServer::new(|| {
                App::new().route(
                    "/",
                    web::get().to(|| async { HttpResponse::Ok().body("Hello World".to_string()) }),
                )
            })
            .bind("127.0.0.1:65534")
            .unwrap()
            .run()
            .await
            .unwrap()
        });

        Ok(())
    }
}
