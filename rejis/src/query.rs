//!
use rusqlite::ToSql;
use serde::{de::DeserializeOwned, Serialize};

pub mod path {
    use std::fmt::Display;

    #[derive(Debug, Clone)]
    pub enum PathElement {
        Field(&'static str),
        Index(usize),
    }

    impl Display for PathElement {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                PathElement::Field(field) => {
                    write!(f, ".{field}")
                }
                PathElement::Index(index) => {
                    write!(f, "[{index}]")
                }
            }
        }
    }

    impl From<usize> for PathElement {
        fn from(value: usize) -> Self {
            PathElement::Index(value)
        }
    }

    impl From<&'static str> for PathElement {
        fn from(value: &'static str) -> Self {
            PathElement::Field(value)
        }
    }

    #[derive(Default, Debug, Clone)]
    pub struct Path(Vec<PathElement>);

    impl Path {
        pub fn join<P: Into<PathElement>>(&self, element: P) -> Self {
            let mut path = self.0.clone();
            path.push(element.into());
            Path(path)
        }
    }

    impl Display for Path {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("$")?;
            for field in &self.0 {
                write!(f, "{field}")?;
            }

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::Path;

        #[test]
        fn build_path() {
            let path = Path::default().join("pets").join(0).join("name");

            assert_eq!(path.to_string(), "$.pets[0].name");
        }
    }
}

use crate::{
    filter::{Any, Comparison, Operator},
    Table,
};
use std::{fmt::Debug, marker::PhantomData, ops::Deref};

pub use self::path::Path;

/// Indicates the `QueryConstructor` struct which describes the json structure
/// of the object.
///
/// You will want to implement this trait for any object which you want to
/// traverse using queries and filters. the `QueryType` object acts as a kind
/// of *reflection* of the real object, where each member is a `Query` object
/// describing said member.
///
/// Can (and should) be automatically derived when possible.
pub trait Queryable<Root>: Serialize + DeserializeOwned + Clone + 'static
where
    Root: Table,
{
    type QueryType: QueryConstructor<Root> + Clone;
}

/// Typed construction of JSON Path based on `Queryable` structs.
///
/// [`Deref`]s into its `<Field as Queryable>::QueryType` struct,
/// so it can be used to move deeper into the json structure in a type-safe
/// way.
///
/// Maintains a reference to the `Root` [`Table`] type, so receiving database
/// functions can deduce which table's structure the query maps to.
#[derive(Clone)]
pub struct Query<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
{
    /// Json path of the current query
    path: Path,
    /// Inner query type used for further descending into the structure
    subquery: <Field as Queryable<Root>>::QueryType,
    _data: PhantomData<(Field, Root)>,
}

impl<Field: Debug, Root: Debug> Debug for Query<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Query").field("path", &self.path).finish()
    }
}

impl<Field> Default for Query<Field, Field>
where
    Field: Queryable<Field> + Table,
{
    /// Construct an empty root-level query of type [`Table`]
    fn default() -> Self {
        let path = Path::default();

        Query::<Field, Field> {
            subquery: Field::QueryType::new::<Field>(&path),
            path,
            _data: PhantomData::default(),
        }
    }
}

impl<Field, Root> Query<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
{
    /// Construct a new `Query` with the given path.
    pub fn new(path: Path) -> Self {
        Query {
            subquery: Field::QueryType::new::<Field>(&path),
            path,
            _data: PhantomData,
        }
    }

    /// Compare value at query's path using the given `operator` and `value`.
    pub fn cmp<Value: Into<<Field::QueryType as QueryConstructor<Root>>::Inner>>(
        &self,
        operator: Operator,
        value: Value,
    ) -> Comparison<Field, Root>
    where
        <Field::QueryType as QueryConstructor<Root>>::Inner: ToSql,
    {
        Comparison {
            query: self.clone(),
            operator,
            value: value.into(),
        }
    }

