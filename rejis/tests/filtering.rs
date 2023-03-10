use rejis::{
    filter::And,
    filter::Operator::{Equal, Like, NotEqual},
    Database, Table,
};
use rusqlite::Connection;
use utils::User;

mod utils {
    use rejis::{Queryable, Table};
    use serde::{Deserialize, Serialize};

    #[derive(Queryable, Table, Serialize, Deserialize, Debug, Clone)]
    pub struct User {
        pub first_name: String,
        pub last_name: String,
        pub age: u8,
    }
}

/// Utility function providing a database pre-seeded with a number of different users
fn user_database() -> Database {
    // Open an in-memory database, create the table and populate it with
    // three users.
    let db = Database::new(Connection::open_in_memory().unwrap());
    db.create_table::<User>().unwrap();

    // John Smith
    db.insert(User {
        first_name: String::from("John"),
        last_name: String::from("Smith"),
        age: 32,
    });

    // Jane Smith
    db.insert(User {
        first_name: String::from("Jane"),
        last_name: String::from("Smith"),
        age: 35,
    });

    // Thomas Anderson
    db.insert(User {
        first_name: String::from("Thomas"),
        last_name: String::from("Anderson"),
        age: 24,
    });

    // John Anderson
    db.insert(User {
        first_name: String::from("John"),
        last_name: String::from("Anderson"),
        age: 48,
    });

    // Richard LaFleur
    db.insert(User {
        first_name: String::from("Richard"),
        last_name: String::from("LaFleur"),
        age: 36,
    });

    db
}

#[test]
fn single_property_equality() {
    let db = user_database();

    let johns = db.get(User::query().first_name.cmp(Equal, "John")).unwrap();

    assert_eq!(johns.len(), 2);
    assert!(johns.iter().all(|john| john.first_name == "John"));
}

#[test]
fn single_property_inequality() {
    let db = user_database();

    let non_smiths = db
        .get(User::query().last_name.cmp(NotEqual, "Smith"))
        .unwrap();

    assert_eq!(non_smiths.len(), 3);
    assert!(non_smiths
        .iter()
        .all(|non_smith| non_smith.last_name != "Smith"));
}

#[test]
fn multi_property_equality() {
    let db = user_database();

    // Find all John Smiths
    let john_smith = db
        .get(And((
            User::query().first_name.cmp(Equal, "John"),
            User::query().last_name.cmp(Equal, "Smith"),
        )))
        .unwrap();

    assert_eq!(john_smith.len(), 1);
    assert_eq!(john_smith[0].first_name, "John");
    assert_eq!(john_smith[0].last_name, "Smith");
    assert_eq!(john_smith[0].age, 32);
}

#[test]
fn multi_property_inequality() {
    let db = user_database();

    // Find all non-Smith Johns
    let john_smith = db
        .get(And((
            User::query().first_name.cmp(Equal, "John"),
            User::query().last_name.cmp(NotEqual, "Smith"),
        )))
        .unwrap();

    println!("{john_smith:?}");

    assert_eq!(john_smith.len(), 1);
    assert_eq!(john_smith[0].first_name, "John");
    assert_eq!(john_smith[0].last_name, "Anderson");
    assert_eq!(john_smith[0].age, 48);
}

#[test]
fn like_matching() {
    let db = user_database();

    let jays = db.get(User::query().first_name.cmp(Like, "J%")).unwrap();

    // Should yield John Smith, Jane Smith and John Anderson
    assert_eq!(jays.len(), 3);
}
