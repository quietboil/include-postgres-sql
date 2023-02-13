use include_postgres_sql::*;

include_sql!("examples/library.sql");

#[cfg(not(feature = "tokio"))]
fn main() -> Result<(), postgres::Error> {
    use postgres::{Config, NoTls};

    let mut db = Config::new()
        .host("localhost")
        .user("postgres")
        .dbname("postgres")
        .connect(NoTls)?
    ;

    db.init_library()?;

    db.loan_books(&["War and Peace", "Gone With the Wind"], "Sheldon Cooper")?;
    db.loan_books(&["The Lord of the Rings", "Master and Commander"], "Leonard Hofstadter")?;

    db.get_loaned_books("Sheldon Cooper", |row| {
        let book_title : &str = row.try_get(0)?;
        println!("{book_title}");
        Ok(())
    })?;

    println!("---");

    db.get_loaned_books("Leonard Hofstadter", |row| {
        let book_title : &str = row.try_get(0)?;
        println!("{book_title}");
        Ok(())
    })?;

    db.drop_library()?;

    Ok(())
}

#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() -> Result<(),tokio_postgres::Error> {
    use tokio_postgres::{Config, NoTls};
    
    let (db, conn) = Config::new()
        .host("localhost")
        .user("postgres")
        .dbname("postgres")
        .connect(NoTls).await?
    ;
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    db.init_library().await?;

    db.loan_books(&["War and Peace", "Gone With the Wind"], "Sheldon Cooper").await?;
    db.loan_books(&["The Lord of the Rings", "Master and Commander"], "Leonard Hofstadter").await?;

    db.get_loaned_books("Sheldon Cooper", |row| {
        let book_title : &str = row.try_get(0)?;
        println!("{book_title}");
        Ok(())
    }).await?;

    println!("---");

    db.get_loaned_books("Leonard Hofstadter", |row| {
        let book_title : &str = row.try_get(0)?;
        println!("{book_title}");
        Ok(())
    }).await?;

    db.drop_library().await?;

    Ok(())
}
