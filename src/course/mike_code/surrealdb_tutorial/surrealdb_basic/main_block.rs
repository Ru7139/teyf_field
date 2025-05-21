mod project {
    use serde::{Deserialize, Serialize};
    use surrealdb::{RecordId, engine::remote::ws::Ws, opt::auth::Root};

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

        let record1: Option<Record> = db
            .create("user")
            .content(Person {
                name: "Jonmo".into(),
                age: 28u32,
            })
            .await?;

        let record2: Option<Record> = db
            .create("user")
            .content(Person {
                name: "Yacci".into(),
                age: 381u32,
            })
            .await?;

        dbg!(&record1.as_ref().unwrap().id);
        dbg!(&record2.as_ref().unwrap().id);

        let person: Vec<Option<User>> = db.select("user").await?;
        person.iter().for_each(|x| println!("{:?}", x));

        let person1: Option<User> = db
            .select(("user", record1.as_ref().unwrap().id.key().to_string()))
            .await?;
        dbg!(&person1);

        let person_deleted: Option<User> = db
            .delete(("user", record1.as_ref().unwrap().id.key().to_string()))
            .await?;
        dbg!("person deleted", &person_deleted);

        let person_update: Option<User> = db
            .update(("user", record2.as_ref().unwrap().id.key().to_string()))
            .content(User {
                name: "Anne".into(),
                age: 10u32,
                id: RecordId::from(record2.as_ref().unwrap().id.clone()),
            })
            .await?;
        dbg!("person updated", &person_update);

        let person: Vec<Option<User>> = db.select("user").await?;
        person.iter().for_each(|x| println!("{:?}", x));

        // let _ = child.kill();
        // dbg!("command line offline");

        Ok(())
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        name: String,
        age: u32,
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct Record {
        id: RecordId,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct User {
        id: RecordId,
        name: String,
        age: u32,
    }
}
