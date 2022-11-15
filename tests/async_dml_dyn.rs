#[cfg(feature = "tokio")]
mod test {
    use include_postgres_sql::{ include_sql, impl_sql };
    use tokio_postgres::{Config, NoTls};

    include_sql!("tests/sql/dml_dyn.sql");

    #[tokio::test]
    async fn impl_method_with_in_params() -> Result<(),tokio_postgres::Error> {
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
        tr.delete_genres(&["New Age", "Casual Listening", "White Noise"]).await?;
        tr.rollback().await?;

        Ok(())
    }
}