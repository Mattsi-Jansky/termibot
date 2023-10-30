use crate::change_request::ChangeRequest;
use crate::entry::Entry;
use crate::reason::Reason;
use async_trait::async_trait;
use mockall::automock;

use sqlx::{sqlite::SqliteConnectOptions, Error, Pool, Sqlite, SqlitePool};

use tracing::error;

#[async_trait]
#[automock]
pub trait KarmaRepository {
    async fn upsert_karma_change(&self, request: ChangeRequest);
    async fn get_karma_for(&self, name: &str) -> Option<i64>;
    async fn get_top(&self, n: i32) -> Vec<Entry>;
    async fn insert_karma_reason(&self, name: &str, change: i64, value: &str);
    async fn get_reasons(&self, name: &str) -> Vec<Reason>;
}

pub struct SqliteKarmaRepository {
    connection: Pool<Sqlite>,
}

impl SqliteKarmaRepository {
    async fn new(filename: &'static str) -> SqliteKarmaRepository {
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
        SqliteKarmaRepository { connection }
    }

    pub async fn default() -> Self {
        SqliteKarmaRepository::new("karma.db").await
    }

    fn log_if_error<T>(result: Result<T, sqlx::Error>) {
        match result {
            Ok(_) => {}
            Err(err) => {
                error!("Error communicating with DB - was the file deleted or locked? Error is as follows, but do not trust it it will often be wrong or unhelpful: {}", err.to_string())
            }
        }
    }
}

#[async_trait]
impl KarmaRepository for SqliteKarmaRepository {
    async fn upsert_karma_change(&self, request: ChangeRequest) {
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

    async fn get_karma_for(&self, name: &str) -> Option<i64> {
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

    async fn get_top(&self, n: i32) -> Vec<Entry> {
        let mut result = vec![];
        let records = sqlx::query!(
            "SELECT IdName, DisplayName, Karma FROM Entries ORDER BY Karma DESC LIMIT ?",
            n
        )
        .fetch_all(&self.connection)
        .await;

        for record in records.unwrap() {
            result.push(Entry {
                id_name: record.IdName,
                display_name: record.DisplayName,
                karma: record.Karma,
            })
        }

        result
    }

    async fn insert_karma_reason(&self, name: &str, change: i64, value: &str) {
        let result = sqlx::query!(
            "INSERT INTO Reasons (Name, Change, Value) VALUES (?, ?, ?)",
            name,
            change,
            value
        )
        .execute(&self.connection)
        .await;
        Self::log_if_error(result);
    }

    async fn get_reasons(&self, name: &str) -> Vec<Reason> {
        let mut result = vec![];

        let records = sqlx::query!("SELECT Value, Change FROM Reasons WHERE Name = ?", name)
            .fetch_all(&self.connection)
            .await;

        for record in records.unwrap() {
            result.push(Reason {
                change: record.Change,
                value: record.Value,
            })
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change_request::ChangeRequest;
    use crate::services::karma_repository::KarmaRepository;
    use serial_test::serial;
    use std::fs;
    use std::path::Path;
    use tracing_test::traced_test;
    use crate::services::karma_parser::KarmaCapture;

    const DATABASE_FILENAME: &str = "testdb.db";

    impl KarmaCapture {
        pub fn new(name: String, is_increment: bool, reason: Option<String>) -> Self {
            Self { name, is_increment, reason }
        }
    }

    #[tokio::test]
    #[serial]
    async fn given_database_does_not_exist_should_create_it() {
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());

        SqliteKarmaRepository::new(DATABASE_FILENAME).await;

        assert!(Path::new(DATABASE_FILENAME).exists());
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }

    #[tokio::test]
    #[serial]
    async fn should_insert_karma_and_get_new_number() {
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
        let repo = SqliteKarmaRepository::new(DATABASE_FILENAME).await;

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
        let repo = SqliteKarmaRepository::new(DATABASE_FILENAME).await;

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
    async fn should_get_top_karma_scores() {
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
        let repo = SqliteKarmaRepository::new(DATABASE_FILENAME).await;
        repo.upsert_karma_change(ChangeRequest::new("sunnydays", 1))
            .await;
        repo.upsert_karma_change(ChangeRequest::new("rainydays", -1))
            .await;

        let result = repo.get_top(10).await;

        assert_eq!(2, result.len());
        assert_eq!(
            &Entry {
                id_name: "sunnydays".to_string(),
                display_name: "sunnydays".to_string(),
                karma: 1
            },
            result.get(0).unwrap()
        );
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }

    #[tokio::test]
    #[serial]
    #[traced_test]
    async fn given_database_error_upsert_should_log_error_but_not_panic() {
        let repo = SqliteKarmaRepository::new(DATABASE_FILENAME).await;
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());

        repo.upsert_karma_change(ChangeRequest::new("rainydays", -1))
            .await;

        logs_assert(|lines: &[&str]| match lines.len() {
            2 => Ok(()),
            n => Err(format!("Expected two logs, but found {}", n)),
        });
        assert!(logs_contain("Error communicating with DB - was the file deleted or locked? Error is as follows, but do not trust it it will often be wrong or unhelpful: error returned from database: (code: 1) no such table: Entries"));
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }

    #[tokio::test]
    #[serial]
    #[traced_test]
    async fn given_database_error_get_karma_should_log_error() {
        let repo = SqliteKarmaRepository::new(DATABASE_FILENAME).await;
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());

        let _karma = repo.get_karma_for("Rainydays").await;

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
        let repo = SqliteKarmaRepository::new(DATABASE_FILENAME).await;

        let karma = repo.get_karma_for("Rainydays").await;

        logs_assert(|lines: &[&str]| match lines.len() {
            0 => Ok(()),
            n => Err(format!("Expected zero logs, but found {}", n)),
        });
        assert_eq!(None, karma);
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }

    #[tokio::test]
    #[serial]
    async fn should_insert_karma_reason() {
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
        let repo = SqliteKarmaRepository::new(DATABASE_FILENAME).await;

        repo.upsert_karma_change(ChangeRequest::new("rAinydays", -1))
            .await;
        repo.insert_karma_reason("rainydays", -1, "for being warm")
            .await;
        let results = repo.get_reasons("rainydays").await;

        assert_eq!(1, results.len());
        assert_eq!("for being warm", results.get(0).unwrap().value);
        fs::remove_file(DATABASE_FILENAME).unwrap_or(());
    }
}
