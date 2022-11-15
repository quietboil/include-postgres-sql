#[cfg(not(feature = "tokio"))]
mod test {
    use include_postgres_sql::{include_sql, impl_sql};
    use postgres::{Config, NoTls, Error};

    include_sql!("tests/sql/query_dyn_args.sql");

    #[test]
    fn impl_method_with_in_params() -> Result<(), Error> {
        let mut db = Config::new()
            .host("localhost")
            .user("postgres")
            .dbname("chinook")
            .connect(NoTls)?
        ;

        let mut row_num = 0;
        db.get_top_sales(&["London", "Berlin"], 2, |row| {
            let artist_name : &str = row.try_get("artist_name")?;
            let track_name  : &str = row.try_get("track_name")?;
            let num_sold    :  i64 = row.try_get("num_sold")?;

            row_num += 1;
            match row_num {
                1 => {
                    assert_eq!(artist_name, "Cidade Negra");
                    assert_eq!(track_name, "Firmamento");
                    assert_eq!(num_sold, 2);        
                },
                2 => {
                    assert_eq!(artist_name, "Iron Maiden");
                    assert_eq!(track_name, "The Number Of The Beast");
                    assert_eq!(num_sold, 2);        
                },
                3 => {
                    assert_eq!(artist_name, "Van Halen");
                    assert_eq!(track_name, "Eruption");
                    assert_eq!(num_sold, 2);        
                },
                _ => {
                    panic!("unexpected row");
                },
            }
            Ok(())
        })?;

        Ok(())
    }
}