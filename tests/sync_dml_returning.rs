#[cfg(not(feature = "tokio"))]
mod test {
    use include_postgres_sql::{impl_sql, include_sql};
    use postgres::{Config, NoTls};

    include_sql!("tests/sql/dml_returning.sql");

    #[test]
    fn impl_method_with_in_params() -> Result<(), postgres::Error> {
        let mut db = Config::new()
            .host("localhost")
            .user("postgres")
            .dbname("chinook")
            .connect(NoTls)?;

        let mut tr = db.transaction()?;
        let res = tr.new_genre("New Age")?;
        let new_id = res.try_get("genre_id")?;
        assert!(new_id > 0);
        tr.delete_genre(new_id)?;
        tr.rollback()?;

        Ok(())
    }
}
