use include_postgres_sql::*;

include_sql!("tests/sql/query_no_args.sql");
    
#[cfg(feature = "tokio")]
#[tokio::main]
async fn main() -> Result<(), tokio_postgres::Error> {
    use bb8_postgres::PostgresConnectionManager;
    use bb8_postgres::bb8::Pool;
    use tokio_postgres::{Config, NoTls};

    let mut config = Config::new();
    config.host("localhost").user("postgres").dbname("chinook");

    let manager = PostgresConnectionManager::new(config, NoTls);
    let pool = Pool::builder().build(manager).await?;

    let mut tasks = Vec::new();
    for _ in 0..20 {
        let pool = pool.clone();
        let task = tokio::spawn(async move {
            let conn = pool.get().await.unwrap();
            conn.get_top_artists(|row| {
                let artist_name : &str = row.try_get("artist_name")?;
                let num_albums  :  i64 = row.try_get("num_albums")?;
                println!("{artist_name}: {num_albums}");
                Ok(())
            }).await.unwrap();
        });
        tasks.push(task);
    }
    for task in tasks {
        let _ = task.await;
    }
    Ok(())
}

#[cfg(not(feature = "tokio"))]
fn main() {}