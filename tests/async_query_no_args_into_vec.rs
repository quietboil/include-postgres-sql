#[cfg(feature = "tokio")]
mod test {
    use std::convert::TryFrom;

    use include_postgres_sql::{impl_sql, include_sql};
    use tokio_postgres::{Config, NoTls, Error, Row};

    include_sql!("tests/sql/query_no_args_into_vec.sql");

    struct ArtistAlbums {
        artist_name : String,
        num_albums  : i64
    }

    impl TryFrom<Row> for ArtistAlbums {
        type Error = Error;

        fn try_from(row: Row) -> Result<Self, Self::Error> {
            let artist_name = row.try_get(0)?;
            let num_albums  = row.try_get(1)?;
            Ok(Self { artist_name, num_albums })
        }
    }

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

        let top_artists : Vec<ArtistAlbums> = db.get_top_artists().await?;
        assert_eq!(top_artists.len(), 3);

        assert_eq!(top_artists[0].artist_name, "Iron Maiden");
        assert_eq!(top_artists[0].num_albums, 21);        

        assert_eq!(top_artists[1].artist_name, "Led Zeppelin");
        assert_eq!(top_artists[1].num_albums, 14);        

        assert_eq!(top_artists[2].artist_name, "Deep Purple");
        assert_eq!(top_artists[2].num_albums, 11);        
 
        Ok(())
    }
}
