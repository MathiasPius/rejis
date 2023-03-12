/// Translates a comparison operator token into an `Operator`
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

/// DSL for building queries in a more human-readable way
///
/// See tests in `tests/macro_query.rs` for examples on how to
/// use it.
#[macro_export]
macro_rules! Q {
    // AND operator support. Both sides of the comparison must be
    // enclosed in parentheses for macro parsing reasons.
    (($($head:tt)*) && ($($tail:tt)*)) => {
        ::rejis::filter::And(
            Q!($($head)*),
            Q!($($tail)*),
        )
    };
    // OR operator support. Both sides of the comparison must be
    // enclosed in parentheses for macro parsing reasons.
    (($($head:tt)*) || ($($tail:tt)*)) => {
        ::rejis::filter::Or(
            Q!($($head)*),
            Q!($($tail)*),
        )
    };
    // Comparison with literals
    ($out:expr => $op:tt $value:literal) => {
        ::rejis::Q!(
             ::rejis::Query::cmp(
                 &$out,
                 ::rejis::Op!{$op},
                 $value,
             ) =>
        )
    };
    // Comparisons with expressions: User.name == &names[0].
    // Leading & is used work around abmiguous matching with tt lists
    ($out:expr => $op:tt &$value:expr) => {
        ::rejis::Q!(
             ::rejis::Query::cmp(
                 &$out,
                 ::rejis::Op!{$op},
                 $value,
             ) =>
        )
    };
    // Comparison with plain idents.
    ($out:expr => $op:tt $value:ident) => {
        ::rejis::Q!(
             ::rejis::Query::cmp(
                 &$out,
                 ::rejis::Op!{$op},
                 $value,
             ) =>
        )
    };
    // Indexing step of path traveling, with no trailing members.
    ($out:expr => [$index:literal]) => {
        ::rejis::Q!(
            ::rejis::VecField::at(
                ::std::ops::Deref::deref(&$out),
                $index
            ) =>
        );
    };
    // User.pets[..].name == "Value"
    // Note that only one indexing segment is allowed per statement.
    ($out:expr => [..]$(.$sub:ident)+ $op:tt $value:literal) => {
        ::rejis::Q!(
            ::rejis::Query::any(
                &$out,
                |query| query$(.$sub)*.clone(),
                ::rejis::Op!{$op},
                $value,
            ) =>
       )
    };
    // User.pets[..].name == &names[0]
    ($out:expr => [..]$(.$sub:ident)+ $op:tt &$value:expr) => {
        ::rejis::Q!(
            ::rejis::Query::any(
                &$out,
                |query| query$(.$sub)*.clone(),
                ::rejis::Op!{$op},
                $value,
            ) =>
       )
    };
    // User.pets[..].name == name
    ($out:expr => [..]$(.$sub:ident)+ $op:tt $value:ident) => {
        ::rejis::Q!(
            ::rejis::Query::any(
                &$out,
                |query| query$(.$sub)*.clone(),
                ::rejis::Op!{$op},
                $value,
            ) =>
       )
    };
    // User.pets[..] == "Value"
    ($out:expr => [..] $op:tt $value:literal) => {
        ::rejis::Q!(
            ::rejis::Query::any(
                &$out,
                |query| query.clone(),
                ::rejis::Op!{$op},
                $value,
            ) =>
       )
    };
    // User.pets[..] == &name[0]
    ($out:expr => [..] $op:tt &$value:expr) => {
        ::rejis::Q!(
            ::rejis::Query::any(
                &$out,
                |query| query.clone(),
                ::rejis::Op!{$op},
                $value,
            ) =>
       )
    };
    // User.pets[..] == name
    ($out:expr => [..] $op:tt $value:ident) => {
        ::rejis::Q!(
            ::rejis::Query::any(
                &$out,
                |query| query.clone(),
                ::rejis::Op!{$op},
                $value,
            ) =>
       )
    };
    // Continuously matches subquery such as User.address.number and yields the remainder.
    ($out:expr => .$next:tt $($tail:tt)*) => {
        ::rejis::Q!($out.$next => $($tail)*);
    };
    // Epsilon statement
    ($out:expr =>) => {
        $out
    };
    // Entrypoint of a single path such as `User.name` where `User`is the $root.
    ($root:ident $($tail:tt)*) => {
        ::rejis::Q!($root::query() => $($tail)*);
    };
}
