#[cfg(feature = "derive")]
mod filtering {
    use rejis::{
        filter::And,
        filter::Operator::{Equal, Like, NotEqual},
        Database, Table,
    };
    use rejis_test_data::{user_database, User};

    #[test]
    fn single_property_equality() {
        let db = user_database();

        let johns = db
            .get(&User::query().first_name.cmp(Equal, "John"))
            .unwrap();

        assert_eq!(johns.len(), 2);
        assert!(johns.iter().all(|john| john.first_name == "John"));
    }

    #[test]
    fn single_property_inequality() {
        let db = user_database();

        let non_smiths = db
            .get(&User::query().last_name.cmp(NotEqual, "Smith"))
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
            .get(&And(
                User::query().first_name.cmp(Equal, "John"),
                User::query().last_name.cmp(Equal, "Smith"),
            ))
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
            .get(&And(
                User::query().first_name.cmp(Equal, "John"),
                User::query().last_name.cmp(NotEqual, "Smith"),
            ))
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

        let jays = db.get(&User::query().first_name.cmp(Like, "J%")).unwrap();

        // Should yield John Smith, Jane Smith and John Anderson
        assert_eq!(jays.len(), 3);
    }

    #[test]
    fn array_matching() {
        let db = user_database();

        let garfield_owners = db
            .get(
                &User::query()
                    .pets
                    .any(|query| query.name.clone(), Like, "Jimmy"),
            )
            .unwrap();

        println!("{:#?}", garfield_owners);
    }
}
