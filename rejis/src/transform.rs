use rusqlite::{Row, Statement};
use serde::de::DeserializeOwned;

use crate::{map::Select, Query, Queryable, Table};

#[derive(thiserror::Error, Debug)]
pub enum TransformError {
    #[error("failure while reading value from mapped row")]
    Sql(#[from] rusqlite::Error),
    #[error("failure while deserializing mapped value")]
    Serde(#[from] serde_json::Error),
}

pub trait Transform {
    type Root: Table;
    type Field: Queryable<Self::Root>;
    type Output: FromRow;

    fn bind(&self, statement: &mut Statement<'_>, index: &mut usize)
        -> Result<(), rusqlite::Error>;
    fn cte(&self, name: &str, f: &mut impl std::fmt::Write) -> std::fmt::Result;

    fn statement(&self) -> String {
        String::from("select result.value from result")
    }

    fn extract(&self, row: &Row) -> Result<<Self::Output as FromRow>::Output, TransformError> {
        Self::Output::from_row(row)
    }

    fn map<Subfield: Queryable<Self::Root>>(
        self,
        query: &Query<Subfield, Self::Root>,
    ) -> Select<Subfield, Self::Root, Self>
    where
        Self: Sized,
    {
        Select {
            selector: query.clone(),
            inner: self,
        }
    }
}

pub trait FromRow: Sized {
    type Output;
    fn from_row(row: &Row) -> Result<Self::Output, TransformError>;
}

impl<A> FromRow for (A,)
where
    A: DeserializeOwned,
{
    type Output = A;
    fn from_row(row: &Row) -> Result<Self::Output, TransformError> {
        let value: String = row.get(0)?;
        Ok(serde_json::from_str(&value)?)
    }
}
