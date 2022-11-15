#[cfg(feature = "tokio")]
mod test {
    use include_postgres_sql::{ include_sql, impl_sql };
    use tokio_postgres::{Config, NoTls};

    include_sql!("tests/sql/dml_simple.sql");

    #[tokio::test]
    async fn impl_dml_method() -> Result<(),tokio_postgres::Error> {
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
        tr.new_genre(99, "New Age").await?;
        tr.delete_genre(99).await?;
        tr.rollback().await?;

        Ok(())
    }
}