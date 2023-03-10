use std::fmt::{Debug, Display, Formatter};

use rusqlite::{Statement, ToSql};

use crate::query::{Query, QueryConstructor, Queryable, Table};

/// Structure capable of producing a valid sql where-clause,
/// and binding parameters to it.
///
/// See [`Comparison`] which generates singular `A = B` clauses
/// or [`And`] which itself composes multiple such statements.
pub trait Filter<Root>: Display {
    fn bind_parameters(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error>;
}

/// [`Comparison`] operator.
#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Like,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Operator::Equal => "=",
            Operator::NotEqual => "!=",
            Operator::GreaterThan => ">",
            Operator::GreaterThanOrEqual => ">=",
            Operator::LessThan => "<",
            Operator::LessThanOrEqual => "<=",
            Operator::Like => "like",
        })
    }
}

/// Describes comparison between the `Query` path of a `Root` object, given
/// a comparable value and operator.
#[derive(Debug)]
pub struct Comparison<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
    <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql + Debug,
{
    pub(crate) query: Query<Field, Root>,
    pub(crate) operator: Operator,
    pub(crate) value: <Field::QueryType as QueryConstructor<Root>>::Inner,
}

impl<Field, Root> Filter<Root> for Comparison<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
    <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql + Debug,
{
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

impl<Field, Root> Display for Comparison<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
    <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(json_extract(value, ?) {operator} ?)",
            operator = &self.operator
        )
    }
}

#[derive(Debug)]
pub struct And<F>(pub F);

impl<Root: Table, A, B> Filter<Root> for And<(A, B)>
where
    A: Filter<Root>,
    B: Filter<Root>,
{
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

impl<A, B> Display for And<(A, B)>
where
    A: Display,
    B: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({f1} and {f2})", f1 = self.0 .0, f2 = self.0 .1)
    }
}
