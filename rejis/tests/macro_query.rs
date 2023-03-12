#[cfg(all(feature = "macros", feature = "derive"))]
mod macros {
    use rejis::Q;

    use rejis::{Executor, Table};
    use testutils::user_database;
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
    fn simple_filtering_dsl() {
        let db = user_database();

        let johns = Q! {
            User.first_name == "John"
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", johns);
        assert_eq!(johns.len(), 2);
    }

    #[test]
    fn multi_filtering_dsl() {
        let db = user_database();

        let johns = Q! {
            (User.first_name == "John") && (User.last_name != "Smith")
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", johns);
        assert_eq!(johns.len(), 1);
        assert_eq!(johns[0].last_name, "Anderson");
    }

    #[test]
    fn multi_filtering_or_dsl() {
        let db = user_database();

        let johns = Q! {
            (
                (User.first_name == "John") && (User.last_name == "Smith")
            ) ||
            (
                (User.first_name == "Thomas") && (User.last_name == "Anderson")
            )
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", johns);
        assert_eq!(johns.len(), 2);
    }

    #[test]
    fn expr_query() {
        let db = user_database();

        let first_name = vec!["John"];

        let johns = Q! {
            User.first_name == &first_name[0]
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", johns);
        assert_eq!(johns.len(), 2);
    }

    #[test]
    fn ident_query() {
        let db = user_database();

        let first_name = "John";

        let johns = Q! {
            User.first_name == first_name
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", johns);
        assert_eq!(johns.len(), 2);
    }

    #[test]
    fn any_query_literal() {
        let db = user_database();

        let garfield_owners = Q! {
            User.pets[..].name == "Garfield"
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", garfield_owners);
        assert_eq!(garfield_owners.len(), 1);
    }

    #[test]
    fn any_query_ident() {
        let db = user_database();

        let name = "Garfield";

        let garfield_owners = Q! {
            User.pets[..].name == name
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", garfield_owners);
        assert_eq!(garfield_owners.len(), 1);
    }

    #[test]
    fn any_query_complex() {
        let db = user_database();

        let name = vec!["Garfield"];

        let garfield_owners = Q! {
            User.pets[..].name == &name[0]
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", garfield_owners);
        assert_eq!(garfield_owners.len(), 1);
    }

    #[test]
    fn any_query_multiples() {
        let db = user_database();

        let jimmy_owners = Q! {
            User.pets[..].name == "Jimmy"
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", jimmy_owners);
        assert_eq!(jimmy_owners.len(), 2);
    }

    #[test]
    fn complex_any_query() {
        let db = user_database();

        let jane = Q! {
            (User.pets[..].name == "Jimmy") && (User.last_name == "Smith")
        }
        .get(&db)
        .unwrap();

        println!("{:#?}", jane);
        assert_eq!(jane.len(), 1);
        println!("{:#?}", jane[0]);
    }

    #[test]
    fn filter_deletion() {
        let db = user_database();

        let smiths_with_jimmies = Q! {
            (User.pets[..].name == "Jimmy") && (User.last_name == "Smith")
        };

        let jane = smiths_with_jimmies.get(&db).unwrap();
        assert_eq!(jane.len(), 1);

        smiths_with_jimmies.delete(&db).unwrap();

        let jane = smiths_with_jimmies.get(&db).unwrap();
        assert_eq!(jane.len(), 0);
    }
}
