#[cfg(feature = "tokio")]
mod test {
    use futures_util::{pin_mut, TryStreamExt};
    use include_postgres_sql::{impl_sql, include_sql};
    use tokio_postgres::{Config, NoTls};

    include_sql!("tests/sql/query_with_gen_args_into_rows.sql");

    #[tokio::test]
    async fn impl_method_with_params() -> Result<(),tokio_postgres::Error> {
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
        let rows = db.get_top_sales("London", 2i64).await?;
        pin_mut!(rows);
        while let Some(row) = rows.try_next().await? {
            let artist_name : &str = row.try_get("artist_name")?;
            let track_name  : &str = row.try_get("track_name")?;
            let num_sold    :  i64 = row.try_get("num_sold")?;

            row_num += 1;

            assert_eq!(artist_name, "Cidade Negra");
            assert_eq!(track_name, "Firmamento");
            assert_eq!(num_sold, 2);
        }
        assert_eq!(row_num, 1);

        Ok(())
    }
}