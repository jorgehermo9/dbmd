//! Shared infrastructure for real-database integration tests.

use std::{error::Error, thread, time::Duration};

use mysql::prelude::Queryable;
use postgres::{Client, NoTls};
use testcontainers_modules::{
    clickhouse::ClickHouse,
    mariadb::Mariadb,
    mysql::Mysql,
    postgres::Postgres,
    testcontainers::{
        core::{IntoContainerPort, WaitFor},
        runners::SyncRunner,
        Container, GenericImage, ImageExt,
    },
};

/// One locally owned MySQL 9.7.1 fixture container.
pub struct MysqlServer {
    _container: Container<Mysql>,
    url: String,
}

impl MysqlServer {
    /// Starts MySQL, executes `sql`, and waits until client connections succeed.
    pub fn start(sql: &str) -> Result<Self, TestError> {
        let container = Mysql::default()
            .with_init_sql(sql.as_bytes().to_vec())
            .with_tag("9.7.1")
            .with_startup_timeout(Duration::from_secs(180))
            .start()?;
        let url = mysql_family_url(&container)?;
        wait_for_mysql_family(&url)?;
        ensure_mysql_family_version(&url, "9.7.1")?;
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

/// One locally owned MariaDB 12.3.2 fixture container.
pub struct MariaDbServer {
    _container: Container<Mariadb>,
    url: String,
}

impl MariaDbServer {
    /// Starts MariaDB, executes `sql`, and waits until client connections succeed.
    pub fn start(sql: &str) -> Result<Self, TestError> {
        let container = Mariadb::default()
            .with_init_sql(sql.as_bytes().to_vec())
            .with_tag("12.3.2")
            .with_cmd(["--plugin-load-add=auth_mysql_sha2"])
            .with_startup_timeout(Duration::from_secs(180))
            .start()?;
        let url = mysql_family_url(&container)?;
        wait_for_mysql_family(&url)?;
        ensure_mysql_family_version(&url, "12.3.2-MariaDB")?;
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

fn ensure_mysql_family_version(url: &str, expected_prefix: &str) -> Result<(), TestError> {
    let mut connection = mysql::Conn::new(mysql::Opts::from_url(url)?)?;
    let version = connection
        .query_first::<String, _>("SELECT VERSION()")?
        .ok_or("database fixture returned no server version")?;
    if version.starts_with(expected_prefix) {
        Ok(())
    } else {
        Err(
            format!("database fixture version {version} does not match target {expected_prefix}")
                .into(),
        )
    }
}

/// One locally owned ClickHouse 26.6.1.1193 fixture container.
pub struct ClickHouseServer {
    _container: Container<ClickHouse>,
    endpoint: String,
}

impl ClickHouseServer {
    /// Starts ClickHouse and executes semicolon-delimited fixture statements.
    pub fn start(sql: &str) -> Result<Self, TestError> {
        Self::start_with_settings(sql, &[])
    }

    /// Starts ClickHouse and applies explicit query settings to every fixture statement.
    pub fn start_with_settings(sql: &str, settings: &[(&str, &str)]) -> Result<Self, TestError> {
        let container = ClickHouse::default()
            .with_tag("26.6.1.1193")
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
        server.ensure_version("26.6.1.1193")?;
        server.execute(sql, settings)?;
        Ok(server)
    }

    /// Returns the fixture HTTP endpoint.
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    fn execute(&self, sql: &str, settings: &[(&str, &str)]) -> Result<(), TestError> {
        let client = reqwest::blocking::Client::new();
        for statement in sql
            .split(";\n")
            .map(str::trim)
            .filter(|statement| !statement.is_empty())
        {
            let response = client
                .post(&self.endpoint)
                .query(settings)
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

    fn ensure_version(&self, expected_prefix: &str) -> Result<(), TestError> {
        let response = reqwest::blocking::Client::new()
            .post(&self.endpoint)
            .body("SELECT version() FORMAT TabSeparated")
            .send()?
            .error_for_status()?
            .text()?;
        let version = response.trim();
        if version.starts_with(expected_prefix) {
            Ok(())
        } else {
            Err(format!(
                "ClickHouse fixture version {version} does not match target {expected_prefix}"
            )
            .into())
        }
    }
}

/// Thread-safe erased error returned by shared integration-test helpers.
pub type TestError = Box<dyn Error + Send + Sync>;
/// Standard result type for one database fixture case.
pub type TestResult = Result<(), TestError>;

const CONFIG_EXTENSION_CONTROL: &str = r"comment = 'dbmd extension configuration fixture'
default_version = '1.0'
relocatable = true
";
const CONFIG_EXTENSION_SQL: &str = r#"
CREATE TYPE dbmd_extension_state AS ENUM ('enabled', 'disabled');
CREATE TYPE dbmd_extension_pair AS (left_value integer, right_value text);
CREATE DOMAIN dbmd_extension_positive AS integer CHECK (VALUE > 0);
CREATE TYPE dbmd_extension_range AS RANGE (SUBTYPE = integer);
CREATE SEQUENCE dbmd_extension_sequence;
CREATE COLLATION dbmd_extension_collation FROM "C";
CREATE TABLE dbmd_extension_config (
    id integer PRIMARY KEY,
    enabled boolean NOT NULL,
    state dbmd_extension_state NOT NULL DEFAULT 'enabled'
);
CREATE INDEX dbmd_extension_config_enabled_idx ON dbmd_extension_config (enabled);
CREATE VIEW dbmd_extension_enabled AS
SELECT id FROM dbmd_extension_config WHERE enabled;
CREATE FUNCTION dbmd_extension_identity(value integer)
RETURNS integer LANGUAGE sql IMMUTABLE RETURN value;
CREATE FUNCTION dbmd_extension_sum_state(state integer, value integer)
RETURNS integer LANGUAGE sql IMMUTABLE
RETURN coalesce(state, 0) + coalesce(value, 0);
CREATE AGGREGATE dbmd_extension_sum(integer) (
    SFUNC = dbmd_extension_sum_state,
    STYPE = integer,
    INITCOND = '0'
);
CREATE PROCEDURE dbmd_extension_noop()
LANGUAGE sql AS 'SELECT 1';
CREATE FUNCTION dbmd_extension_trigger()
RETURNS trigger LANGUAGE plpgsql AS 'BEGIN RETURN NEW; END';
CREATE TRIGGER dbmd_extension_config_trigger
BEFORE INSERT ON dbmd_extension_config
FOR EACH ROW EXECUTE FUNCTION dbmd_extension_trigger();
SELECT pg_catalog.pg_extension_config_dump('dbmd_extension_config', 'WHERE enabled');
"#;

/// One locally owned PostgreSQL container shared by a suite of fixture cases.
pub struct PostgresServer {
    _container: PostgresContainer,
    host: String,
    port: u16,
}

enum PostgresContainer {
    Core { _guard: Container<Postgres> },
    PgVector { _guard: Container<GenericImage> },
}

impl PostgresServer {
    /// Starts the pinned PostgreSQL image and retains its RAII container guard.
    pub fn start() -> Result<Self, TestError> {
        let container = Postgres::default().with_tag("18.4-alpine").start()?;
        let host = container.get_host()?.to_string();
        let port = container.get_host_port_ipv4(5432)?;
        let server = Self {
            _container: PostgresContainer::Core { _guard: container },
            host,
            port,
        };
        server.ensure_version("18.4")?;
        Ok(server)
    }

    /// Starts PostgreSQL 18.4 with the pinned pgvector 0.8.2 extension available.
    pub fn start_pgvector() -> Result<Self, TestError> {
        let container = GenericImage::new("pgvector/pgvector", "0.8.2-pg18")
            .with_exposed_port(5432.tcp())
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_copy_to(
                "/usr/share/postgresql/18/extension/dbmd_fixture.control",
                CONFIG_EXTENSION_CONTROL.as_bytes().to_vec(),
            )
            .with_copy_to(
                "/usr/share/postgresql/18/extension/dbmd_fixture--1.0.sql",
                CONFIG_EXTENSION_SQL.as_bytes().to_vec(),
            )
            .with_env_var("POSTGRES_DB", "postgres")
            .with_env_var("POSTGRES_USER", "postgres")
            .with_env_var("POSTGRES_PASSWORD", "postgres")
            .with_startup_timeout(Duration::from_secs(180))
            .start()?;
        let host = container.get_host()?.to_string();
        let port = container.get_host_port_ipv4(5432)?;
        let server = Self {
            _container: PostgresContainer::PgVector { _guard: container },
            host,
            port,
        };
        server.ensure_version("18.4")?;
        Ok(server)
    }

    fn ensure_version(&self, expected_prefix: &str) -> Result<(), TestError> {
        let mut client = Client::connect(&self.connection_string("postgres"), NoTls)?;
        let version = client
            .query_one("SELECT current_setting('server_version')", &[])?
            .get::<_, String>(0);
        if version.starts_with(expected_prefix) {
            Ok(())
        } else {
            Err(format!(
                "PostgreSQL fixture version {version} does not match target {expected_prefix}"
            )
            .into())
        }
    }

    /// Creates an isolated logical database and executes its fixture SQL.
    pub fn database(&self, sql: &str) -> Result<PostgresDatabase<'_>, TestError> {
        let name = format!("dbmd_test_{:016x}", fixture_hash(sql.as_bytes()));
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

fn fixture_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    })
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
