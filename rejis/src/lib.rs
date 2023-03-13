//! **rejis** aims to explore development of an API built on top of `rusqlite` which allows
//! simple querying and storage of `serde_json` serializable structures within an sqlite database.
//!
//! The purpose of this is to be able to use sqlite as a persistent store of data without spending
//! time up front translating between complex nested structures and sqlite tables.
//!
//! Rejis is *not* an ORM, as it completely disregards the *relational* benefits of a database, and
//! instead abuses sqlite as a sort of low-performance on-disk `Vec`, with some sql-aware abstractions
//! built on top of it, to reduce database roundtrips.
//!
//! The API is currently in flux and subject to change.
//!  
//! # Design
//! **rejis** uses `rusqlite` for interacting with a sqlite database.
//!
//! All top-level types are stored in a separate table as created by the
//! [`Table`](crate::table::Table) trait implementation for that type.
//! In most cases you will simply derive this trait automatically, and the
//! table will be a table with the same name as your type, but in lower case,
//! containing a single column named `value`.
//!
//! The `value` column contains the `serde_json` serialized contents of the
//! objects you store in the database.
//!
//! Querying uses composable [`Transforms`](crate::transform::Transform) based
//! on sequentially applied [CTEs](https://www.sqlite.org/lang_with.html),
//! which can then be executed against a [`rusqlite::Connection`](rusqlite::Connection).
//!
//! Producing these queries can be done in one of three ways:
//!
//! * Using derive macros for [`Queryable`](crate::query::Queryable) and [`Table`](crate::table::Table),
//!   and the [`Q!`](crate::macros::Q) macro DSL for building these queries.
//!   This is the fastest and simplest way.
//!
//!   For examples of how this is done, see [tests/macro_query.rs](rejis/tests/macro_query.rs)
//!
//! * Using the derive macros above, but building your query yourself using only
//!   Rust's type system.
//!
//!   For examples of how this is done, see [tests/filtering.rs](rejis/tests/filtering.rs)
//!
//! * By manually implementing [`Queryable`](crate::query::Queryable) and [`Table`](crate::table::Table), and constructing
//!   your queries using only Rust's type system. This is the most cumbersome, but
//!   also the least "magical" way of doing it, so if you're curious about how `rejis`
//!   works under the hood, this is a good place to start.
//!
//!   For examples of how this is done, see [tests/no_magic.rs](rejis/tests/no_magic.rs)
//!  
//! # Examples
//! You can use the [`Database`](crate::database::Database) trait for creating simple single-column tables for holding
//! objects of a certain type:
//!
//! **Setup**
//! ```rust
//! use rejis::{Database, Queryable, Table};
//! use serde::{Serialize, Deserialize};
//! use rusqlite::Connection;
//!
//! #[derive(Queryable, Table, Serialize, Deserialize, Debug, Clone)]
//! struct User {
//!   first_name: String,
//!   last_name: String,
//!   pets: Vec<String>,
//!   age: u8,
//! }
//!
//! let conn = Connection::open_in_memory().unwrap();
//! conn.init::<User>().unwrap();
//! ```
//! With the database connection set up, and a table `user` created, you
//! can start inserting objects:
//!
//! **Insertion**
//! ```rust
//! # use rejis::{Database, Queryable, Table};
//! # use serde::{Serialize, Deserialize};
//! # use rusqlite::Connection;
//! #
//! # #[derive(Queryable, Table, Serialize, Deserialize, Debug, Clone)]
//! # struct User {
//! #   first_name: String,
//! #   last_name: String,
//! #   pets: Vec<String>,
//! #   age: u8,
//! # }
//! #
//! # let conn = Connection::open_in_memory().unwrap();
//! # conn.init::<User>().unwrap();
//! #
//! // Add Jon Arbuckle with his pets to our database.
//! conn.insert(&User {
//!   first_name: String::from("Jon"),
//!   last_name: String::from("Arbuckle"),
//!   age: 29,
//!   pets: vec![
//!     String::from("Garfield"),
//!     String::from("Odie"),
//!   ],
//! }).unwrap();
//!
//! // Throw in Sabrina too, so we know our filters work later.
//! conn.insert(&User {
//!   first_name: String::from("Sabrina"),
//!   last_name: String::from("Spellman"),
//!   age: 15,
//!   pets: vec![
//!     String::from("Salem"),
//!   ],
//! }).unwrap();
//! ```
//! **Querying**
//! ```rust
//! # use rejis::{Database, Queryable, Table};
//! # use serde::{Serialize, Deserialize};
//! # use rusqlite::Connection;
//! #
//! # #[derive(Queryable, Table, Serialize, Deserialize, Debug, Clone)]
//! # struct User {
//! #   first_name: String,
//! #   last_name: String,
//! #   pets: Vec<String>,
//! #   age: u8,
//! # }
//! #
//! # let conn = Connection::open_in_memory().unwrap();
//! # conn.init::<User>().unwrap();
//! #
//! # conn.insert(&User {
//! #   first_name: String::from("Jon"),
//! #   last_name: String::from("Arbuckle"),
//! #   age: 29,
//! #   pets: vec![
//! #     String::from("Garfield"),
//! #     String::from("Odie"),
//! #   ],
//! # }).unwrap();
//! #
//! # conn.insert(&User {
//! #   first_name: String::from("Sabrina"),
//! #   last_name: String::from("Spellman"),
//! #   age: 15,
//! #   pets: vec![
//! #     String::from("Salem"),
//! #   ],
//! # }).unwrap();
//! // `Q` is a macro used for building queries in a custom
//! // domain-specific language which is designed to look
//! // like standard logical operations for readability.
//! use rejis::Q;
//!
//! // Find the pets of everyone in the 'Arbuckle' family
//! let arbuckles = conn.get(&Q!{
//!   User.last_name == "Arbuckle"
//! }).unwrap();
//!
//! // We know Jon is our only result.
//! assert_eq!(arbuckles.len(), 1);
//! let jon = arbuckles.first().unwrap();
//! assert_eq!(jon.pets, vec!["Garfield", "Odie"]);
//! ```
//! This is pretty straight-forward, but what if our `User` object is actually massive?
//!
//! In that case, it'd be preferable to be able to *only* fetch the `pets` field!
//!
//! **Mapping**
//! ```rust
//! # use rejis::{Database, Queryable, Table};
//! # use serde::{Serialize, Deserialize};
//! # use rusqlite::Connection;
//! #
//! # #[derive(Queryable, Table, Serialize, Deserialize, Debug, Clone)]
//! # struct User {
//! #   first_name: String,
//! #   last_name: String,
//! #   pets: Vec<String>,
//! #   age: u8,
//! # }
//! #
//! # let conn = Connection::open_in_memory().unwrap();
//! # conn.init::<User>().unwrap();
//! #
//! # conn.insert(&User {
//! #   first_name: String::from("Jon"),
//! #   last_name: String::from("Arbuckle"),
//! #   age: 29,
//! #   pets: vec![
//! #     String::from("Garfield"),
//! #     String::from("Odie"),
//! #   ],
//! # }).unwrap();
//! #
//! # conn.insert(&User {
//! #   first_name: String::from("Sabrina"),
//! #   last_name: String::from("Spellman"),
//! #   age: 15,
//! #   pets: vec![
//! #     String::from("Salem"),
//! #   ],
//! # }).unwrap();
//! # use rejis::Q;
//! // Transform trait implements the `.map(..)` function
//! use crate::rejis::transform::Transform;
//!
//! // Find all the 'Arbuckle' families, and map the
//! // resulting users to their own `pets` member.
//! let arbuckle_pets = conn.get(
//!   &Q!{
//!     User.last_name == "Arbuckle"
//!   }.map(&User::query().pets)
//! ).unwrap();
//!
//! // arbuckle_pets is a Vec<Vec<String>> since we're querying a
//! // nested `pets: Vec<String>` from each `User`, and each user
//! // is returned as its own result. Since `Jon` is the only
//! // Arbuckle, we can just get his pets from the first result
//! let jons_pets = arbuckle_pets.first().unwrap();
//! assert_eq!(jons_pets.as_ref(), vec!["Garfield", "Odie"]);
//! ```
//! This way we avoid having to extract and deserialize the entire
//! Jon Arbuckle `User`, instead fetching only the data we care about.
//!
//! Satisfied with the list of pets, and wanting nothing more to do with
//! Jon Arbuckle, we can move on to..
//!
//! **Deleting**
//! ```rust
//! # use rejis::{Database, Queryable, Table};
//! # use serde::{Serialize, Deserialize};
//! # use rusqlite::Connection;
//! #
//! # #[derive(Queryable, Table, Serialize, Deserialize, Debug, Clone)]
//! # struct User {
//! #   first_name: String,
//! #   last_name: String,
//! #   pets: Vec<String>,
//! #   age: u8,
//! # }
//! #
//! # let conn = Connection::open_in_memory().unwrap();
//! # conn.init::<User>().unwrap();
//! #
//! # conn.insert(&User {
//! #   first_name: String::from("Jon"),
//! #   last_name: String::from("Arbuckle"),
//! #   age: 29,
//! #   pets: vec![
//! #     String::from("Garfield"),
//! #     String::from("Odie"),
//! #   ],
//! # }).unwrap();
//! #
//! # use rejis::Q;
//! // Find all the 'Arbuckle' users...
//! // And promptly delete them!
//! let deleted_rows = conn.delete(&Q!{
//!     User.last_name == "Arbuckle"
//! }).unwrap();
//!
//! assert_eq!(deleted_rows, 1);
//! ```
//!
//! # Roadmap
//! * **Updating/replacing entire entries**
//!   ```rust,ignore
//!   // Replace John Smith with Jane Doe
//!   Q! {
//!     (User.first_name == "John") && (User.last_name == "Smith")
//!   }.replace(&conn, User {
//!     first_name: "Jane".to_string(),
//!     last_name: "Doe".to_string(),
//!   })
//!   ```
//!
//! ## Tentative features

