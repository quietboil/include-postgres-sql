#[cfg(not(feature = "tokio"))]
mod test {
    use include_postgres_sql::{impl_sql, include_sql};
    use postgres::{Config, NoTls};

    include_sql!("tests/sql/dml_dyn.sql");

    #[test]
    fn impl_method_with_in_params() -> Result<(), postgres::Error> {
        let mut db = Config::new()
            .host("localhost")
            .user("postgres")
            .dbname("chinook")
            .connect(NoTls)?;

        let mut tr = db.transaction()?;
        tr.delete_genres(&["New Age", "Casual Listening", "White Noise"])?;
        tr.rollback()?;

        Ok(())
    }
}
