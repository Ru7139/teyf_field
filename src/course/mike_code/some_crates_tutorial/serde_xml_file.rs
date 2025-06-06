mod project {
    use serde::{Deserialize, Serialize};
    use serde_xml_rs::{from_str, to_string};

    #[test]
    #[ignore]
    fn main() {
        let xml = r#"<Person>
            <name>Mike</name>
            <age>20</age>
            </Person>"#; // can not writte as <age>20u32</age>
        let person: Person = from_str(xml).unwrap();
        dbg!(&person);

        let person_xml = to_string(&person).unwrap();
        dbg!(person_xml);
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct Person {
        name: String,
        age: u32,
    }
}
