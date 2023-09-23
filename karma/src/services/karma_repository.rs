use crate::change_request::ChangeRequest;
use sqlx::sqlite::SqliteRow;
use sqlx::{sqlite::SqliteConnectOptions, Pool, Row, Sqlite, SqlitePool};
use std::{future::Future, path::Path};

pub struct KarmaRepository {
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
        sqlx::migrate!()
            .run(&connection)
            .await
            .expect("Failed applying Karma migrations");
        KarmaRepository { connection }
    }

    pub async fn default() -> Self {
        KarmaRepository::new("karma.db").await
    }

    pub async fn upsert_karma_change(&self, request: ChangeRequest) {
        let id_name = request.name.to_lowercase();
        sqlx::query!(
            "INSERT INTO Entries (IdName, DisplayName, Karma) VALUES (?, ?, ?)",
            id_name,
            request.name,
            request.amount
        )
        .execute(&self.connection)
        .await;
    }

    pub async fn get_karma_for(&self, name: &str) -> Option<i64> {
        let id_name = name.to_lowercase();
        sqlx::query!("SELECT Karma FROM Entries WHERE IdName = ?", id_name)
            .fetch_one(&self.connection)
            .await
            .ok()
            .map(|record| record.Karma)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_request::ChangeRequest;
    use serial_test::serial;
    use std::fs;
    use std::path::Path;

    const DATABASE_FILENAME: &'static str = "testdb.db";

    #[tokio::test]
    #[serial]
    async fn given_database_does_not_exist_should_create_it() {
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());

        KarmaRepository::new(DATABASE_FILENAME).await;

        assert!(Path::new(DATABASE_FILENAME).exists());
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }

    #[tokio::test]
    #[serial]
    async fn should_insert_karma_and_get_new_number() {
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
        let repo = KarmaRepository::new(DATABASE_FILENAME).await;

        repo.upsert_karma_change(ChangeRequest::new("rAinydays", -1))
            .await;
        let karma = repo.get_karma_for("Rainydays").await;
        assert_eq!(Some(-1), karma);
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }
}
