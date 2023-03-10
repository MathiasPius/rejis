use rusqlite::{Statement, ToSql};

use crate::query::{Query, QueryConstructor, Queryable, Table};

/// Structure capable of producing a valid sql where-clause,
/// and binding parameters to it.
///
/// See [`Comparison`] which generates singular `A = B` clauses
/// or [`And`] which itself composes multiple such statements.
pub trait Filter<Root> {
    fn sql_string(&self) -> String;
    fn bind_parameters(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error>;
}

/// [`Comparison`] operator.
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
}

impl AsRef<str> for Operator {
    fn as_ref(&self) -> &'static str {
        match self {
            Operator::Equal => "=",
            Operator::NotEqual => "!=",
            Operator::GreaterThan => ">",
            Operator::GreaterThanOrEqual => ">=",
            Operator::LessThan => "<",
            Operator::LessThanOrEqual => "<=",
            Operator::Like => "like",
        }
    }
}

/// Describes comparison between the `Query` path of a `Root` object, given
/// a comparable value and operator.
pub struct Comparison<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
    <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql,
{
    pub(crate) query: Query<Field, Root>,
    pub(crate) operator: Operator,
    pub(crate) value: <Field::QueryType as QueryConstructor<Root>>::Inner,
}

impl<Field, Root> Filter<Root> for Comparison<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
    <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql,
{
    fn sql_string(&self) -> String {
        let operator = self.operator.as_ref();
        format!("json_extract(value, ?) {operator} ?")
    }

    fn bind_parameters(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        statement.raw_bind_parameter(*index, self.query.path().to_string())?;
        *index += 1;
        statement.raw_bind_parameter(*index, &self.value)?;
        *index += 1;

        Ok(())
    }
}

pub struct And<T>(pub T);

impl<Root: Table, A, B> Filter<Root> for And<(A, B)>
where
    A: Filter<Root>,
    B: Filter<Root>,
{
    fn sql_string(&self) -> String {
        let f1 = self.0 .0.sql_string();
        let f2 = self.0 .1.sql_string();
        format!("({f1} and {f2})")
    }

    fn bind_parameters(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        self.0 .0.bind_parameters(statement, index)?;
        self.0 .1.bind_parameters(statement, index)?;
        Ok(())
    }
}
