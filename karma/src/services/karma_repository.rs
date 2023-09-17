use sqlx::{sqlite::SqliteConnectOptions, Error, Pool, Sqlite, SqlitePool};
use std::{future::Future, path::Path};

struct KarmaRepository {
    connection: Pool<Sqlite>,
}

impl KarmaRepository {
    async fn new(filename: &'static str) -> KarmaRepository {
        let options = SqliteConnectOptions::new()
            .filename(filename)
            .create_if_missing(true);

        let connection = SqlitePool::connect_with(options)
            .await
            .expect("Cannot read sqlite DB");
        KarmaRepository { connection }
    }

    async fn default() -> Self {
        KarmaRepository::new("karma.db").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;

    #[tokio::test]
    async fn given_database_does_not_exist_should_create_it() {
        let filename = "testdb.db";
        fs::remove_file(filename).unwrap_or(());

        KarmaRepository::new(filename).await;

        assert!(Path::new(filename).exists());
        fs::remove_file(filename).unwrap_or(());
    }
}
