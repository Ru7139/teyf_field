mod project {
    use surrealdb::{engine::remote::ws::Ws, opt::auth::Root};

    use super::super::db;

    #[tokio::test]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let db_port: u16 = 7778u16;
        db::db_start(db_port).await;
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        let db = surrealdb::Surreal::new::<Ws>(format!("127.0.0.1:{}", db_port)).await?;

        db.signin(Root {
            username: "ruut",
            password: "ruut",
        })
        .await?;

        db.use_ns("alpha_ns").use_db("alpha_db").await?;

        Ok(())
    }
}
