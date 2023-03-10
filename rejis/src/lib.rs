mod database;
pub mod filter;
pub mod path;
mod query;

pub use database::Database;
pub use query::*;
pub use rejis_derive::{Queryable, Table};

#[macro_export]
macro_rules! Op {
    {==} => {::rejis::filter::Operator::Equal};
    {!=} => {::rejis::filter::Operator::NotEqual};
    {>} => {::rejis::filter::Operator::GreaterThan};
    {>=} => {::rejis::filter::Operator::GreaterThanOrEqual};
    {<} => {::rejis::filter::Operator::LessThan};
    {<=} => {::rejis::filter::Operator::LessThanOrEqual};
    {like} => {::rejis::filter::Operator::Like};
    {$op:tt} => {
        compile_error!("This macro only accepts == != > >= < <= like");
    }
}

#[macro_export]
macro_rules! Q {
    (($($head:tt)*) && ($($tail:tt)*)) => {
        ::rejis::filter::And((
            Q!($($head)*),
            Q!($($tail)*),
        ))
    };
    (($($head:tt)*) || ($($tail:tt)*)) => {
        ::rejis::filter::Or((
            Q!($($head)*),
            Q!($($tail)*),
        ))
    };
    ($out:expr => $op:tt $value:literal) => {
        ::rejis::Q!(
             ::rejis::Query::cmp(
                 &$out,
                 ::rejis::Op!{$op},
                 $value,
             ) =>
        )
    };
    ($out:expr => $op:tt &$value:expr) => {
        ::rejis::Q!(
             ::rejis::Query::cmp(
                 &$out,
                 ::rejis::Op!{$op},
                 $value,
             ) =>
        )
    };
    ($out:expr => $op:tt $value:ident) => {
        ::rejis::Q!(
             ::rejis::Query::cmp(
                 &$out,
                 ::rejis::Op!{$op},
                 $value,
             ) =>
        )
    };
    ($out:expr => $op:tt $(tail:tt)*) => {
        ::rejis::Q!(
             ::rejis::Query::cmp(
                 &$out,
                 ::rejis::Op!{$op},
                 Q!($($tail)*),
             ) =>
        )
    };
    ($out:expr => [$index:literal]) => {
        ::rejis::Q!(
            ::rejis::VecField::at(
                ::std::ops::Deref::deref(&$out),
                $index
            ) =>
        );
    };
    ($out:expr => [..]$(.$sub:ident)+ $op:tt $value:literal $($tail:tt)*) => {
        ::rejis::Q!(
            ::rejis::Query::any(
                &$out,
                |query| query$(.$sub)*.clone(),
                ::rejis::Op!{$op},
                $value,
            ) => $($tail)*
       )
    };
    ($out:expr => .$next:tt $($tail:tt)*) => {
        ::rejis::Q!($out.$next => $($tail)*);
    };
    ($out:expr =>) => {
        $out
    };
    ($root:ident $($tail:tt)*) => {
        ::rejis::Q!($root::query() => $($tail)*);
    };
}
