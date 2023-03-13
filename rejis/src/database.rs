use rusqlite::Connection;
use std::fmt::Write;

use crate::{
    transform::{FromRow, Transform, TransformError},
    Table,
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

/// Simple user-friendly API for inserting, querying and deleting structures
/// which implement both [`Queryable`](::rejis::Queryable) and [`Table`].
pub trait Database {
    /// Initialize a table for `Root` on the database.
    fn init<Root: Table>(&self) -> Result<usize, rusqlite::Error>;

    /// Insert `value` into the database.
    ///
    /// Note: table must already exist. Table can be created using [`Database::init`]
    fn insert<Root: Table>(&self, value: &Root) -> Result<usize, TransformError>;

    /// Retrieve all items matching `transform`.
    fn get<T: Transform>(
        &self,
        transform: &T,
    ) -> Result<Vec<<<T as Transform>::Output as FromRow>::Output>, TransformError>;

    /// Delete all items matching `filter`.
    fn delete<T: Transform>(&self, filter: &T) -> Result<usize, TransformError>;
}

impl Database for Connection {
    fn init<Root: Table>(&self) -> Result<usize, rusqlite::Error> {
        Ok(self.execute(
            &format!(
                "create table if not exists {table} (value text not null) strict;",
                table = Root::TABLE_NAME
            ),
            (),
        )?)
    }

    fn insert<Root: Table>(&self, value: &Root) -> Result<usize, TransformError> {
        Ok(self.execute(
            &format!(
                "insert into {table}(value) values(json(?1))",
                table = Root::TABLE_NAME
            ),
            (serde_json::to_string(value)?,),
        )?)
    }

    fn get<T: Transform>(
        &self,
        transform: &T,
    ) -> Result<Vec<<<T as Transform>::Output as FromRow>::Output>, TransformError> {
        let sql = sql_query_builder(
            <T as Transform>::Root::TABLE_NAME,
            transform,
            &transform.statement(),
        )
        .unwrap();

        let mut stmt = self.prepare(&sql)?;
        transform.bind(&mut stmt, &mut 1)?;

        let mut objects = Vec::new();
        let mut rows = stmt.raw_query();
        while let Some(result) = rows.next()? {
            objects.push(transform.extract(result).unwrap());
        }

        Ok(objects)
    }

    fn delete<T: Transform>(&self, filter: &T) -> Result<usize, TransformError> {
        let table = <T as Transform>::Root::TABLE_NAME;

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

        let mut stmt = self.prepare(&sql)?;
        filter.bind(&mut stmt, &mut 1)?;
        Ok(stmt.raw_execute()?)
    }
}
