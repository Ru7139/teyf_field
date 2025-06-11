mod project {
    use actix_web::{App, HttpResponse, HttpServer, Responder, web};
    use serde::Deserialize;
    use surrealdb::engine::remote::ws::Ws;

    #[actix_web::test]
    async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _sdb = surrealdb::Surreal::new::<Ws>("127.0.0.1:65535").await?;

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

        let assert_get_closure =
            async |x: &reqwest::Client,
                   webpage: &str,
                   msg: &str|
                   -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                assert_eq!(
                    x.get(format!("http://127.0.0.1:65534/{}", webpage))
                        .send()
                        .await?
                        .text()
                        .await?,
                    msg
                );
                Ok(())
            };

        let assert_post_closure =
            async |x: &reqwest::Client,
                   webpage: &str,
                   // body:
                   msg: &str|
                   -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                assert_eq!(
                    x.post(format!("http://127.0.0.1:65534/{}", webpage))
                        // .body()
                        .send()
                        .await?
                        .text()
                        .await?,
                    msg
                );
                Ok(())
            };

        let assert_part = tokio::spawn(async move {
            let rqs_client = reqwest::Client::new();

            let user_id_webpage = "user/37";
            let user_id_msg = "id is 37";
            assert_get_closure(&rqs_client, user_id_webpage, user_id_msg)
                .await
                .unwrap();

            let jet_rocket_webpage = "jet_rocket?destination=NewYork&code=U7787";
            let jet_rocket_msg = "The rocket U7787 is heading NewYork";
            assert_get_closure(&rqs_client, jet_rocket_webpage, jet_rocket_msg)
                .await
                .unwrap();

            let user_json_webpage = "/user_json_request";
            let user_json_msg = "User name: Adolf, User age: 27";
        });

        assert_part.await?;
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

    #[actix_web::post("/user_json_request")]
    async fn json_request(user:actix_web::web::Json<User>) -> impl Responder {
        let msg = format!("User name: {}, User age: {}", user.name, user.age);
        HttpResponse::Ok().body(msg)
    }

    #[derive(Deserialize)]
    struct User {
        name: String,
        age: u32,
    }
}
