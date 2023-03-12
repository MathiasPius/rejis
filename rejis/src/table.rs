use rusqlite::Connection;
use serde::Serialize;

use crate::{transform::TransformError, Query, Queryable};

/// Describes how to store the type for which it is implemented
/// in an sqlite table.
pub trait Table: Queryable<Self> + Serialize + Sized + 'static {
    /// Name used for the table in the database when reading or writing
    /// this object to it.
    const TABLE_NAME: &'static str;

    fn query() -> Query<Self, Self> {
        Query::<Self, Self>::default()
    }

    fn init(conn: &Connection) -> Result<usize, rusqlite::Error> {
        conn.execute(
            &format!(
                "create table if not exists {table} (value text not null) strict;",
                table = Self::TABLE_NAME
            ),
            (),
        )
    }

    fn insert(&self, conn: &Connection) -> Result<usize, TransformError> {
        Ok(conn.execute(
            &format!(
                "insert into {table}(value) values(json(?1))",
                table = Self::TABLE_NAME
            ),
            (serde_json::to_string(self)?,),
        )?)
    }
}
