use dbmd_core::SourceSnapshot;
use serde::Serialize;

pub type Snapshot = SourceSnapshot<Catalog>;

macro_rules! semantic_enum {
    ($(#[$meta:meta])* pub enum $name:ident { $($variant:ident => $label:literal),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
        #[serde(rename_all = "snake_case")]
        #[non_exhaustive]
        pub enum $name { $($variant),+ }

        impl $name {
            #[must_use]
            pub const fn display_name(self) -> &'static str {
                match self { $(Self::$variant => $label),+ }
            }
        }
    };
}

semantic_enum! {
    /// Constraint family reported by `duckdb_constraints()`.
    pub enum ConstraintKind {
        Check => "check",
        ForeignKey => "foreign key",
        PrimaryKey => "primary key",
        NotNull => "not null",
        Unique => "unique"
    }
}

semantic_enum! {
    /// Function family reported by `duckdb_functions()`.
    pub enum FunctionKind {
        Table => "table function",
        Scalar => "scalar function",
        Aggregate => "aggregate function",
        Pragma => "pragma",
        Macro => "macro",
        TableMacro => "table macro"
    }
}

semantic_enum! {
    /// Evaluation stability reported by `duckdb_functions()`.
    pub enum FunctionStability {
        Consistent => "consistent",
        Volatile => "volatile",
        ConsistentWithinQuery => "consistent within query"
    }
}

semantic_enum! {
    /// Installation provenance reported by `duckdb_extensions()`.
    pub enum ExtensionInstallMode {
        Unknown => "unknown",
        Repository => "repository",
        CustomPath => "custom path",
        StaticallyLinked => "statically linked",
        NotInstalled => "not installed"
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Catalog {
    pub databases: Vec<Database>,
    pub schemas: Vec<Schema>,
    pub tables: Vec<Table>,
    pub views: Vec<View>,
    pub sequences: Vec<Sequence>,
    pub types: Vec<Type>,
    pub functions: Vec<Function>,
    pub extensions: Vec<Extension>,
    pub secrets: Vec<Secret>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Database {
    pub name: String,
    pub path: Option<String>,
    pub comment: Option<String>,
    pub database_type: String,
    pub readonly: bool,
    /// User/catalog metadata tags in deterministic key order.
    pub tags: std::collections::BTreeMap<String, String>,
    /// Non-secret retained attach options in deterministic key order.
    pub options: std::collections::BTreeMap<String, String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Schema {
    pub database: String,
    pub name: String,
    pub comment: Option<String>,
    /// User/catalog metadata tags in deterministic key order.
    pub tags: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Table {
    pub database: String,
    pub schema: String,
    pub name: String,
    pub comment: Option<String>,
    pub temporary: bool,
    pub columns: Vec<Column>,
    pub constraints: Vec<Constraint>,
    pub indexes: Vec<Index>,
    pub definition: String,
    /// User/catalog metadata tags in deterministic key order.
    pub tags: std::collections::BTreeMap<String, String>,
}
impl Table {
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.database, self.schema, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Column {
    pub name: String,
    pub position: u64,
    pub data_type: String,
    pub numeric_precision: Option<u64>,
    pub numeric_precision_radix: Option<u64>,
    pub numeric_scale: Option<u64>,
    pub nullable: bool,
    pub default: Option<String>,
    pub generated_expression: Option<String>,
    pub comment: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Constraint {
    pub catalog_index: u64,
    pub name: String,
    pub kind: ConstraintKind,
    pub text: String,
    pub expression: Option<String>,
    pub columns: Vec<String>,
    pub referenced_table: Option<String>,
    pub referenced_columns: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Index {
    pub name: String,
    pub index_type: String,
    pub unique: bool,
    pub primary: bool,
    pub expressions: String,
    pub comment: Option<String>,
    pub definition: String,
    /// User/catalog metadata tags in deterministic key order.
    pub tags: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct View {
    pub database: String,
    pub schema: String,
    pub name: String,
    pub comment: Option<String>,
    pub temporary: bool,
    pub definition: String,
    /// User/catalog metadata tags in deterministic key order.
    pub tags: std::collections::BTreeMap<String, String>,
}
impl View {
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}.{}", self.database, self.schema, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Sequence {
    pub database: String,
    pub schema: String,
    pub name: String,
    pub comment: Option<String>,
    pub temporary: bool,
    pub start: i64,
    pub minimum: i64,
    pub maximum: i64,
    pub increment: i64,
    pub cycle: bool,
    pub definition: Option<String>,
    /// User/catalog metadata tags in deterministic key order.
    pub tags: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Type {
    pub database: String,
    pub schema: String,
    pub name: String,
    pub logical_type: String,
    pub definition: String,
    pub size: Option<u64>,
    pub category: Option<String>,
    pub labels: Vec<String>,
    pub comment: Option<String>,
    /// User/catalog metadata tags in deterministic key order.
    pub tags: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Function {
    pub database: String,
    pub schema: String,
    pub name: String,
    pub kind: FunctionKind,
    pub description: Option<String>,
    pub comment: Option<String>,
    pub return_type: Option<String>,
    pub parameters: Vec<String>,
    pub parameter_types: Vec<String>,
    pub varargs: Option<String>,
    pub definition: Option<String>,
    pub side_effects: Option<bool>,
    pub stability: Option<FunctionStability>,
    /// User/catalog metadata tags in deterministic key order.
    pub tags: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Extension {
    pub name: String,
    pub loaded: bool,
    pub installed: bool,
    pub version: Option<String>,
    pub description: Option<String>,
    pub aliases: Vec<String>,
    pub install_mode: Option<ExtensionInstallMode>,
    pub installed_from: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Secret {
    pub name: String,
    pub secret_type: String,
    pub provider: String,
    pub persistent: bool,
    pub storage: String,
    pub scope: Vec<String>,
}
