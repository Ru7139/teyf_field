mod project {
    use actix_web::{App, HttpResponse, HttpServer, Responder, web};
    use serde::Deserialize;

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
                    .service(user_id)
                    .service(jet_rocket)
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

        let rsps = rqs_client
            .get("http://127.0.0.1:65534/jet_rocket?destination=NewYork&code=U7787")
            .send()
            .await?;

        assert_eq!(rsps.text().await?, "The rocket U7787 is heading NewYork");

        // _side_running_server.await?;
        Ok(())
    }

    #[actix_web::get("/hello")]
    async fn hello() -> impl Responder {
        HttpResponse::Ok().body("hello")
    }

    #[actix_web::get("/user/{id}")]
    async fn user_id(path: web::Path<i32>) -> impl Responder {
        let id = path.into_inner();
        let msg = format!("id is {}", id);
        HttpResponse::Ok().body(msg)
    }

    #[actix_web::get("/jet_rocket")]
    async fn jet_rocket(data: web::Query<JetRocket>) -> impl Responder {
        let msg = format!("The rocket {} is heading {}", data.code, data.destination);
        HttpResponse::Ok().body(msg)
    }

    #[derive(Deserialize)]
    struct JetRocket {
        destination: String,
        code: String,
    }
}
