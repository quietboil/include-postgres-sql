#[cfg(feature = "tokio")]
mod test {
    use include_postgres_sql::{ include_sql, impl_sql };
    use tokio_postgres::{Config, NoTls};

    include_sql!("tests/sql/dml_returning.sql");

    #[tokio::test]
    async fn impl_method_returning_row() -> Result<(),tokio_postgres::Error> {
        let (mut db, conn) = Config::new()
            .host("localhost")
            .user("postgres")
            .dbname("chinook")
            .connect(NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = conn.await {
                eprintln!("connection error: {}", e);
            }
        });

        let tr = db.transaction().await?;
        let res = tr.new_genre("New Age").await?;
        let new_id = res.try_get("genre_id")?;
        assert!(new_id > 0);
        tr.delete_genre(new_id).await?;
        tr.rollback().await?;

        Ok(())
    }
}