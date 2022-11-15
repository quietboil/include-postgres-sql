**include-postgres-sql** is an extension of [include-sql][1] for using Postgres SQL in Rust. It completes include-sql by providing `impl_sql` macro to generate database access methods from the included SQL. include-postgres-sql uses [Rust-Postgres][2] for database access.

# Features

**include-postgres-sql** has a single feature - `tokio` - which, when selected, makes include-postgres-sql generate async databases access methods that can be used with [tokio-postgres][5].

# Usage

Add `include-postgres-sql` and `postgres` as a dependency:

```toml
[dependencies]
include-postgres-sql = "0.1"
postgres = "0.19"
```

Write your SQL and save it in a file. For example, let's say the following is the content of the `library.sql` file that is saved in the project's `src` folder:

```sql
-- name: get_loaned_books?
-- Returns the list of books loaned to a patron
-- # Parameters
-- param: user_id: &str - user ID
SELECT book_title
  FROM library
 WHERE loaned_to = :user_id
 ORDER BY 1;

-- name: loan_books!
-- Updates the book records to reflect loan to a patron
-- # Parameters
-- param: user_id: &str - user ID
-- param: book_ids: i32 - book IDs
UPDATE library
   SET loaned_to = :user_id
     , loaned_on = current_timestamp
 WHERE book_id IN (:book_ids);
```

And then use it in Rust as:

```rust , ignore
use include_postgres_sql::{include_sql, impl_sql};
use postgres::{Config, NoTls, Error};

include_sql!("src/library.sql");

fn main() -> Result<(),Error> {
    let args : Vec<String> = std::env::args().collect();
    let user_id = &args[1];

    let mut db = Config::new().host("localhost").connect(NoTls)?;

    db.get_loaned_books(user_id, |row| {
        let book_title : &str = row.try_get("book_title")?;
        println!("{}", book_title);
        Ok(())
    })?;

    Ok(())
}
```

> **Note** that the path to the SQL file must be specified relative to the project root, i.e. relative to `CARGO_MANIFEST_DIR`, even if you keep your SQL file alongside rust module that includes it. Because include-sql targets stable Rust, this requirement will persist until [SourceFile][3] stabilizes.

# Async

Add the following dependencies:

```toml
[dependencies]
include-postgres-sql = { version = "0.1", features = ["tokio"] }
tokio-postgres = "0.7"
tokio = { version = "1", features = ["full"] }
```

> **Note** `full` tokio features are not required. `tokio` dependency is listed like that for illustration only.

The same SQL as above can then be used in async Rust as:

```rust , ignore
use include_postgres_sql::{include_sql, impl_sql};
use tokio_postgres::{Config, NoTls, Error};

include_sql!("src/library.sql");

#[tokio::main]
fn main() -> Result<(),Error> {
    let args : Vec<String> = std::env::args().collect();
    let user_id = &args[1];

    let (db, conn) = Config::new().host("localhost").connect(NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    db.get_loaned_books(user_id, |row| {
        let book_title : &str = row.try_get("book_title")?;
        println!("{}", book_title);
        Ok(())
    }).await?;

    Ok(())
}
```

# Anatomy of the Included SQL File

Please see the **Anatomy of the Included SQL File** in [include-sql][4] documentation for the description of the format that include-sql can parse.

# Generated Methods

**include-postgres-sql** generates 5 variants of database access methods using the following selectors:
* `?` - methods that process rows retrieved by `SELECT`,
* `^` - methods that return rows retrieved by `SELECT` (as `postgres::RowIter` or `tokio_postgres::RowStream` ),
* `%` - methods that extract data from selected rows into row specific structs and return them as `Vec`,
* `!` - methods that execute all other non-`SELECT` methods, and
* `->` - methods that execute `RETURNING` statements and provide access to returned data.

## Process Selected Rows

### Callback

For the `SELECT` statement like:

```sql
-- name: get_loaned_books?
-- param: user_id: &str
SELECT book_title FROM library WHERE loaned_to = :user_id;
```

The method with the following signature is generated:

```rust , ignore
fn get_loaned_books<F>(&self, user_id: &str, row_callback: F) -> Result<(),postgres::Error>
where F: Fn(postgres::Row) -> Result<(),postgres::Error>;
```

Where:
- `user_id` is a parameter that has the same name as the SQL parameter with the declared (in the SQL) type as `&str`.
- `F` is a type of a callback (closure) that the method implementation will call to process each row.

### Row Iterator

When the same `SELECT` statement is tagged as `^`:

```sql
-- name: get_loaned_books^
-- param: user_id: &str
SELECT book_title FROM library WHERE loaned_to = :user_id;
```

