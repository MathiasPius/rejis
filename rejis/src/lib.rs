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

/*
#[macro_export]
macro_rules! Segment {
    {$root:ident} => {
        $root::query()
    };
    {$root:ident, .$head:ident} => {
        ::rejis::Segment!{$root}.$head
    };
    {$root:ident, $($prev_tokens:tt)+.$head:tt} => {
        ::rejis::Segment!{$root, $($prev_tokens)*}.$head
    };
    {$root:ident, [$index:literal]} => {
        ::rejis::Segment!{$root}.at($index)
    };
    {$root:ident, $($prev_index:tt)+ [$index:literal]} => {
        ::rejis::Segment!{$root, $($prev_index)*}.at($index)
    };
}

#[macro_export]
macro_rules! Query {
    {$root:ident $($tokens:tt)* } => {
        &::rejis::Segment!($root, $($tokens)*)
    };
}
*/
/*
#[macro_export]
macro_rules!  Query {
    {} => {};
    {$out:expr} => {
        $out
    };
    {$out:expr => .$head:ident $($tail:tt)*} => {
        Query!{$out.$head => $(tail)*}
    };
    {$out:expr => [$index:literal]$($tail:tt)*} => {
        Query!{$out.at($index) => $(tail)*}
    };
    {$root:ident ($tail:tt)*} => {
        Query!{$root::query() => $(tail)*}
    }
}

#[macro_export]
macro_rules!  Query {
    {$elem:tt} => {
        stringify!($elem)
    };
    {($tail:tt)*} => {
        println!(concat!((Query!{$tail})*));
    };
}
*/
#[macro_export]
macro_rules! QQ {
    ($out:expr => && $($tail:tt)*) => {
        ::rejis::filter::And((
            $out,
            QQ!($($tail)*)
        ))
    };
    ($out:expr => || $($tail:tt)*) => {
        ::rejis::filter::Or((
            $out,
            QQ!($($tail)*)
        ))
    };
    ($out:expr => == $value:literal $($tail:tt)*) => {
       ::rejis::QQ!(
            ::rejis::Query::cmp(
                &$out, 
                ::rejis::filter::Operator::Equal,
                $value
            ) => $($tail)*
       )
    };
    ($out:expr => [$index:literal] $($tail:tt)*) => {
        ::rejis::QQ!(
            ::rejis::VecField::at(
                ::std::ops::Deref::deref(&$out),
                $index
            ) => $($tail)*
        );
    };
    ($out:expr => .$next:tt $($tail:tt)*) => {
        ::rejis::QQ!($out.$next => $($tail)*);
    };
    ($out:expr =>) => {
        $out
    };
    ($root:ident $($tail:tt)*) => {
        ::rejis::QQ!($root::query() => $($tail)*);
    };
}
