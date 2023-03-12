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

/// Functionality related to querying and selection. Implemented for [`Transform`]
/// allowing queries to be executed against a [`Connection`]
pub trait Executor {
    type Output;
    fn get(&self, conn: &Connection) -> Result<Vec<Self::Output>, TransformError>;
    fn delete(&self, conn: &Connection) -> Result<usize, TransformError>;
}

impl<T> Executor for T
where
    T: Transform,
{
    type Output = <<T as Transform>::Output as FromRow>::Output;

    /// Execute this transform against the connection, returning
    /// the selected object from the database.
    fn get(&self, conn: &Connection) -> Result<Vec<Self::Output>, TransformError> {
        let sql = sql_query_builder(
            <Self as Transform>::Root::TABLE_NAME,
            self,
            &self.statement(),
        )
        .unwrap();

        let mut stmt = conn.prepare(&sql)?;
        self.bind(&mut stmt, &mut 1)?;

        let mut objects = Vec::new();
        let mut rows = stmt.raw_query();
        while let Some(result) = rows.next()? {
            objects.push(self.extract(result).unwrap());
        }

        Ok(objects)
    }

    /// Delete all entries matching this transform on the connection.
    fn delete(&self, conn: &Connection) -> Result<usize, TransformError> {
        let table = <Self as Transform>::Root::TABLE_NAME;

        let sql = sql_query_builder(
            table,
            self,
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

        let mut stmt = conn.prepare(&sql)?;
        self.bind(&mut stmt, &mut 1)?;
        Ok(stmt.raw_execute()?)
    }
}

/// Combined implementation of [`Executor`] and [`Table`]
///
/// [`Table`] trait defines functions for table creation
/// and data insertion, and is implemented on the stored
/// structure type.
///
/// [`Executor`] is implemented on [`Transform`]s, since
/// it deals primarily with queries.
///
/// This trait is implemented for [`Connection`], combining
/// all the functionality of those traits (init, insertion
/// querying, deletion) into a single API.
pub trait Database {
    /// Initialize a table for `Root` on the database.
    fn init<Root: Table>(&self) -> Result<usize, rusqlite::Error>;

    /// Insert `value` into the database.
    ///
    /// Note: table must already exist. Table can be created using [`Database::init`]
    fn insert<Root: Table>(&self, value: &Root) -> Result<usize, TransformError>;

    /// Retrieve all items matching `transform`.
    fn get<Output>(
        &self,
        transform: &dyn Executor<Output = Output>,
    ) -> Result<Vec<Output>, TransformError>;

    /// Delete all items matching `filter`.
    fn delete<Output>(
        &self,
        filter: &dyn Executor<Output = Output>,
    ) -> Result<usize, TransformError>;
}

impl Database for Connection {
    fn init<Root: Table>(&self) -> Result<usize, rusqlite::Error> {
        Root::init(self)
    }

    fn insert<Root: Table>(&self, value: &Root) -> Result<usize, TransformError> {
        value.insert(self)
    }

    fn get<Output>(
        &self,
        transform: &dyn Executor<Output = Output>,
    ) -> Result<Vec<Output>, TransformError> {
        transform.get(self)
    }

    fn delete<Output>(
        &self,
        filter: &dyn Executor<Output = Output>,
    ) -> Result<usize, TransformError> {
        filter.delete(self)
    }
}
