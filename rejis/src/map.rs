use rusqlite::Row;
use serde::de::DeserializeOwned;

use crate::{filter::Filter, Query, Queryable, Table};

#[derive(thiserror::Error, Debug)]
pub enum MapError {
    #[error("failure while reading value from mapped row")]
    Sql(#[from] rusqlite::Error),
    #[error("failure while deserializing mapped value")]
    Serde(#[from] serde_json::Error),
}

pub struct Select<Field, Root, F>
where
    Root: Table,
    Field: Queryable<Root>,
    F: Filter<Root>,
{
    pub(crate) filter: F,
    pub(crate) selector: Query<Field, Root>,
}

impl<Root: Table, F: Filter<Root>> From<F> for Select<Root, Root, F> {
    fn from(value: F) -> Self {
        Select {
            filter: value,
            selector: Query::default(),
        }
    }
}

impl<Root: Table, F: Filter<Root> + Clone> From<&F> for Select<Root, Root, F> {
    fn from(value: &F) -> Self {
        Select {
            filter: value.clone(),
            selector: Query::default(),
        }
    }
}

pub trait Map<Field: Queryable<Root>, Root: Table> {
    fn selector(&self) -> String;
    fn extract(&self, row: &Row) -> Result<Field, MapError>;
}

impl<Field, Root, F> Map<Field, Root> for Select<Field, Root, F>
where
    Root: Table,
    Field: Queryable<Root> + DeserializeOwned,
    F: Filter<Root>,
{
    fn selector(&self) -> String {
        let query = self.selector.path();
        format!("select cast(json_extract(result.value, '{query}') as text) from result")
    }

    fn extract(&self, row: &Row) -> Result<Field, MapError> {
        let value: String = row.get(0)?;
        let deserialized: Field = serde_json::from_str(&value)?;

        Ok(deserialized)
    }
}

impl<Field, Root, F> Filter<Root> for Select<Field, Root, F>
where
    Root: Table,
    Field: Queryable<Root>,
    F: Filter<Root>,
{
    fn bind_parameters(
        &self,
        statement: &mut rusqlite::Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        self.filter.bind_parameters(statement, index)
    }

    fn statement(&self, name: &str, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.filter.statement(name, f)
    }
}
