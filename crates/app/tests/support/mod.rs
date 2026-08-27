//! Reusable project setup for application integration tests.

#![allow(
    dead_code,
    reason = "each integration-test binary compiles this shared module independently"
)]

use std::{collections::BTreeMap, path::Path};

use dbmd_app::RenderRequest;
use dbmd_test_support::TestProject as ProjectFixture;
use rusqlite::Connection;

pub struct TestProject {
    project: ProjectFixture,
    environment: BTreeMap<String, String>,
}

impl TestProject {
    pub fn new() -> Self {
        Self {
            project: ProjectFixture::new(),
            environment: BTreeMap::new(),
        }
    }

    pub fn from_sqlite_fixture(config: &str, schema: &str, analytics_schema: &str) -> Self {
        let mut project = Self::new();
        let database_path = project.path().join("app.db");
        let connection = Connection::open(&database_path).expect("test database should open");
        connection
            .execute_batch(schema)
            .expect("test schema should execute");
        let analytics_path = project.path().join("analytics.db");
        let analytics =
            Connection::open(&analytics_path).expect("attached test database should open");
        analytics
            .execute_batch(analytics_schema)
            .expect("attached test schema should execute");
        project.write("dbmd.toml", config);
        project.environment.extend([
            (
                "DBMD_TEST_DATABASE".to_string(),
                database_path.to_string_lossy().into_owned(),
            ),
            (
                "DBMD_TEST_ANALYTICS_DATABASE".to_string(),
                analytics_path.to_string_lossy().into_owned(),
            ),
        ]);
        project
    }

    pub fn request(&self) -> RenderRequest {
        RenderRequest::with_environment(self.path().join("dbmd.toml"), self.environment())
    }

    pub fn environment(&self) -> BTreeMap<String, String> {
        self.environment.clone()
    }

    pub fn output_path(&self) -> std::path::PathBuf {
        self.path().join("DATABASE.md")
    }

    pub fn path(&self) -> &Path {
        self.project.root()
    }

    pub fn write(
        &self,
        relative: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> std::path::PathBuf {
        self.project.write(relative, contents)
    }

    pub fn config(&self, contents: impl AsRef<[u8]>) -> std::path::PathBuf {
        self.write("dbmd.toml", contents)
    }

    pub fn sqlite(&self, relative: impl AsRef<Path>, sql: &str) -> std::path::PathBuf {
        let path = self.project.path(relative);
        Connection::open(&path)
            .expect("SQLite application fixture should open")
            .execute_batch(sql)
            .expect("SQLite application fixture should execute");
        path
    }

    pub fn create_dir(&self, relative: impl AsRef<Path>) -> std::path::PathBuf {
        self.project.create_dir(relative)
    }
}

impl Default for TestProject {
    fn default() -> Self {
        Self::new()
    }
}
