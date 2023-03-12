use rejis::{
    filter::{
        And,
        Operator::{self, Equal},
    },
    Executor, Query, QueryConstructor, Queryable, Table,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Pet {
    name: String,
}

#[derive(Debug, Clone)]
struct PetQuery<Root: Table> {
    name: Query<String, Root>,
}

impl<Root: Table> Queryable<Root> for Pet {
    type QueryType = PetQuery<Root>;
}

impl<Root: Table> QueryConstructor<Root> for PetQuery<Root> {
    type Inner = User;

    fn new<Field>(path: &rejis::path::Path) -> Self
    where
        Field: Queryable<Root>,
    {
        PetQuery {
            name: Query::new(path.join("name")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    first_name: String,
    last_name: String,
    pets: Vec<Pet>,
}

// Queryable implementation
impl<Root: Table> Queryable<Root> for User {
    type QueryType = UserQuery<Root>;
}

#[derive(Debug, Clone)]
struct UserQuery<Root: Table> {
    first_name: Query<String, Root>,
    last_name: Query<String, Root>,
    pets: Query<Vec<Pet>, Root>,
}

impl<Root: Table> QueryConstructor<Root> for UserQuery<Root> {
    type Inner = User;

    fn new<Field>(path: &rejis::path::Path) -> Self
    where
        Field: Queryable<Root>,
    {
        UserQuery {
            first_name: Query::new(path.join("first_name")),
            last_name: Query::new(path.join("last_name")),
            pets: Query::new(path.join("pets")),
        }
    }
}

// Table implementation
impl Table for User {
    const TABLE_NAME: &'static str = "user";
}

#[test]
fn insert_and_query() {
    let db = Connection::open_in_memory().unwrap();

    User::init(&db).unwrap();

    User {
        first_name: String::from("John"),
        last_name: String::from("Smith"),
        pets: vec![Pet {
            name: String::from("Garfield"),
        }],
    }
    .insert(&db)
    .unwrap();

    let john_smith = And(
        User::query().first_name.cmp(Equal, "John"),
        User::query().last_name.cmp(Equal, "Smith"),
    )
    .get(&db)
    .unwrap();

    assert_eq!(john_smith.len(), 1);
    assert_eq!(john_smith[0].first_name, "John");
    assert_eq!(john_smith[0].last_name, "Smith");
}

#[test]
fn filter_deletion() {
    let db = Connection::open_in_memory().unwrap();

    User::init(&db).unwrap();

    User {
        first_name: String::from("John"),
        last_name: String::from("Smith"),
        pets: vec![Pet {
            name: String::from("Jimmy"),
        }],
    }
    .insert(&db)
    .unwrap();

    let smiths_with_jimmies =
        User::query()
            .pets
            .any(|pet| pet.name.clone(), Operator::Equal, "Jimmy");

    let jane = smiths_with_jimmies.get(&db).unwrap();
    assert_eq!(jane.len(), 1);

    smiths_with_jimmies.delete(&db).unwrap();

    let jane = smiths_with_jimmies.get(&db).unwrap();
    assert_eq!(jane.len(), 0);
}

#[test]
fn query_uninitialized_table() {
    let db = Connection::open_in_memory().unwrap();

    // Attempt to fetch a user from a database which has
    // not been initialized with a user table.
    User::query()
        .first_name
        .cmp(Operator::Equal, "Jimmy")
        .get(&db)
        .unwrap_err();
}

#[test]
fn debug_printing_transforms() {
    let query = And(
        User::query()
            .pets
            .any(|query| query.name.clone(), Operator::GreaterThan, "Lol"),
        User::query().first_name.cmp(Operator::LessThan, "xyz"),
    );

    println!("{:#?}", query);
}
