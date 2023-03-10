mod database;
pub mod filter;
pub mod path;
mod query;

pub use database::Database;
pub use query::{Query, QueryConstructor, Queryable, Table};
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
macro_rules! Logical {
    {&&} => {::rejis::filter::And};
    {||} => {::rejis::filter::Or}
}

#[macro_export]
macro_rules! Q {
    {$root:ident.$filter:tt $op:tt $value:tt} => {
        $root::query().$filter.cmp(::rejis::Op!{$op}, $value);
    };
    {$root1:ident.$filter1:tt $op1:tt $value1:tt $log:tt $root2:ident.$filter2:tt $op2:tt $value2:tt} => {
        ::rejis::Logical!{$log}((
            Q! {
                $root1.$filter1 $op1 $value1
            },
            Q! {
                $root2.$filter2 $op2 $value2
            }
        ));
    };
    {($($a:tt)*) $log:tt ($($b:tt)*)} => {
        ::rejis::Logical!{$log}((
            Q! {
                $($a)*
            },
            Q! {
                $($b)*
            }
        ));
    };
}
