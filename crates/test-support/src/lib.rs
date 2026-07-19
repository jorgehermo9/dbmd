//! Shared infrastructure for real-database integration tests.

use std::{
    error::Error,
    process,
    sync::atomic::{AtomicU64, Ordering},
    thread,
    time::Duration,
};

use postgres::{Client, NoTls};
use testcontainers_modules::{
    clickhouse::ClickHouse,
    mariadb::Mariadb,
    mysql::Mysql,
    postgres::Postgres,
    testcontainers::{runners::SyncRunner, Container, ImageExt},
};

/// One locally owned MySQL 8.4 fixture container.
pub struct MysqlServer {
    _container: Container<Mysql>,
    url: String,
}

impl MysqlServer {
    /// Starts MySQL, executes `sql`, and waits until client connections succeed.
    pub fn start(sql: &str) -> Result<Self, TestError> {
        let container = Mysql::default()
            .with_init_sql(sql.as_bytes().to_vec())
            .with_tag("8.4")
            .start()?;
        let url = mysql_family_url(&container)?;
        wait_for_mysql_family(&url)?;
        Ok(Self {
            _container: container,
            url,
        })
    }

    /// Returns the root fixture URL for the `test` schema.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// One locally owned MariaDB 11.8 fixture container.
pub struct MariaDbServer {
    _container: Container<Mariadb>,
    url: String,
}

impl MariaDbServer {
    /// Starts MariaDB, executes `sql`, and waits until client connections succeed.
    pub fn start(sql: &str) -> Result<Self, TestError> {
        let container = Mariadb::default()
            .with_init_sql(sql.as_bytes().to_vec())
            .with_tag("11.8")
            .start()?;
        let url = mysql_family_url(&container)?;
        wait_for_mysql_family(&url)?;
        Ok(Self {
            _container: container,
            url,
        })
    }

    /// Returns the root fixture URL for the `test` schema.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }
}

fn mysql_family_url<I>(container: &Container<I>) -> Result<String, TestError>
where
    I: testcontainers_modules::testcontainers::Image,
{
    let host = container.get_host()?.to_string();
    let host = if host == "localhost" {
        "127.0.0.1"
    } else {
        &host
    };
    Ok(format!(
        "mysql://root@{host}:{}/test",
        container.get_host_port_ipv4(3306)?
    ))
}

fn wait_for_mysql_family(url: &str) -> Result<(), TestError> {
    let options = mysql::Opts::from_url(url)?;
    let options =
        mysql::OptsBuilder::from_opts(options).tcp_connect_timeout(Some(Duration::from_secs(1)));
    for _ in 0..60 {
        if mysql::Conn::new(options.clone()).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }
    Err("database fixture did not accept MySQL-protocol connections".into())
}

/// One locally owned ClickHouse 25.8 fixture container.
pub struct ClickHouseServer {
    _container: Container<ClickHouse>,
    endpoint: String,
}

impl ClickHouseServer {
    /// Starts ClickHouse and executes semicolon-delimited fixture statements.
    pub fn start(sql: &str) -> Result<Self, TestError> {
        let container = ClickHouse::default()
            .with_tag("25.8")
            .with_env_var("CLICKHOUSE_SKIP_USER_SETUP", "1")
            .start()?;
        let endpoint = format!(
            "http://{}:{}",
            container.get_host()?,
            container.get_host_port_ipv4(8123)?
        );
        let server = Self {
            _container: container,
            endpoint,
        };
        server.execute(sql)?;
        Ok(server)
    }

    /// Returns the fixture HTTP endpoint.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn execute(&self, sql: &str) -> Result<(), TestError> {
        let client = reqwest::blocking::Client::new();
        for statement in sql
            .split(";\n")
            .map(str::trim)
            .filter(|statement| !statement.is_empty())
        {
            let response = client
                .post(&self.endpoint)
                .body(statement.to_string())
                .send()?;
            let status = response.status();
            let body = response.text()?;
            if !status.is_success() {
                return Err(format!("ClickHouse fixture failed: {body}").into());
            }
        }
        Ok(())
    }
}

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
