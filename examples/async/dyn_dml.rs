use include_postgres_sql::{ include_sql, impl_sql };
use tokio_postgres::{Config, NoTls};

include_sql!("examples/async/dyn_dml.sql");

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

    println!("Deleting some genres...");
    let tr = db.transaction().await?;
    tr.delete_genres(&["New Age", "Casual Listening", "White Noise"]).await?;
    tr.rollback().await?;
    println!("Done.");

    Ok(())
}
