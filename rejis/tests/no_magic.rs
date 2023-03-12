use rejis::{
    filter::{And, Operator::Equal},
    Database, Query, QueryConstructor, Queryable, Table,
};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    first_name: String,
    last_name: String,
}

// Queryable implementation
impl<Root: Table> Queryable<Root> for User {
    type QueryType = UserQuery<Root>;
}

#[derive(Debug, Clone)]
struct UserQuery<Root: Table> {
    first_name: Query<String, Root>,
    last_name: Query<String, Root>,
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
        }
    }
}

// Table implementation
impl Table for User {
    const TABLE_NAME: &'static str = "user";
}

#[test]
fn insert_and_query() {
    let db = Database::new(Connection::open_in_memory().unwrap());

    db.create_table::<User>().unwrap();
    db.insert(User {
        first_name: String::from("John"),
        last_name: String::from("Smith"),
    });

    let john_smith = db
        .get::<User, _>(And(
            User::query().first_name.cmp(Equal, "John"),
            User::query().last_name.cmp(Equal, "Smith"),
        ))
        .unwrap();

    assert_eq!(john_smith.len(), 1);
    assert_eq!(john_smith[0].first_name, "John");
    assert_eq!(john_smith[0].last_name, "Smith");
}
