use crate::change_request::ChangeRequest;
use regex::internal::Input;
use sqlx::sqlite::{SqliteQueryResult, SqliteRow};
use sqlx::{sqlite::SqliteConnectOptions, Error, Pool, Row, Sqlite, SqlitePool};
use std::{future::Future, path::Path};
use tracing::error;

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
        let existing_karma = self.get_karma_for(&id_name[..]).await;

        match existing_karma {
            None => {
                let result = sqlx::query!(
                    "INSERT INTO Entries (IdName, DisplayName, Karma) VALUES (?, ?, ?)",
                    id_name,
                    request.name,
                    request.amount
                )
                .execute(&self.connection)
                .await;
                Self::log_if_error(result);
            }
            Some(karma) => {
                let new_karma = karma + request.amount;
                let result = sqlx::query!(
                    "UPDATE Entries SET Karma = ?\
                    WHERE IdName = ?",
                    new_karma,
                    id_name,
                )
                .execute(&self.connection)
                .await;
                Self::log_if_error(result);
            }
        }
    }

    fn log_if_error<T>(result: Result<T, sqlx::Error>) {
        match result {
            Ok(_) => {}
            Err(err) => {
                error!("Error communicating with DB - was the file deleted or locked? Error is as follows, but do not trust it it will often be wrong or unhelpful: {}", err.to_string())
            }
        }
    }

    pub async fn get_karma_for(&self, name: &str) -> Option<i64> {
        let id_name = name.to_lowercase();
        let result: Result<i64, sqlx::Error> =
            sqlx::query!("SELECT Karma FROM Entries WHERE IdName = ?", id_name)
                .fetch_one(&self.connection)
                .await
                .map(|record| record.Karma);

        match result {
            Ok(karma) => Some(karma),
            Err(err) => match err {
                Error::RowNotFound => None,
                _ => {
                    error!("Error communicating with DB - was the file deleted or locked? Error is as follows, but do not trust it it will often be wrong or unhelpful: {}", err);
                    None
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_request::ChangeRequest;
    use serial_test::serial;
    use std::fs;
    use std::path::Path;
    use tracing_test::traced_test;

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

    #[tokio::test]
    #[serial]
    async fn should_insert_two_karma_changes_and_get_new_number() {
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
        let repo = KarmaRepository::new(DATABASE_FILENAME).await;

        repo.upsert_karma_change(ChangeRequest::new("Sunnydays", 1))
            .await;
        repo.upsert_karma_change(ChangeRequest::new("sUnnydays", 1))
            .await;
        let karma = repo.get_karma_for("sunnydays").await;
        assert_eq!(Some(2), karma);
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }

    #[tokio::test]
    #[serial]
    #[traced_test]
    async fn given_database_error_upsert_should_log_error_but_not_panic() {
        let repo = KarmaRepository::new(DATABASE_FILENAME).await;
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());

        repo.upsert_karma_change(ChangeRequest::new("rainydays", -1))
            .await;

        logs_assert(|lines: &[&str]| match lines.len() {
            2 => Ok(()),
            n => Err(format!("Expected one logs, but found {}", n)),
        });
        assert!(logs_contain("Error communicating with DB - was the file deleted or locked? Error is as follows, but do not trust it it will often be wrong or unhelpful: error returned from database: (code: 1) no such table: Entries"));
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }

    #[tokio::test]
    #[serial]
    #[traced_test]
    async fn given_database_error_get_karma_should_log_error() {
        let repo = KarmaRepository::new(DATABASE_FILENAME).await;
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());

        let karma = repo.get_karma_for("Rainydays").await;

        logs_assert(|lines: &[&str]| match lines.len() {
            1 => Ok(()),
            n => Err(format!("Expected one logs, but found {}", n)),
        });
        assert!(logs_contain("Error communicating with DB - was the file deleted or locked? Error is as follows, but do not trust it it will often be wrong or unhelpful: error returned from database: (code: 1) no such table: Entries"));
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }

    #[tokio::test]
    #[serial]
    #[traced_test]
    async fn given_no_karma_entry_get_karma_should_return_zero_and_not_log_anything() {
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
        let repo = KarmaRepository::new(DATABASE_FILENAME).await;

        let karma = repo.get_karma_for("Rainydays").await;

        logs_assert(|lines: &[&str]| match lines.len() {
            0 => Ok(()),
            n => Err(format!("Expected zero logs, but found {}", n)),
        });
        assert_eq!(None, karma);
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }
}
