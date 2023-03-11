//! Structures used for applying filters to queries.
use crate::query::{Query, QueryConstructor, Queryable, Table};
use rusqlite::{Statement, ToSql};
use std::fmt::{Debug, Display, Formatter, Write};

/// Structure capable of producing a valid sql where-clause,
/// and binding parameters to it.
///
/// See [`Comparison`] which generates singular `A = B` clauses
/// or [`And`] which itself composes multiple such statements.
pub trait Filter<Root: Table> {
    fn bind_parameters(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error>;

    fn statement(&self, name: &str, f: &mut impl Write) -> std::fmt::Result;
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

impl<Field: Debug, Root: Debug> Debug for Comparison<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
    <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Comparison")
            .field("query", &self.query)
            .field("operator", &self.operator)
            .field("value", &self.value as &dyn Debug)
            .finish()
    }
}

impl<Field, Root> Filter<Root> for Comparison<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
    <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql,
{
    fn bind_parameters(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        statement.raw_bind_parameter(*index, &self.value)?;
        *index += 1;

        Ok(())
    }

    fn statement(&self, name: &str, f: &mut impl Write) -> std::fmt::Result {
        write!(
            f,
            ",\n    {name} as (
        select root.rowid, root.value
        from root
        where json_extract(root.value, '{path}') {operator} ?
    )",
            path = self.query.path(),
            operator = self.operator
        )
    }
}

pub struct And<F>(pub F);

impl<F: Debug> Debug for And<F>
where
    F: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("And").field(&self.0).finish()
    }
}

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

    fn statement(&self, name: &str, f: &mut impl Write) -> std::fmt::Result {
        self.0 .0.statement(&format!("{name}_a"), f)?;
        self.0 .1.statement(&format!("{name}_b"), f)?;

        write!(
            f,
            ",\n    {name} as (
        select {name}_a.rowid, {name}_a.value
        from {name}_a
        inner join {name}_b
        on {name}_a.rowid = {name}_b.rowid
    )"
        )
    }
}

pub struct Or<F>(pub F);

impl<F: Debug> Debug for Or<F>
where
    F: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Or").field(&self.0).finish()
    }
}

impl<Root: Table, A, B> Filter<Root> for Or<(A, B)>
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

    fn statement(&self, name: &str, f: &mut impl Write) -> std::fmt::Result {
        self.0 .0.statement(&format!("{name}_a"), f)?;
        self.0 .1.statement(&format!("{name}_b"), f)?;

        write!(
            f,
            ",\n    {name} as (
        select * from {name}_a
        union all
        select * from {name}_b
    )"
        )
    }
}

/// Describes comparison between the `Query` path of a `Root` object, given
/// a comparable value and operator.
#[derive(Debug)]
pub struct Any<Field, InnerField, Root>
where
    Field: Queryable<Root>,
    InnerField: Queryable<Root>,
    Root: Table,
    <InnerField::QueryType as QueryConstructor<Root>>::Inner: ToSql + Debug,
{
    pub(crate) outer_query: Query<Field, Root>,
    pub(crate) inner_query: Query<InnerField, Root>,
    pub(crate) operator: Operator,
    pub(crate) value: <InnerField::QueryType as QueryConstructor<Root>>::Inner,
}

impl<Field, InnerField, Root> Filter<Root> for Any<Field, InnerField, Root>
where
    Field: Queryable<Root>,
    InnerField: Queryable<Root>,
    Root: Table,
    <InnerField::QueryType as QueryConstructor<Root>>::Inner: ToSql + Debug,
{
    fn bind_parameters(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        statement.raw_bind_parameter(*index, &self.value)?;
        *index += 1;

        Ok(())
    }

    fn statement(&self, name: &str, f: &mut impl Write) -> std::fmt::Result {
        write!(
            f,
            ",\n    {name} as (
        select distinct rowid, value from (
            select root.rowid, root.value
            from root, json_each(root.value, '{outer_path}')
            where json_extract(json_each.value, '{inner_path}') {operator} ?
        )
    )",
            outer_path = self.outer_query.path(),
            inner_path = self.inner_query.path(),
            operator = self.operator
        )
    }
}
