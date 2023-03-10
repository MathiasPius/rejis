#[cfg(feature = "derive")]
mod filtering {
    use rejis::{
        filter::And,
        filter::Operator::{Equal, Like, NotEqual},
        Executor, Table,
    };

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
    fn single_property_equality() {
        let db = testutils::user_database();

        let johns = User::query()
            .first_name
            .cmp(Equal, "John")
            .get(&db)
            .unwrap();

        assert_eq!(johns.len(), 2);
        assert!(johns.iter().all(|john| john.first_name == "John"));
    }

    #[test]
    fn single_property_inequality() {
        let db = testutils::user_database();

        let non_smiths = User::query()
            .last_name
            .cmp(NotEqual, "Smith")
            .get(&db)
            .unwrap();

        assert_eq!(non_smiths.len(), 3);
        assert!(non_smiths
            .iter()
            .all(|non_smith| non_smith.last_name != "Smith"));
    }

    #[test]
    fn multi_property_equality() {
        let db = testutils::user_database();

        // Find all John Smiths
        let john_smith = And(
            User::query().first_name.cmp(Equal, "John"),
            User::query().last_name.cmp(Equal, "Smith"),
        )
        .get(&db)
        .unwrap();

        assert_eq!(john_smith.len(), 1);
        assert_eq!(john_smith[0].first_name, "John");
        assert_eq!(john_smith[0].last_name, "Smith");
        assert_eq!(john_smith[0].age, 32);
    }

    #[test]
    fn multi_property_inequality() {
        let db = testutils::user_database();

        // Find all non-Smith Johns
        let john_smith = And(
            User::query().first_name.cmp(Equal, "John"),
            User::query().last_name.cmp(NotEqual, "Smith"),
        )
        .get(&db)
        .unwrap();

        println!("{john_smith:?}");

        assert_eq!(john_smith.len(), 1);
        assert_eq!(john_smith[0].first_name, "John");
        assert_eq!(john_smith[0].last_name, "Anderson");
        assert_eq!(john_smith[0].age, 48);
    }

    #[test]
    fn like_matching() {
        let db = testutils::user_database();

        let jays = User::query().first_name.cmp(Like, "J%").get(&db).unwrap();

        // Should yield John Smith, Jane Smith and John Anderson
        assert_eq!(jays.len(), 3);
    }

    #[test]
    fn array_matching() {
        let db = testutils::user_database();

        let garfield_owners = User::query()
            .pets
            .any(|query| query.name.clone(), Like, "Jimmy")
            .get(&db)
            .unwrap();

        println!("{:#?}", garfield_owners);
    }
}
