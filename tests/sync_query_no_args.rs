#[cfg(not(feature = "tokio"))]
mod test {
    use include_postgres_sql::{include_sql, impl_sql};
    use postgres::{Config, NoTls, Error};

    include_sql!("tests/sql/query_no_args.sql");

    #[test]
    fn impl_method_without_params() -> Result<(), Error> {
        let mut db = Config::new()
            .host("localhost")
            .user("postgres")
            .dbname("chinook")
            .connect(NoTls)?
        ;

        let mut row_num = 0;
        db.get_top_artists(|row| {
            let artist_name : &str = row.try_get("artist_name")?;
            let num_albums  :  i64 = row.try_get("num_albums")?;
            
            row_num += 1;
            match row_num {
                1 => {
                    assert_eq!(artist_name, "Iron Maiden");
                    assert_eq!(num_albums, 21);        
                },
                2 => {
                    assert_eq!(artist_name, "Led Zeppelin");
                    assert_eq!(num_albums, 14);        
                },
                3 => {
                    assert_eq!(artist_name, "Deep Purple");
                    assert_eq!(num_albums, 11);        
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