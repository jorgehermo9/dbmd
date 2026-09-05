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

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! semantic_cases {
        ($($name:ident: $value:expr => $display:literal, $serialized:literal;)+) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!($value.display_name(), $display);
                    assert_eq!(
                        serde_json::to_string(&$value).expect("semantic enum should serialize"),
                        $serialized
                    );
                }
            )+
        };
    }

    semantic_cases! {
        presents_check_constraint: ConstraintKind::Check => "check", "\"check\"";
        presents_foreign_key_constraint: ConstraintKind::ForeignKey => "foreign key", "\"foreign_key\"";
        presents_primary_key_constraint: ConstraintKind::PrimaryKey => "primary key", "\"primary_key\"";
        presents_not_null_constraint: ConstraintKind::NotNull => "not null", "\"not_null\"";
        presents_unique_constraint: ConstraintKind::Unique => "unique", "\"unique\"";
        presents_table_function: FunctionKind::Table => "table function", "\"table\"";
        presents_scalar_function: FunctionKind::Scalar => "scalar function", "\"scalar\"";
        presents_aggregate_function: FunctionKind::Aggregate => "aggregate function", "\"aggregate\"";
        presents_pragma_function: FunctionKind::Pragma => "pragma", "\"pragma\"";
        presents_macro_function: FunctionKind::Macro => "macro", "\"macro\"";
        presents_table_macro_function: FunctionKind::TableMacro => "table macro", "\"table_macro\"";
        presents_consistent_stability: FunctionStability::Consistent => "consistent", "\"consistent\"";
        presents_volatile_stability: FunctionStability::Volatile => "volatile", "\"volatile\"";
        presents_query_consistent_stability: FunctionStability::ConsistentWithinQuery => "consistent within query", "\"consistent_within_query\"";
        presents_unknown_install_mode: ExtensionInstallMode::Unknown => "unknown", "\"unknown\"";
        presents_repository_install_mode: ExtensionInstallMode::Repository => "repository", "\"repository\"";
        presents_custom_path_install_mode: ExtensionInstallMode::CustomPath => "custom path", "\"custom_path\"";
        presents_statically_linked_install_mode: ExtensionInstallMode::StaticallyLinked => "statically linked", "\"statically_linked\"";
        presents_not_installed_mode: ExtensionInstallMode::NotInstalled => "not installed", "\"not_installed\"";
    }
}
