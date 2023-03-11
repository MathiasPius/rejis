#[cfg(feature = "derive")]
mod mapping {
    use rejis::{
        filter::{Filter, Operator::Equal},
        Table,
    };

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
                pets: vec![
                    Pet {
                        name: String::from("Jimmy"),
                    },
                    Pet {
                        name: String::from("Jimmy"),
                    },
                ],
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
                pets: vec![Pet {
                    name: String::from("Jimmy"),
                }],
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
    fn mapping_into_pets() {
        let db = testutils::user_database();

        // Find ages of all Johns
        let ages: Vec<u8> = db
            .get(
                User::query()
                    .first_name
                    .cmp(Equal, "John")
                    .map(&User::query().age),
            )
            .unwrap();

        println!("{ages:?}");
        assert_eq!(ages.len(), 2);
    }
}
