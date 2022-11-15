#[cfg(feature = "tokio")]
mod test {
    use futures_util::{pin_mut, TryStreamExt};
    use include_postgres_sql::{impl_sql, include_sql};
    use tokio_postgres::{Config, NoTls};

    include_sql!("tests/sql/query_no_args_into_rows.sql");

    #[tokio::test]
    async fn impl_method_without_params() -> Result<(), tokio_postgres::Error> {
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

        let mut row_num = 0;
        let rows = db.get_top_artists().await?;
        pin_mut!(rows);
        while let Some(row) = rows.try_next().await? {
            let artist_name : &str = row.try_get("artist_name")?;
            let num_albums  :  i64 = row.try_get("num_albums")?;

            row_num += 1;
            match row_num {
                1 => {
                    assert_eq!(artist_name, "Iron Maiden");
                    assert_eq!(num_albums, 21);
                }
                2 => {
                    assert_eq!(artist_name, "Led Zeppelin");
                    assert_eq!(num_albums, 14);
                }
                3 => {
                    assert_eq!(artist_name, "Deep Purple");
                    assert_eq!(num_albums, 11);
                }
                _ => {
                    panic!("unexpected row");
                }
            }
        }

        Ok(())
    }
}