Then a regular `postgres` variant of the generated method will have the following signature:

```rust , ignore
fn get_loaned_books<'a>(&'a self, user_id: &str) -> Result<postgres::RowIter<'a>,postgres::Error>;
```

### Row Stream

For the same - tagged as `^` - `SELECT` statement the `tokio-postgres` variant, i.e. when `tokio` feature is used, of the generated method will have the following signature:

```rust , ignore
async fn get_loaned_books(&self, user_id: &str) -> Result<tokio_postgres::RowStream,tokio_postgres::Error>;
```

### Vector

When the same `SELECT` statement is tagged as `%`:

```sql
-- name: get_loaned_books%
-- param: user_id: &str
SELECT isbn, book_title FROM library WHERE loaned_to = :user_id;
```

Then a regular `postgres` variant of the generated method will have the following signature:

```rust , ignore
fn get_loaned_books<R>(&self, user_id: &str) -> Result<Vec<R>,postgres::Error>
where R: TryFrom<Row>, postgres::Error: From<R::Error>;;
```

It requires a struct defines that is capable deserializing a returned `Row`. For example, to deserialize the rows from the above query the following struct can be defined:

```rust , ignore
struct LoanedBook {
    isbn: String,
    title: String,
}

impl TryFrom<postgres::Row> for LoanedBook {
    type Error = postgres::Error;

    fn try_from(row: postgres::Row) -> Result<Self, Self::Error> {
        let isbn  = row.try_get(0)?;
        let title = row.try_get(1)?;
        Ok(Self {isbn, title})
    }
}
```

Then the entire result set can be retrieved as:

```rust , ignore
let loaned_books : Vec<LoanedBook> = db.get_loaned_books(user_id)?;
```

## Execute Non-Select Statements

For non-select statements - INSERT, UPDATE, DELETE, etc. - like the following:

```sql
-- name: loan_books!
-- param: user_id: &str
-- param: book_ids: i32
UPDATE library
   SET loaned_to = :user_id
     , loaned_on = current_timestamp
 WHERE book_id IN (:book_ids);
```

The method with the following signature is generated:

```rust , ignore
fn loan_books(&self, user_id: &str, book_ids: &[i32]) -> Result<u64,postgres::Error>;
```

Where:
- `user_id` is a parameter that has the same name as the SQL parameter with the declared (in the SQL) type as `&str`,
- `book_ids` is a parameter for the matching IN-list parameter where each item in a collection has type `u32`.

## RETURNING Statements

For DELETE, INSERT, and UPDATE statements that return data via `RETURNING` clause like:

```sql
-- name: add_new_book->
-- param: isbn: &str
-- param: book_title: &str
INSERT INTO library (isbn, book_title)
VALUES (:isbn, :book_title)
RETURNING book_id;
```

The method with the following signature is generated:

```rust , ignore
fn add_new_book(&self, isbn: &str, book_title: &str) -> Result<postgres::Row,postgres::Error>;
```

# Inferred Parameter Types

If a statement parameter type is not explicitly specified via `param:`, **include-postgres-sql** will use `impl postgres::types::ToSql` for the corresponding scalar method parameters. For example, if the SQL from the example above has not provided its parameter type:

```sql
-- name: get_loaned_books?
SELECT book_title
  FROM library
 WHERE loaned_to = :user_id
 ORDER BY 1;
```

Then the signature of the generated method would be:

```rust , ignore
fn get_loaned_books<F>(&self, user_id: impl postgres::types::ToSql, row_callback: F) -> Result<(),postgres::Error>
where F: Fn(postgres::Row) -> Result<(),postgres::Error>;
```

For the "IN list" type of parameters **include-postgres-sql** will generate a method parameter as a slice where each element is the same generic type supplied by **include-sql**:

```sql
-- name: loan_books!
UPDATE library
   SET loaned_to = :user_id
     , loaned_on = current_timestamp
 WHERE book_id IN (:book_ids);
```

The signature of the generated method would be:

```rust , ignore
fn loan_books<BookIds: postgres::types::ToSql>(&self, user_id: impl postgres::types::ToSql, book_ids: &[BookIds]) -> Result<u64,postgres::Error>;
```

# Examples

**include-postgres-sql** integration [tests][6] are written as tiny applications that can be used as examples of various usages of included SQL.

[1]: https://crates.io/crates/include-sql
[2]: https://crates.io/crates/postgres
[3]: https://doc.rust-lang.org/proc_macro/struct.SourceFile.html
[4]: https://quietboil.github.io/include-sql
[5]: https://crates.io/crates/tokio-postgres
[6]: https://github.com/quietboil/include-postgres-sql/tree/master/tests