use rejis::{
    filter::{
        And,
        Operator::{self, Equal},
    },
    Database, Query, QueryConstructor, Queryable, Table,
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

    fn new<Field>(path: &rejis::Path) -> Self
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

    fn new<Field>(path: &rejis::Path) -> Self
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

    db.init::<User>().unwrap();

    db.insert(&User {
        first_name: String::from("John"),
        last_name: String::from("Smith"),
        pets: vec![Pet {
            name: String::from("Garfield"),
        }],
    })
    .unwrap();

    let john_smith = db
        .get(&And(
            User::query().first_name.cmp(Equal, "John"),
            User::query().last_name.cmp(Equal, "Smith"),
        ))
        .unwrap();

    assert_eq!(john_smith.len(), 1);
    assert_eq!(john_smith[0].first_name, "John");
    assert_eq!(john_smith[0].last_name, "Smith");
}

#[test]
fn filter_deletion() {
    let db = Connection::open_in_memory().unwrap();

    db.init::<User>().unwrap();

    db.insert(&User {
        first_name: String::from("John"),
        last_name: String::from("Smith"),
        pets: vec![Pet {
            name: String::from("Jimmy"),
        }],
    })
    .unwrap();

    let smiths_with_jimmies =
        User::query()
            .pets
            .any(|pet| pet.name.clone(), Operator::Equal, "Jimmy");

    let jane = db.get(&smiths_with_jimmies).unwrap();
    assert_eq!(jane.len(), 1);

    db.delete(&smiths_with_jimmies).unwrap();

    let jane = db.get(&smiths_with_jimmies).unwrap();
    assert_eq!(jane.len(), 0);
}

#[test]
fn query_uninitialized_table() {
    let db = Connection::open_in_memory().unwrap();

    // Attempt to fetch a user from a database which has
    // not been initialized with a user table.
    db.get(&User::query().first_name.cmp(Operator::Equal, "Jimmy"))
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
