#[cfg(all(feature = "macros", feature = "derive"))]
mod macros {
    use rejis::Database;
    use rejis::Q;

    use rejis::Table;
    use rejis_test_data::user_database;
    use rejis_test_data::User;

    #[test]
    fn simple_filtering_dsl() {
        let db = user_database();

        let johns = db
            .get(&Q! {
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
            .get(&Q! {
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

        let johns = db
            .get(&Q! {
                (
                    (User.first_name == "John") && (User.last_name == "Smith")
                ) ||
                (
                    (User.first_name == "Thomas") && (User.last_name == "Anderson")
                )
            })
            .unwrap();

        println!("{:#?}", johns);
        assert_eq!(johns.len(), 2);
    }

    #[test]
    fn expr_query() {
        let db = user_database();

        let first_name = vec!["John"];

        let johns = db
            .get(&Q! {
                User.first_name == &first_name[0]
            })
            .unwrap();

        println!("{:#?}", johns);
        assert_eq!(johns.len(), 2);
    }

    #[test]
    fn ident_query() {
        let db = user_database();

        let first_name = "John";

        let johns = db
            .get(&Q! {
                User.first_name == first_name
            })
            .unwrap();

        println!("{:#?}", johns);
        assert_eq!(johns.len(), 2);
    }

    #[test]
    fn any_query_literal() {
        let db = user_database();

        let garfield_owners = db
            .get(&Q! {
                User.pets[..].name == "Garfield"
            })
            .unwrap();

        println!("{:#?}", garfield_owners);
        assert_eq!(garfield_owners.len(), 1);
    }

    #[test]
    fn any_query_ident() {
        let db = user_database();

        let name = "Garfield";

        let garfield_owners = db
            .get(&Q! {
                User.pets[..].name == name
            })
            .unwrap();

        println!("{:#?}", garfield_owners);
        assert_eq!(garfield_owners.len(), 1);
    }

    #[test]
    fn any_query_complex() {
        let db = user_database();

        let name = vec!["Garfield"];

        let garfield_owners = db
            .get(&Q! {
                User.pets[..].name == &name[0]
            })
            .unwrap();

        println!("{:#?}", garfield_owners);
        assert_eq!(garfield_owners.len(), 1);
    }

    #[test]
    fn any_query_multiples() {
        let db = user_database();

        let jimmy_owners = db
            .get(&Q! {
                User.pets[..].name == "Jimmy"
            })
            .unwrap();

        println!("{:#?}", jimmy_owners);
        assert_eq!(jimmy_owners.len(), 2);
    }

    #[test]
    fn complex_any_query() {
        let db = user_database();

        let jane = db
            .get(&Q! {
                (User.pets[..].name == "Jimmy") && (User.last_name == "Smith")
            })
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

        let jane = db.get(&smiths_with_jimmies).unwrap();
        assert_eq!(jane.len(), 1);

        db.delete(&smiths_with_jimmies).unwrap();

        let jane = db.get(&smiths_with_jimmies).unwrap();
        assert_eq!(jane.len(), 0);
    }

    #[test]
    fn end_to_end_database() {
        use rejis::Database;
        let conn = user_database();

        let bobby_finder = Q! {
            User.first_name == "Bobby"
        };

        // Make sure Bobby hasn't been inserted yet.
        assert_eq!(conn.get(&bobby_finder).unwrap().len(), 0);

        // Insert our Bobby
        conn.insert(&User {
            first_name: String::from("Bobby"),
            last_name: String::from("Tables"),
            age: 8,
            pets: vec![],
        })
        .unwrap();
        assert_eq!(conn.get(&bobby_finder).unwrap().len(), 1);

        // Delete him again, and make sure he's gone.
        conn.delete(&bobby_finder).unwrap();
        assert_eq!(conn.get(&bobby_finder).unwrap().len(), 0);
    }
}
