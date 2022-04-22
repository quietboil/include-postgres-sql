use include_postgres_sql::{ include_sql, impl_sql };
use tokio_postgres::{Config, NoTls};

include_sql!("examples/async/query_no_args.sql");

#[tokio::main]
async fn main() -> Result<(),tokio_postgres::Error> {
    let (db, conn) = Config::new()
        .host("localhost")
        .user("postgres")
        .dbname("chinook")
        .connect(NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
        }
    });

    db.get_top_artists(|row| {
        let artist_name : &str = row.try_get("artist_name")?;
        let num_albums  :  i64 = row.try_get("num_albums")?;
        println!("{}: {}", artist_name, num_albums);
        Ok(())
    }).await?;

    Ok(())
}
