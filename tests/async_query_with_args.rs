#[cfg(feature = "tokio")]
mod test {
    use include_postgres_sql::{ include_sql, impl_sql };
    use tokio_postgres::{Config, NoTls};

    include_sql!("tests/sql/query_with_args.sql");

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
        db.get_top_sales("London", 2, |row| {
            let artist_name : &str = row.try_get("artist_name")?;
            let track_name : &str = row.try_get("track_name")?;
            let num_sold : i64 = row.try_get("num_sold")?;

            row_num += 1;

            assert_eq!(artist_name, "Cidade Negra");
            assert_eq!(track_name, "Firmamento");
            assert_eq!(num_sold, 2);

            Ok(())
        }).await?;
        assert_eq!(row_num, 1);

        Ok(())
    }
}