    /// Construct a dot-separated json-path from this query.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<Field, Root> Query<Vec<Field>, Root>
where
    Field: Queryable<Root>,
    Vec<Field>: Queryable<Root>,
    Root: Table,
{
    pub fn any<
        InnerField: Queryable<Root>,
        Value: Into<<InnerField::QueryType as QueryConstructor<Root>>::Inner>,
        F: FnOnce(Query<Field, Root>) -> Query<InnerField, Root>,
    >(
        &self,
        f: F,
        operator: Operator,
        value: Value,
    ) -> Any<Vec<Field>, InnerField, Root>
    where
        <InnerField::QueryType as QueryConstructor<Root>>::Inner: ToSql,
    {
        let indexed = VecField::<Field, Root>::new::<Field>(&Path::default()).wildcard();

        Any {
            outer_query: self.clone(),
            inner_query: f(indexed),
            operator,
            value: value.into(),
        }
    }
}

/// This is a hack, allowing us to step right through the `Query` abstraction
/// into the sub-field `QueryConstructor`.
///
/// This allows us to keep metadata about the query around through
/// the [`Query`] struct, without having to write boilerplate in all
/// the implementations of `QueryConstructor`.
///
/// It is also necessary in order to be able to write functions such as
/// `Database::get`, which take the generic `Query` object as a parameter.
impl<Field, Root> Deref for Query<Field, Root>
where
    Field: Queryable<Root>,
    Root: Table,
{
    type Target = <Field as Queryable<Root>>::QueryType;

    fn deref(&self) -> &Self::Target {
        &self.subquery
    }
}

/// Exposes a constructor for `QueryType`s when building sub-queries.
///
/// Implemented by the a `Queryable` type's `QueryType`
pub trait QueryConstructor<Root>
where
    Root: Table,
{
    type Inner: Serialize + DeserializeOwned;

    /// Given the parent path `path` and a `Queryable` type `Field`, construct
    /// an instance of the `Field`'s `Queryable` type.
    fn new<Field>(path: &Path) -> Self
    where
        Field: Queryable<Root>;
}

macro_rules! unit_field_impl {
    ($inner:ident, $field_type: ident) => {
        #[doc = concat!("Implementation of [`Queryable`] for `", stringify!($inner), "`")]
        #[derive(Clone)]
        pub struct $field_type;

        impl<Root> Queryable<Root> for $inner
        where
            Root: Table,
        {
            type QueryType = $field_type;
        }

        impl<Root> QueryConstructor<Root> for $field_type
        where
            Root: Table,
        {
            type Inner = $inner;

            fn new<Field: Queryable<Root>>(_parent: &Path) -> Self {
                $field_type
            }
        }
    };
}

unit_field_impl!(String, StringQuery);
unit_field_impl!(u8, U8Query);
unit_field_impl!(u16, U16Query);
unit_field_impl!(u32, U32Query);
unit_field_impl!(u64, U64Query);
unit_field_impl!(u128, U128Query);
unit_field_impl!(usize, UsizeQuery);
unit_field_impl!(i8, I8Query);
unit_field_impl!(i16, I16Query);
unit_field_impl!(i32, I32Query);
unit_field_impl!(i64, I64Query);
unit_field_impl!(i128, I128Query);
unit_field_impl!(isize, IsizeQuery);
unit_field_impl!(bool, BoolQuery);

/// A struct field of type [`Vec`]
#[derive(Debug, Clone)]
pub struct VecField<Field, Root>(Query<Field, Root>)
where
    Field: Queryable<Root>,
    Root: Table;

impl<Field, Root> Queryable<Root> for Vec<Field>
where
    Field: Queryable<Root> + Clone,
    Root: Table + Clone,
{
    type QueryType = VecField<Field, Root>;
}

impl<T, Root> QueryConstructor<Root> for VecField<T, Root>
where
    T: Queryable<Root>,
    Root: Table,
{
    type Inner = T;

    fn new<Field: Queryable<Root>>(path: &Path) -> Self {
        VecField(Query {
            path: path.clone(),
            subquery: T::QueryType::new::<Field>(path),
            _data: PhantomData::default(),
        })
    }
}

impl<T: Queryable<Root>, Root: Table> VecField<T, Root> {
    pub fn at(&self, index: usize) -> Query<T, Root> {
        let path = self.0.path.join(index);
        Query {
            subquery: T::QueryType::new::<T>(&path),
            path,
            _data: PhantomData::default(),
        }
    }

    pub(crate) fn wildcard(&self) -> Query<T, Root> {
        let path = Path::default();
        Query {
            subquery: T::QueryType::new::<T>(&path),
            path,
            _data: PhantomData::default(),
        }
    }
}
