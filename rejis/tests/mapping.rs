#[cfg(feature = "derive")]
mod mapping {
    use rejis::{filter::Operator::Equal, transform::Transform, Database, Table};
    use rejis_test_data::{user_database, User};

    #[test]
    fn mapping_into_pets() {
        let db = user_database();

        // Find ages of all Johns
        let ages = db
            .get(
                &User::query()
                    .first_name
                    .cmp(Equal, "John")
                    .map(&User::query().age),
            )
            .unwrap();

        println!("{ages:?}");
        assert_eq!(ages.len(), 2);
    }
}