//! * **Partial updates**
//!
//!   Using a filter for selection, a query for targeting, and a closure for manipulation,
//!   a nice api could be made for highly selective updates
//!   ```rust,ignore
//!   // Uppercase all the last names of people called Jon
//!   db.modify(
//!     // Select specific rows
//!     &Q! { User.first_name == "Jon" },
//!     // Target the last_name specifically for replacement
//!     &Q! { User.last_name },
//!     // Provide a function for how the name should be transformed.
//!     |last_name| last_name.to_uppercase()
//!   );
//!   ```
//!   Query mapping might be very relevant for the final form of this API.
//!
//! * **Two-stage Query application**
//!
//!   The current API requires all parameters to be known at construction time, even though
//!   these values are only fed into the SQL prepared statement when it is actually applied.
//!   This is because the values for comparisons for example are stored within the `Comparison`
//!   struct itself.
//!
//!   Being able to construct an entire query once and then re-using could bring better performance.
//!   Since the `rusqlite` prepared statement API borrows the connection object, it might not be
//!   possible to reuse the entire statement as-is, but sqlite itself supports caching prepared
//!   statements, presumeably based on the input sql, so if the sql statement itself can be constructed
//!   once, and then reused, it might still have an impact.
//!
//!
//! * **Expression indices**
//!   
//!   Sqlite supports creating an index over an expression within a table,
//!   which might be useful for improving the performance of commonly used queries.
//!   If you use `rejis` for storing users for example, it might be useful to be able to create
//!   an index over `json_extract(value, '$.id')` or `$.name`, if those are used to find users.
//!
//! # Shortcomings
//! * Query paths only allow a single indexing element.
//!   Reason for this is in the complexity of implementing the SQL CTE and the Q!-macro DSL support for that use case.
//!
pub mod filter;
mod map;
pub mod transform;

mod table;
pub use table::Table;

mod database;
pub use database::Database;

#[cfg(feature = "derive")]
pub use rejis_derive::{Queryable, Table};

mod query;
pub use query::*;

#[cfg(feature = "macros")]
mod macros;
#[cfg(feature = "macros")]
pub use macros::*;
