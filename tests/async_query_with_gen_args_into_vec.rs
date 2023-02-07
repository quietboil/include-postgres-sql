#[cfg(feature = "tokio")]
mod test {
    use std::convert::TryFrom;

    use include_postgres_sql::{impl_sql, include_sql};
    use tokio_postgres::{Config, NoTls, Error, Row};

    include_sql!("tests/sql/query_with_gen_args_into_vec.sql");

    struct TrackSale {
        artist_name : String,
        track_name  : String,
        num_sold    : i64
    }

    impl TryFrom<Row> for TrackSale {
        type Error = Error;

        fn try_from(row: Row) -> Result<Self, Self::Error> {
            let artist_name = row.try_get(0)?;
            let track_name  = row.try_get(1)?;
            let num_sold    = row.try_get(2)?;
            Ok(Self { artist_name, track_name, num_sold })
        }
    }

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

        let sales : Vec<TrackSale> = db.get_top_sales("London", 2i64).await?;
        assert_eq!(sales.len(), 1);

        assert_eq!(sales[0].artist_name, "Cidade Negra");
        assert_eq!(sales[0].track_name, "Firmamento");
        assert_eq!(sales[0].num_sold, 2);

        Ok(())
    }
}