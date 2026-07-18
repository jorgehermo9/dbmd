//! Shared infrastructure for real-database integration tests.

use std::{
    error::Error,
    process,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use postgres::{Client, NoTls};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::SyncRunner, Container, ImageExt},
};

/// Thread-safe erased error returned by shared integration-test helpers.
pub type TestError = Box<dyn Error + Send + Sync>;
/// Standard result type for one database fixture case.
pub type TestResult = Result<(), TestError>;

static NEXT_DATABASE: AtomicU64 = AtomicU64::new(1);

/// One locally owned PostgreSQL container shared by a suite of fixture cases.
pub struct PostgresServer {
    _container: Container<Postgres>,
    host: String,
    port: u16,
}

impl PostgresServer {
    /// Starts the pinned PostgreSQL image and retains its RAII container guard.
    pub fn start() -> Result<Self, TestError> {
        let container = Postgres::default().with_tag("17-alpine").start()?;
        let host = container.get_host()?.to_string();
        let port = container.get_host_port_ipv4(5432)?;
        Ok(Self {
            _container: container,
            host,
            port,
        })
    }

    /// Creates an isolated logical database and executes its fixture SQL.
    pub fn database(&self, sql: &str) -> Result<PostgresDatabase<'_>, TestError> {
        let sequence = NEXT_DATABASE.fetch_add(1, Ordering::Relaxed);
        let name = format!("dbmd_test_{}_{}", process::id(), sequence);
        let mut admin = Client::connect(&self.connection_string("postgres"), NoTls)?;
        admin.batch_execute(&format!("CREATE DATABASE {name}"))?;

        let database = PostgresDatabase { server: self, name };
        let mut client = database.connect()?;
        client.batch_execute(sql)?;
        Ok(database)
    }

    fn connection_string(&self, database: &str) -> String {
        format!(
            "postgres://postgres:postgres@{}:{}/{database}",
            self.host, self.port
        )
    }
}

/// One per-case database dropped forcibly when its fixture guard leaves scope.
pub struct PostgresDatabase<'a> {
    server: &'a PostgresServer,
    name: String,
}

impl PostgresDatabase<'_> {
    /// Opens a client connected to this fixture's isolated logical database.
    ///
    /// # Errors
    ///
    /// Returns the PostgreSQL driver error when the fixture database cannot be reached.
    pub fn connect(&self) -> Result<Client, postgres::Error> {
        Client::connect(&self.connection_string(), NoTls)
    }

    #[must_use]
    /// Returns a connection URL scoped to this fixture database.
    pub fn connection_string(&self) -> String {
        self.server.connection_string(&self.name)
    }
}

impl Drop for PostgresDatabase<'_> {
    fn drop(&mut self) {
        match Client::connect(&self.server.connection_string("postgres"), NoTls) {
            Ok(mut admin) => {
                if let Err(error) = admin.batch_execute(&format!(
                    "DROP DATABASE IF EXISTS {} WITH (FORCE)",
                    self.name
                )) {
                    eprintln!(
                        "failed to drop PostgreSQL fixture database `{}`: {error}",
                        self.name
                    );
                }
            }
            Err(error) => eprintln!(
                "failed to connect for PostgreSQL fixture cleanup `{}`: {error}",
                self.name
            ),
        }
    }
}

/// One independently isolated fixture case in a shared-container suite.
#[derive(Clone, Copy)]
pub struct PostgresCase {
    /// Stable case name used in aggregated failure reports.
    pub name: &'static str,
    /// Case function executed against the shared server.
    pub run: fn(&PostgresServer) -> TestResult,
}

/// Runs all fixture cases concurrently against one locally owned container.
pub fn run_postgres_cases(cases: &[PostgresCase]) {
    let server = PostgresServer::start().expect("shared PostgreSQL test container should start");
    let failures = thread::scope(|scope| {
        let handles = cases
            .iter()
            .map(|case| {
                let handle = scope.spawn(|| (case.run)(&server));
                (case.name, handle)
            })
            .collect::<Vec<_>>();

        handles
            .into_iter()
            .filter_map(|(name, handle)| match handle.join() {
                Ok(Ok(())) => None,
                Ok(Err(error)) => Some(format!(
                    "{name}: {}\n  debug: {error:?}",
                    error_chain(error.as_ref())
                )),
                Err(_) => Some(format!("{name}: panicked")),
            })
            .collect::<Vec<_>>()
    });

    assert!(
        failures.is_empty(),
        "PostgreSQL fixture failures:\n{}",
        failures.join("\n")
    );
}

fn error_chain(error: &(dyn Error + 'static)) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(error) = source {
        message.push_str("\n  caused by: ");
        message.push_str(&error.to_string());
        source = error.source();
    }
    message
}
