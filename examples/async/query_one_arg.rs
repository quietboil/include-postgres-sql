use include_postgres_sql::{ include_sql, impl_sql };
use tokio_postgres::{Config, NoTls};

include_sql!("examples/async/query_one_arg.sql");

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

    db.get_artist_albums("U2", |row| {
        let artist_name : &str = row.try_get("artist_name")?;
        let album_title : &str = row.try_get("album_title")?;
        println!("{}: {}", artist_name, album_title);
        Ok(())
    }).await?;

    Ok(())
}
