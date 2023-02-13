[![crates.io](https://img.shields.io/crates/v/include-postgres-sql)](https://crates.io/crates/include-postgres-sql)
[![Documentation](https://docs.rs/include-postgres-sql/badge.svg)](https://docs.rs/include-postgres-sql)
![MIT](https://img.shields.io/crates/l/include-postgres-sql.svg)

**include-postgres-sql** is an extension of [include-sql][1] for using Postgres SQL in Rust. It completes include-sql by providing `impl_sql` macro to generate database access methods from the included SQL. include-postgres-sql uses [Rust-Postgres][2] for database access.

# Example

Write your SQL and save it in a file. For example, let's say the following is saved as `library.sql` in the project's `sql` folder:

```sql
-- name: get_loaned_books?
--
-- Returns the list of books loaned to a patron
--
-- # Parameters
--
-- param: user_id: &str - user ID
--
SELECT book_title
  FROM library
 WHERE loaned_to = :user_id
 ORDER BY 1

-- name: loan_books!
--
-- Updates the book records to reflect loan to a patron
--
-- # Parameters
--
-- param: book_titles: &str - book titles
-- param: user_id: &str - user ID
--
UPDATE library
   SET loaned_to = :user_id
     , loaned_on = current_timestamp
 WHERE book_title IN (:book_titles)
```

And then use it in Rust as:

```rust , ignore
use include_postgres_sql::{include_sql, impl_sql};
use postgres::{Config, NoTls, Error};

include_sql!("sql/library.sql");

fn main() -> Result<(),Error> {
    let mut db = Config::new().host("localhost").connect(NoTls)?;

    db.loan_books(&["War and Peace", "Gone With the Wind"], "Sheldon Cooper")?;

    db.get_loaned_books("Sheldon Cooper", |row| {
        let book_title : &str = row.try_get(0)?;
        println!("{book_title}");
        Ok(())
    })?;

    Ok(())
}
```

Or, when include-postgres-sql `tokio` feature is selected:

```rust , ignore
use include_postgres_sql::{include_sql, impl_sql};
use tokio_postgres::{Config, NoTls, Error};

include_sql!("sql/library.sql");

#[tokio::main]
async fn main() -> Result<(),Error> {
    let (db, conn) = Config::new().host("localhost").connect(NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    db.loan_books(&["War and Peace", "Gone With the Wind"], "Sheldon Cooper").await?;

    db.get_loaned_books("Sheldon Cooper", |row| {
        let book_title : &str = row.try_get(0)?;
        println!("{book_title}");
        Ok(())
    }).await?;

    Ok(())
}
```

# Documentation

The included [documentation][3] describes the supported SQL file format and provides additional details on the generated code.

# 💥 Breaking Changes in 0.2

* [include-sql][1] changed optional statement terminator from `;` to `/`. SQL files that used `;` terminator would need to change it to `/` or remove it completely.

[1]: https://crates.io/crates/include-sql
[2]: https://crates.io/crates/postgres
[3]: https://quietboil.github.io/include-postgres-sql
