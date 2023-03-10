use rusqlite::Connection;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    filter::Filter,
    query::{Queryable, Table},
};

/// Wrapper around a [`Connection`] which enables interaction
/// with the underlying database using the `rejis` system.
pub struct Database(pub(crate) Connection);

impl Database {
    /// Consume an existing [`Connection`] and wrap it in a `rejis` database accessor.
    pub fn new(conn: Connection) -> Self {
        Database(conn)
    }

    /// Construct a table for holding `Root` objects.
    pub fn create_table<Root: Table>(&self) -> Result<usize, rusqlite::Error> {
        self.0.execute(
            &format!(
                "create table if not exists {table} (value text not null) strict;",
                table = Root::TABLE_NAME
            ),
            (),
        )
    }

    /// Insert an object of type `Root`. Fails if the table does not exist.
    ///
    /// Run [`Database::create_table`] before hand.
    pub fn insert<Root: Table + Queryable<Root> + Serialize>(&self, value: Root) {
        self.0
            .execute(
                &format!(
                    "insert into {table}(value) values(json(?1))",
                    table = Root::TABLE_NAME
                ),
                (serde_json::to_string(&value).unwrap(),),
            )
            .unwrap();
    }

    /// Fetch `Root` object(s) using the given filter
    pub fn get<Root>(&self, filter: impl Filter<Root>) -> Result<Vec<Root>, rusqlite::Error>
    where
        Root: Table + DeserializeOwned,
    {
        let sql = format!(
            "select value from {table} where {filters}",
            table = Root::TABLE_NAME,
            filters = filter.sql_string()
        );

        let mut stmt = self.0.prepare(&sql).unwrap();
        filter.bind_parameters(&mut stmt, &mut 1)?;

        let mut objects = Vec::new();
        let mut rows = stmt.raw_query();
        while let Some(result) = rows.next()? {
            let value: String = result.get(0)?;
            let result: Root = serde_json::from_str(&value).unwrap();

            objects.push(result);
        }

        Ok(objects)
    }
}
