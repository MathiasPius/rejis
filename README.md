# rejis
Adventures in type-safe querying of json-serializable structures in SQLite.

This crate aims to explore development of an API built on top of `rusqlite` which allows
simple and sub-optimal querying and storage of `serde_json` serializable structures within
an sqlite database.

The purpose of this is to be able to use sqlite as a persistent store of data without spending
undue time translating between complex nested structures and sqlite tables.

Rejis is not an ORM, as it completely disregards the *relational* benefits of a database, and
instead abuses sqlite as a sort of low-performance on-disk `Vec`, with some sql-aware abstractions
built on top of it, to reduce database roundtrips.

# Roadmap
* Deleting entries. Right now only insertion and queries are supported.
  ```rust
  // Delete all the Johns!
  db.delete(Q! {
    User.first_name == "John"
  });
  ```
* Updating/replacing entire entries.
  ```rust
  // Replace John Smith with Jane Doe
  db.replace(Q! {
    (User.first_name == "John") && (User.last_name == "Smith")
  }, User {
    first_name: "Jane".to_string(),
    last_name: "Doe".to_string(),
  })
  ```

## Tentative features
* Query mapping

  Right now the Query is only used for filtering, but the same structure
  could be useful for optionally narrowing the selection retrieved from the database.
  If all you're interesting in, is the first name of the user, then being able 
  to do something like this could be useful:
  ```rust
  // Last names of everyone named John
  let last_name: Vec<String> = db
    .get(Q! { User.first_name == "John" })
    .map(Q! { User.last_name });
  ```

* Partial updates.

  Using a filter for selection, a query for targeting, and a closure for manipulation,
  a nice api could be made for highly selective updates 
  ```rust
  // Uppercase all the last names of people called John
  db.modify(
    // Select specific rows
    Q! { User.first_name == "John" },
    // Target the last_name specifically for replacement
    Q! { User.last_name },
    // Provide a function for how the name should be transformed.
    |last_name| last_name.to_uppercase()
  );
  ```
  Query mapping might be very relevant for the final form of this API.

* Two-stage Query application.

  The current API requires all parameters to be known at construction time, even though
  these values are only fed into the SQL prepared statement when it is actually applied.
  This is because the values for comparisons for example are stored within the `Comparison`
  struct itself.

  Being able to construct an entire query once and then re-using could bring better performance.
  Since the `rusqlite` prepared statement API borrows the connection object, it might not be
  possible to reuse the entire statement as-is, but sqlite itself supports caching prepared
  statements, presumeably based on the input sql, so if the sql statement itself can be constructed
  once, and then reused, it might still have an impact.


* Expression indices.
  
  Sqlite supports creating an index over an expression within a table,
  which might be useful for improving the performance of commonly used queries.
  If you use `rejis` for storing users for example, it might be useful to be able to create
  an index over `json_extract(value, '$.id')` or `$.name`, if those are used to find users.

# Shortcomings
* Query paths only allow a single indexing element. 
  Reason for this is in the complexity of implementing the SQL CTE and the Q!-macro DSL support for that use case.
