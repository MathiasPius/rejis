use rejis::Q;

use rejis::Table;
use testutils::user_database;
use testutils::User;

mod testutils {
    use rejis::{Database, Queryable, Table};
    use rusqlite::Connection;
    use serde::{Deserialize, Serialize};

    #[derive(Queryable, Serialize, Deserialize, Debug, Clone)]
    pub struct Pet {
        pub name: String,
    }

    #[derive(Queryable, Table, Serialize, Deserialize, Debug, Clone)]
    pub struct User {
        pub first_name: String,
        pub last_name: String,
        pub age: u8,
        pub pets: Vec<Pet>,
    }

    /// Utility function providing a database pre-seeded with a number of different users
    pub fn user_database() -> Database {
        // Open an in-memory database, create the table and populate it with
        // three users.
        let db = Database::new(Connection::open_in_memory().unwrap());
        db.create_table::<User>().unwrap();

        // John Smith
        db.insert(User {
            first_name: String::from("John"),
            last_name: String::from("Smith"),
            age: 32,
            pets: vec![
                Pet {
                    name: String::from("Garfield"),
                },
                Pet {
                    name: String::from("Lucky"),
                },
            ],
        });

        // Jane Smith
        db.insert(User {
            first_name: String::from("Jane"),
            last_name: String::from("Smith"),
            age: 35,
            pets: vec![],
        });

        // Thomas Anderson
        db.insert(User {
            first_name: String::from("Thomas"),
            last_name: String::from("Anderson"),
            age: 24,
            pets: vec![],
        });

        // John Anderson
        db.insert(User {
            first_name: String::from("John"),
            last_name: String::from("Anderson"),
            age: 48,
            pets: vec![],
        });

        // Richard LaFleur
        db.insert(User {
            first_name: String::from("Richard"),
            last_name: String::from("LaFleur"),
            age: 36,
            pets: vec![],
        });

        db
    }
}

#[test]
fn simple_filtering_dsl() {
    let db = user_database();

    let johns = db
        .get(Q! {
            User.first_name == "John"
        })
        .unwrap();

    println!("{:#?}", johns);
    assert_eq!(johns.len(), 2);
}

#[test]
fn multi_filtering_dsl() {
    let db = user_database();

    let johns = db
        .get(Q! {
            (User.first_name == "John") && (User.last_name != "Smith")
        })
        .unwrap();

    println!("{:#?}", johns);
    assert_eq!(johns.len(), 1);
    assert_eq!(johns[0].last_name, "Anderson");
}

#[test]
fn multi_filtering_or_dsl() {
    let db = user_database();

    let query = Q! {
        (
            (User.first_name == "John") && (User.last_name == "Smith")
        ) ||
        (
            (User.first_name == "Thomas") && (User.last_name == "Anderson")
        )
    };

    let johns = db.get(query).unwrap();

    println!("{:#?}", johns);
    assert_eq!(johns.len(), 2);
}

#[test]
fn expr_query() {
    let db = user_database();

    let first_name = vec!["John"];

    let query = Q! {
        User.first_name == &first_name[0]
    };

    let johns = db.get(query).unwrap();

    println!("{:#?}", johns);
    assert_eq!(johns.len(), 2);
}

#[test]
fn ident_query() {
    let db = user_database();

    let first_name = "John";

    let query = Q! {
        User.first_name == first_name
    };

    let johns = db.get(query).unwrap();

    println!("{:#?}", johns);
    assert_eq!(johns.len(), 2);
}
