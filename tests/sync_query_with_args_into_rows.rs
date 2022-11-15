#[cfg(not(feature = "tokio"))]
mod test {
    use include_postgres_sql::{include_sql, impl_sql};
    use postgres::{Config, NoTls, Error, fallible_iterator::FallibleIterator};
    
    include_sql!("tests/sql/query_with_args_into_rows.sql");
    
    #[test]
    fn impl_method_with_params() -> Result<(), Error> {
        let mut db = Config::new()
            .host("localhost")
            .user("postgres")
            .dbname("chinook")
            .connect(NoTls)?
        ;
    
        let mut row_num = 0;
        let mut rows = db.get_top_sales("London", 2)?;
        while let Some(row) = rows.next()? {
            let artist_name : &str = row.try_get("artist_name")?;
            let track_name  : &str = row.try_get("track_name")?;
            let num_sold    :  i64 = row.try_get("num_sold")?;
            
            assert_eq!(artist_name, "Cidade Negra");
            assert_eq!(track_name, "Firmamento");
            assert_eq!(num_sold, 2);
    
            row_num += 1;
        }
        assert_eq!(row_num, 1);
    
        Ok(())
    }
}