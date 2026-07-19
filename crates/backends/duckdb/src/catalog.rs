use dbmd_core::SourceSnapshot;
use serde::Serialize;

pub type Snapshot = SourceSnapshot<Catalog>;

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Database {
    pub name: String,
    pub path: Option<String>,
    pub comment: Option<String>,
    pub database_type: String,
    pub readonly: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Schema {
    pub database: String,
    pub name: String,
    pub comment: Option<String>,
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
    pub nullable: bool,
    pub default: Option<String>,
    pub generated_expression: Option<String>,
    pub comment: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Constraint {
    pub catalog_index: u64,
    pub kind: String,
    pub text: String,
    pub expression: Option<String>,
    pub columns: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Index {
    pub name: String,
    pub unique: bool,
    pub primary: bool,
    pub expressions: String,
    pub comment: Option<String>,
    pub definition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct View {
    pub database: String,
    pub schema: String,
    pub name: String,
    pub comment: Option<String>,
    pub temporary: bool,
    pub definition: String,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Type {
    pub database: String,
    pub schema: String,
    pub name: String,
    pub logical_type: String,
    pub category: Option<String>,
    pub labels: Vec<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Function {
    pub database: String,
    pub schema: String,
    pub name: String,
    pub kind: String,
    pub description: Option<String>,
    pub comment: Option<String>,
    pub return_type: Option<String>,
    pub parameters: Vec<String>,
    pub parameter_types: Vec<String>,
    pub varargs: Option<String>,
    pub definition: Option<String>,
    pub side_effects: Option<bool>,
    pub stability: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Extension {
    pub name: String,
    pub loaded: bool,
    pub installed: bool,
    pub version: Option<String>,
    pub description: Option<String>,
}
