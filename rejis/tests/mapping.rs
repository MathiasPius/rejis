#[cfg(feature = "derive")]
mod mapping {
    use rejis::{filter::Operator::Equal, transform::Transform, Executor, Table};

    use testutils::User;

    mod testutils {
        use rejis::{Queryable, Table};
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

            User::init(&db).unwrap();

            // John Smith
            User {
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
            }
            .insert(&db)
            .unwrap();

            // Jane Smith
            User {
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
            }
            .insert(&db)
            .unwrap();

            // Thomas Anderson
            User {
                first_name: String::from("Thomas"),
                last_name: String::from("Anderson"),
                age: 24,
                pets: vec![],
            }
            .insert(&db)
            .unwrap();

            // John Anderson
            User {
                first_name: String::from("John"),
                last_name: String::from("Anderson"),
                age: 48,
                pets: vec![Pet {
                    name: String::from("Jimmy"),
                }],
            }
            .insert(&db)
            .unwrap();

            // Richard LaFleur
            User {
                first_name: String::from("Richard"),
                last_name: String::from("LaFleur"),
                age: 36,
                pets: vec![],
            }
            .insert(&db)
            .unwrap();

            db
        }
    }

    #[test]
    fn mapping_into_pets() {
        let db = testutils::user_database();

        // Find ages of all Johns
        let ages = User::query()
            .first_name
            .cmp(Equal, "John")
            .map(&User::query().age)
            .get(&db)
            .unwrap();

        println!("{ages:?}");
        assert_eq!(ages.len(), 2);
    }
}
