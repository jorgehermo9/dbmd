use std::path::Path;

use rusqlite::Connection;
use tempfile::TempDir;

pub struct TestDatabase {
    _directory: TempDir,
    path: std::path::PathBuf,
}

impl TestDatabase {
    pub fn from_sql(sql: &str) -> Self {
        let directory = TempDir::new().expect("temporary database directory should be created");
        let path = directory.path().join("fixture.sqlite");
        let connection =
            Connection::open(&path).expect("temporary SQLite database should be opened");
        let version = connection
            .query_row("SELECT sqlite_version()", [], |row| row.get::<_, String>(0))
            .expect("SQLite fixture should expose its library version");
        assert_eq!(
            version, "3.53.3",
            "SQLite fixtures must run against the declared compatibility target"
        );
        connection
            .execute_batch(sql)
            .expect("SQLite fixture should execute successfully");
        drop(connection);

        Self {
            _directory: directory,
            path,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
