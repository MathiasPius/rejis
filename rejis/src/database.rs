use rusqlite::Connection;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Write;

use crate::{
    query::{Queryable, Table},
    transform::{FromRow, Transform},
};

fn sql_query_builder(
    table_name: &str,
    filter: &impl Transform,
    selector: &str,
) -> Result<String, std::fmt::Error> {
    let mut sql = format!(
        "
with
    root as (
        select rowid, value
        from {table_name}
    )",
    );

    filter.cte("result", &mut sql)?;

    write!(&mut sql, "\n{selector}")?;

    Ok(sql)
}

/// Wrapper around a [`Connection`] which enables interaction
/// with the underlying database using the `rejis` system.
pub struct Database(pub Connection);

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
    pub fn get<Root, Transformer>(
        &self,
        query: Transformer,
    ) -> Result<Vec<<<Transformer as Transform>::Output as FromRow>::Output>, rusqlite::Error>
    where
        Root: Table + DeserializeOwned,
        Transformer: Transform,
    {
        let sql = sql_query_builder(Root::TABLE_NAME, &query, &query.statement()).unwrap();

        let mut stmt = self.0.prepare(&sql)?;
        query.bind(&mut stmt, &mut 1)?;

        let mut objects = Vec::new();
        let mut rows = stmt.raw_query();
        while let Some(result) = rows.next()? {
            objects.push(query.extract(result).unwrap());
        }

        Ok(objects)
    }

    /// Delete `Root` object(s) using the given filter
    pub fn delete<Root>(&self, filter: &impl Transform) -> Result<Vec<Root>, rusqlite::Error>
    where
        Root: Table + DeserializeOwned,
    {
        let table = Root::TABLE_NAME;

        let sql = sql_query_builder(
            table,
            filter,
            &format!(
                "
delete from {table}
where exists (
    select * from result
    where result.rowid = {table}.rowid
)"
            ),
        )
        .unwrap();

        let mut stmt = self.0.prepare(&sql)?;
        filter.bind(&mut stmt, &mut 1)?;

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
