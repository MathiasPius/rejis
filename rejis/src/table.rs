use serde::Serialize;

use crate::{Query, Queryable};

/// Describes how to store the type for which it is implemented
/// in an sqlite table.
pub trait Table: Queryable<Self> + Serialize + Sized + 'static {
    /// Name used for the table in the database when reading or writing
    /// this object to it.
    const TABLE_NAME: &'static str;

    fn query() -> Query<Self, Self> {
        Query::<Self, Self>::default()
    }
}
