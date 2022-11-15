#[cfg(not(feature = "tokio"))]
mod test {
    use std::convert::TryFrom;

    use include_postgres_sql::{include_sql, impl_sql};
    use postgres::{Config, NoTls, Error, Row};
    
    include_sql!("tests/sql/query_with_args_into_vec.sql");
    
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

    #[test]
    fn impl_method_with_params() -> Result<(), Error> {
        let mut db = Config::new()
            .host("localhost")
            .user("postgres")
            .dbname("chinook")
            .connect(NoTls)?
        ;
    
        let sales : Vec<TrackSale> = db.get_top_sales("London", 2)?;
        assert_eq!(sales.len(), 1);

        assert_eq!(sales[0].artist_name, "Cidade Negra");
        assert_eq!(sales[0].track_name, "Firmamento");
        assert_eq!(sales[0].num_sold, 2);

        Ok(())
    }
}