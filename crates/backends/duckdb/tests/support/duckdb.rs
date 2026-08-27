use std::path::{Path, PathBuf};

use duckdb::Connection;
use tempfile::TempDir;

pub struct TestDatabase {
    _directory: TempDir,
    path: PathBuf,
}

impl TestDatabase {
    pub fn from_sql(sql: &str) -> Self {
        let directory = TempDir::new().expect("temporary database directory should be created");
        let path = directory.path().join("app.duckdb");
        create_database(&path, sql);

        Self {
            _directory: directory,
            path,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn sibling_path(&self, name: &str) -> PathBuf {
        self.path
            .parent()
            .expect("fixture database should have a parent")
            .join(name)
    }

    pub fn create_sibling(&self, name: &str, sql: &str) -> PathBuf {
        let path = self.sibling_path(name);
        create_database(&path, sql);
        path
    }
}

fn create_database(path: &Path, sql: &str) {
    let connection = Connection::open(path).expect("temporary DuckDB database should be opened");
    let version = connection
        .query_row("SELECT version()", [], |row| row.get::<_, String>(0))
        .expect("DuckDB fixture should expose its library version");
    assert_eq!(
        version, "v1.5.4",
        "DuckDB fixtures must run against the declared compatibility target"
    );
    connection
        .execute_batch(sql)
        .expect("DuckDB fixture should execute successfully");
}
