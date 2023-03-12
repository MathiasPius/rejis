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
