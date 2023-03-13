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
pub fn user_database() -> Connection {
    // Open an in-memory database, create the table and populate it with
    // three users.
    let db = Connection::open_in_memory().unwrap();

    db.init::<User>().unwrap();

    // John Smith
    db.insert(&User {
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
    })
    .unwrap();

    // Jane Smith
    db.insert(&User {
        first_name: String::from("Jane"),
        last_name: String::from("Smith"),
        age: 35,
        pets: vec![
            Pet {
                name: String::from("Jimmy"),
            },
            Pet {
                name: String::from("Jimmy"),
            },
        ],
    })
    .unwrap();

    // Thomas Anderson
    db.insert(&User {
        first_name: String::from("Thomas"),
        last_name: String::from("Anderson"),
        age: 24,
        pets: vec![],
    })
    .unwrap();

    // John Anderson
    db.insert(&User {
        first_name: String::from("John"),
        last_name: String::from("Anderson"),
        age: 48,
        pets: vec![Pet {
            name: String::from("Jimmy"),
        }],
    })
    .unwrap();

    // Richard LaFleur
    db.insert(&User {
        first_name: String::from("Richard"),
        last_name: String::from("LaFleur"),
        age: 36,
        pets: vec![],
    })
    .unwrap();

    db
}
