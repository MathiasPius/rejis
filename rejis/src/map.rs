use crate::{
    transform::{FromRow, Transform},
    Query, Queryable, Table,
};

/// Maps a field into a child field.
pub struct Select<Field, Root, Inner>
where
    Root: Table,
    Field: Queryable<Root>,
    Inner: Transform,
{
    pub(crate) inner: Inner,
    pub(crate) selector: Query<Field, Root>,
}

impl<Field, Root, Filter> Transform for Select<Field, Root, Filter>
where
    Field: Queryable<Root>,
    (Field,): FromRow,
    Root: Table,
    Filter: Transform,
{
    type Root = Root;
    type Field = Field;
    type Output = (Field,);

    fn bind(
        &self,
        statement: &mut rusqlite::Statement<'_>,
        index: &mut usize,
    ) -> Result<(), rusqlite::Error> {
        self.inner.bind(statement, index)
    }

    fn cte(&self, name: &str, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        self.inner.cte(name, f)
    }

    fn statement(&self) -> String {
        let query = self.selector.path();
        format!("select cast(json_extract(result.value, '{query}') as text) from result")
    }
}
