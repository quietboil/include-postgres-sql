use include_postgres_sql::{ include_sql, impl_sql };
use tokio_postgres::{Config, NoTls};

include_sql!("examples/async/returning_dml.sql");

#[tokio::main]
async fn main() -> Result<(),tokio_postgres::Error> {
    let (mut db, conn) = Config::new()
        .host("localhost")
        .user("postgres")
        .dbname("chinook")
        .connect(NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    println!("Creating and deleting a genre...");
    let tr = db.transaction().await?;
    let res = tr.new_genre("New Age").await?;
    let new_id = res.try_get("genre_id")?;
    println!("id={}", new_id);
    tr.delete_genre(new_id).await?;
    tr.rollback().await?;
    println!("Done.");

    Ok(())
}
