mod project {
    use serde::{Deserialize, Serialize};
    use surrealdb::{RecordId, engine::remote::ws::Ws, opt::auth::Root};

    use super::super::sdb_tools;

    #[tokio::test]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let db_port: u16 = 17779u16;
        let mut command = sdb_tools::CommandLines::new(db_port);
        command.db_start();
        command.display_child_and_command();
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        test_content(db_port).await?;

        command.kill_child();

        Ok(())
    }

    async fn test_content(sdb_port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let sdb = surrealdb::Surreal::new::<Ws>(format!("127.0.0.1:{}", sdb_port)).await?;

        sdb.signin(Root {
            username: "ruut",
            password: "ruut",
        })
        .await?;

        sdb.use_ns("alpha_ns").use_db("alpha_db").await?;

        let record1: Option<Record> = sdb
            .create("user")
            .content(Person {
                name: "Jonmo".into(),
                age: 28u32,
            })
            .await?;

        let record2: Option<Record> = sdb
            .create("user")
            .content(Person {
                name: "Yacci".into(),
                age: 381u32,
            })
            .await?;

        dbg!(&record1.as_ref().unwrap().id);
        dbg!(&record2.as_ref().unwrap().id);

        let person: Vec<Option<User>> = sdb.select("user").await?;
        person.iter().for_each(|x| println!("{:?}", x));

        let person1: Option<User> = sdb
            .select(("user", record1.as_ref().unwrap().id.key().to_string()))
            .await?;
        dbg!(&person1);

        let person_deleted: Option<User> = sdb
            .delete(("user", record1.as_ref().unwrap().id.key().to_string()))
            .await?;
        dbg!("person deleted", &person_deleted);

        let person_update: Option<User> = sdb
            .update(("user", record2.as_ref().unwrap().id.key().to_string()))
            .content(User {
                name: "Anne".into(),
                age: 10u32,
                id: RecordId::from(record2.as_ref().unwrap().id.clone()),
            })
            .await?;
        dbg!("person updated", &person_update);

        let person: Vec<Option<User>> = sdb.select("user").await?;
        person.iter().for_each(|x| println!("{:?}", x));

        Ok(())
    }

    #[allow(unused)]
    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        name: String,
        age: u32,
    }

    #[allow(unused)]
    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct Record {
        id: RecordId,
    }

    #[allow(unused)]
    #[derive(Debug, Serialize, Deserialize)]
    struct User {
        id: RecordId,
        name: String,
        age: u32,
    }
}
