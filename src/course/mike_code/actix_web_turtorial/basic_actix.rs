mod project {
    use actix_web::{App, HttpResponse, HttpServer, Responder, web};
    use serde::{Deserialize, Serialize};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    // use surrealdb::engine::remote::ws::Ws;

    #[actix_web::test]
    async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // let _sdb = surrealdb::Surreal::new::<Ws>("127.0.0.1:65535").await?;

        let _side_running_server = tokio::spawn(async move {
            let web_shared_bag = ArcBag {
                temp_counter: Arc::new(Mutex::new(0u32)),
                world_line: Arc::new(Mutex::new(2.712f64)),
            };

            HttpServer::new(move || {
                App::new()
                    .app_data(web::Data::new(web_shared_bag.clone()))
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
                    .service(user_json_request)
                    .service(response_json)
                    .service(arc_bag_temp_counter_add_one)
                    .service(change_world_line)
            })
            .bind("127.0.0.1:65534")
            .unwrap()
            .run()
            .await
            .unwrap()
        });

        let assert_part = tokio::spawn(async move {
            let rqs_client = reqwest::Client::new();

            let user_id_webpage = "/user/37";
            let user_id_msg = "id is 37";
            assert_get_func(&rqs_client, user_id_webpage, user_id_msg)
                .await
                .unwrap();

            let jet_rocket_webpage = "/jet_rocket?destination=NewYork&code=U7787";
            let jet_rocket_msg = "The rocket U7787 is heading NewYork";
            assert_get_func(&rqs_client, jet_rocket_webpage, jet_rocket_msg)
                .await
                .unwrap();

            let user_json_webpage = "/user_json_request";
            let user_json_body = User {
                name: "Adolf".into(),
                age: 27u32,
            };
            let user_json_msg = "User name: Adolf, User age: 27";
            assert_post_func(
                &rqs_client,
                user_json_webpage,
                &user_json_body,
                user_json_msg,
            )
            .await
            .unwrap();

            let response_json_webpage = "/response_json";
            let response_json_msg = serde_json::to_string(&User {
                name: "Newton".into(),
                age: 52u32,
            })
            .unwrap();
            assert_get_func(&rqs_client, response_json_webpage, &response_json_msg)
                .await
                .unwrap();

            let temp_counter_add_webapge = "/arc_bag_temp_counter_add_one";
            let temp_counter_add_msg = "temp counter added one, now temp counter is 1";
            assert_get_func(&rqs_client, temp_counter_add_webapge, temp_counter_add_msg)
                .await
                .unwrap();

            let world_line_webapge = "/change_world_line";
            let world_line_msg = format!(
                "World line been changed into {}",
                2.712f64.powf(2.712f64) - 2f64
            );
            assert_get_func(&rqs_client, world_line_webapge, &world_line_msg)
                .await
                .unwrap();
        });

        assert_part.await?;
        Ok(())
    }

    //
    // ----- ----- ----- ----- main ended here ----- ----- ----- ----- -----
    //

    #[derive(Serialize, Deserialize)]
    struct JetRocket {
        destination: String,
        code: String,
    }

    #[derive(Serialize, Deserialize)]
    struct User {
        name: String,
        age: u32,
    }

    #[derive(Clone)]
    struct ArcBag {
        temp_counter: Arc<Mutex<u32>>,
        world_line: Arc<Mutex<f64>>,
    }
    impl ArcBag {
        async fn add_one_on_temp_counter(&self) -> u32 {
            let mut counter = self.temp_counter.lock().await.clone();
            counter += 1;
            counter
        }

        async fn change_world_line_randomly(&self) -> f64 {
            let mut world = self.world_line.lock().await.clone();
            world = world.powf(world) - 2f64;
            world
        }
    }

    //
    // ----- ----- ----- ----- struct defined ended here ----- ----- ----- ----- -----
    //

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

    #[actix_web::post("/user_json_request")]
    async fn user_json_request(user: web::Json<User>) -> impl Responder {
        let msg = format!("User name: {}, User age: {}", user.name, user.age);
        HttpResponse::Ok().body(msg)
    }

    #[actix_web::get("/response_json")]
    async fn response_json() -> impl Responder {
        let person = User {
            name: "Newton".into(),
            age: 52u32,
        };
        let person_json = serde_json::to_value(&person).unwrap();
        HttpResponse::Ok().json(person_json)
    }

    #[actix_web::get("/arc_bag_temp_counter_add_one")]
    async fn arc_bag_temp_counter_add_one(bag: web::Data<ArcBag>) -> impl Responder {
        let counter = bag.add_one_on_temp_counter().await;
        let msg = format!("temp counter added one, now temp counter is {}", counter);
        HttpResponse::Ok().body(msg)
    }

    #[actix_web::get("/change_world_line")]
    async fn change_world_line(bag: web::Data<ArcBag>) -> impl Responder {
        let world = bag.change_world_line_randomly().await;
        let msg = format!("World line been changed into {}", world);
        HttpResponse::Ok().body(msg)
    }

    //
    // ----- ----- ----- ----- web func defined ended here ----- ----- ----- ----- -----
    //
    async fn assert_get_func(
        x: &reqwest::Client,
        webpage: &str,
        msg: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        assert_eq!(
            x.get(format!("http://127.0.0.1:65534{}", webpage))
                .send()
                .await?
                .text()
                .await?,
            msg
        );
        Ok(())
    }

    async fn assert_post_func(
        x: &reqwest::Client,
        webpage: &str,
        body: &impl Serialize,
        msg: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        assert_eq!(
            x.post(format!("http://127.0.0.1:65534{}", webpage))
                .json(body)
                .send()
                .await?
                .text()
                .await?,
            msg
        );
        Ok(())
    }

    //
    // ----- ----- ----- ----- assert func defined ended here ----- ----- ----- ----- -----
    //
}
