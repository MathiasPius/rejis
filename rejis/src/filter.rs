//! Structures used for applying filters to queries.
use crate::{
    transform::{FromRow, Transform},
    Query, QueryConstructor, Queryable, Table,
};
use rusqlite::{Statement, ToSql};
use std::fmt::{Debug, Display, Formatter};

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

impl<Field: Clone, Root: Clone> Clone for Comparison<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
    <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql + Clone,
{
    fn clone(&self) -> Self {
        Self {
            query: self.query.clone(),
            operator: self.operator,
            value: self.value.clone(),
        }
    }
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

impl<Field, Root> Transform for Comparison<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
    (Root,): FromRow,
    <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql,
{
    type Root = Root;
    type Field = Field;
    type Output = (Root,);

    fn bind(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        statement.raw_bind_parameter(*index, &self.value)?;
        *index += 1;
        Ok(())
    }

    fn cte(&self, name: &str, f: &mut impl std::fmt::Write) -> std::fmt::Result {
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

    fn statement(&self) -> String {
        String::from("select result.value from result")
    }
}

#[derive(Clone)]
pub struct And<A, B>(pub A, pub B);

impl<A, B> Debug for And<A, B>
where
    A: Debug,
    B: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("And").field(&self.0).field(&self.1).finish()
    }
}

impl<Root, Field, A, B> Transform for And<A, B>
where
    Root: Table,
    (Root,): FromRow,
    Field: Queryable<Root>,
    A: Transform<Root = Root, Field = Field>,
    B: Transform<Root = Root, Field = Field>,
{
    type Root = Root;
    type Field = Field;
    type Output = (Root,);

    fn bind(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        self.0.bind(statement, index)?;
        self.1.bind(statement, index)?;
        Ok(())
    }

    fn cte(&self, name: &str, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.0.cte(&format!("{name}_a"), f)?;
        self.1.cte(&format!("{name}_b"), f)?;

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

#[derive(Clone)]
pub struct Or<A, B>(pub A, pub B);

impl<A, B> Debug for Or<A, B>
where
    A: Debug,
    B: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Or").field(&self.0).field(&self.1).finish()
    }
}

impl<Root, Field, A, B> Transform for Or<A, B>
where
    Root: Table,
    (Root,): FromRow,
    Field: Queryable<Root>,
    A: Transform<Root = Root, Field = Field>,
    B: Transform<Root = Root, Field = Field>,
{
    type Root = Root;
    type Field = Field;
    type Output = (Root,);

    fn bind(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        self.0.bind(statement, index)?;
        self.1.bind(statement, index)?;
        Ok(())
    }

    fn cte(&self, name: &str, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.0.cte(&format!("{name}_a"), f)?;
        self.1.cte(&format!("{name}_b"), f)?;

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
pub struct Any<Field, InnerField, Root>
where
    Field: Queryable<Root>,
    InnerField: Queryable<Root>,
    Root: Table,
    <InnerField::QueryType as QueryConstructor<Root>>::Inner: ToSql,
{
    pub(crate) outer_query: Query<Field, Root>,
    pub(crate) inner_query: Query<InnerField, Root>,
    pub(crate) operator: Operator,
    pub(crate) value: <InnerField::QueryType as QueryConstructor<Root>>::Inner,
}

impl<Field: Clone, InnerField: Clone, Root: Clone> Clone for Any<Field, InnerField, Root>
where
    Field: Queryable<Root>,
    InnerField: Queryable<Root>,
    Root: Table,
    <InnerField::QueryType as QueryConstructor<Root>>::Inner: ToSql + Clone,
{
    fn clone(&self) -> Self {
        Self {
            outer_query: self.outer_query.clone(),
            inner_query: self.inner_query.clone(),
            operator: self.operator,
            value: self.value.clone(),
        }
    }
}

impl<Field: Debug, InnerField: Debug, Root: Debug> Debug for Any<Field, InnerField, Root>
where
    Field: Queryable<Root>,
    InnerField: Queryable<Root>,
    Root: Table,
    <InnerField::QueryType as QueryConstructor<Root>>::Inner: ToSql + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Any")
            .field("outer_query", &self.outer_query)
            .field("inner_query", &self.inner_query)
            .field("operator", &self.operator)
            .field("value", &self.value as &dyn Debug)
            .finish()
    }
}

impl<Field, InnerField, Root> Transform for Any<Field, InnerField, Root>
where
    Field: Queryable<Root>,
    InnerField: Queryable<Root>,
    Root: Table,
    (Root,): FromRow,
    <InnerField::QueryType as QueryConstructor<Root>>::Inner: ToSql,
{
    type Root = Root;
    type Field = InnerField;
    type Output = (Root,);

    fn bind(
        &self,
        statement: &mut Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        statement.raw_bind_parameter(*index, &self.value)?;
        *index += 1;

        Ok(())
    }

    fn cte(&self, name: &str, f: &mut impl std::fmt::Write) -> std::fmt::Result {
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
