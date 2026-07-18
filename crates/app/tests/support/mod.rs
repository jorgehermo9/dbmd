use std::{collections::BTreeMap, fs, path::Path};

use dbmd_app::RenderRequest;
use rusqlite::Connection;
use tempfile::TempDir;

pub struct TestProject {
    directory: TempDir,
}

impl TestProject {
    pub fn from_fixture(config: &str, schema: &str, analytics_schema: &str) -> Self {
        let directory = tempfile::tempdir().expect("temporary project should be created");
        let database_path = directory.path().join("app.db");
        let connection = Connection::open(&database_path).expect("test database should open");
        connection
            .execute_batch(schema)
            .expect("test schema should execute");
        let analytics_path = directory.path().join("analytics.db");
        let analytics =
            Connection::open(&analytics_path).expect("attached test database should open");
        analytics
            .execute_batch(analytics_schema)
            .expect("attached test schema should execute");
        fs::write(directory.path().join("dbmd.toml"), config)
            .expect("test config should be written");
        Self { directory }
    }

    pub fn request(&self) -> RenderRequest {
        RenderRequest::with_environment(
            self.directory.path().join("dbmd.toml"),
            BTreeMap::from([
                (
                    "DBMD_TEST_DATABASE".to_string(),
                    self.directory
                        .path()
                        .join("app.db")
                        .to_string_lossy()
                        .into_owned(),
                ),
                (
                    "DBMD_TEST_ANALYTICS_DATABASE".to_string(),
                    self.directory
                        .path()
                        .join("analytics.db")
                        .to_string_lossy()
                        .into_owned(),
                ),
            ]),
        )
    }

    pub fn output_path(&self) -> std::path::PathBuf {
        self.directory.path().join("DATABASE.md")
    }

    pub fn path(&self) -> &Path {
        self.directory.path()
    }
}
