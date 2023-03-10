use rejis::Q;

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

use rejis::Table;
use testutils::user_database;
use utils::User;

mod testutils {
    use rejis::{Database, Queryable, Table};
    use rusqlite::Connection;
    use serde::{Deserialize, Serialize};

    #[derive(Queryable, Table, Serialize, Deserialize, Debug, Clone)]
    pub struct User {
        pub first_name: String,
        pub last_name: String,
        pub age: u8,
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
            User.first_name == "John" && User.last_name == "Smith"
        })
        .unwrap();

    println!("{:#?}", johns);
    assert_eq!(johns.len(), 1);
}
