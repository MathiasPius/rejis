use crate::{
    transform::{FromRow, Transform},
    Query, Queryable, Table,
};

pub struct Select<Field, Root, Inner>
where
    Root: Table,
    Field: Queryable<Root>,
    Inner: Transform,
{
    pub(crate) inner: Inner,
    pub(crate) selector: Query<Field, Root>,
}

impl<Root: Table, Inner: Transform> From<Inner> for Select<Root, Root, Inner> {
    fn from(value: Inner) -> Self {
        Select {
            inner: value,
            selector: Query::default(),
        }
    }
}

impl<Root: Table, Filter: Transform + Clone> From<&Filter> for Select<Root, Root, Filter> {
    fn from(value: &Filter) -> Self {
        Select {
            inner: value.clone(),
            selector: Query::default(),
        }
    }
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
