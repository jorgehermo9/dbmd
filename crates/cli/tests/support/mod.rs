//! Process and project harness for CLI end-to-end tests.

use std::{
    path::Path,
    process::{Command, Output},
};

use dbmd_test_support::TestProject;

pub struct CliProject {
    project: TestProject,
}

impl CliProject {
    pub fn new() -> Self {
        Self {
            project: TestProject::new(),
        }
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

    pub fn sqlite(&self, relative: impl AsRef<Path>, sql: &str) -> std::path::PathBuf {
        let path = self.project.path(relative);
        rusqlite::Connection::open(&path)
            .expect("SQLite CLI fixture should open")
            .execute_batch(sql)
            .expect("SQLite CLI fixture should execute");
        path
    }

    pub fn duckdb(&self, relative: impl AsRef<Path>, sql: &str) -> std::path::PathBuf {
        let path = self.project.path(relative);
        duckdb::Connection::open(&path)
            .expect("DuckDB CLI fixture should open")
            .execute_batch(sql)
            .expect("DuckDB CLI fixture should execute");
        path
    }

    pub fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_dbmd"));
        command.current_dir(self.path());
        command
    }

    pub fn run<I, S>(&self, arguments: I) -> Output
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.command()
            .args(arguments)
            .output()
            .expect("dbmd command should execute")
    }
}

impl Default for CliProject {
    fn default() -> Self {
        Self::new()
    }
}
