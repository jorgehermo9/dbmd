use dbmd_core::SourceSnapshot;
use serde::Serialize;

pub type Snapshot = SourceSnapshot<Catalog>;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Catalog {
    pub schemas: Vec<Schema>,
    pub tables: Vec<Table>,
    pub views: Vec<View>,
    pub routines: Vec<Routine>,
    pub triggers: Vec<Trigger>,
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Schema {
    pub name: String,
    pub default_character_set: String,
    pub default_collation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub engine: Option<String>,
    pub row_format: Option<String>,
    pub collation: Option<String>,
    pub comment: Option<String>,
    pub create_options: Option<String>,
    pub columns: Vec<Column>,
    pub constraints: Vec<Constraint>,
    pub indexes: Vec<Index>,
    pub partitions: Vec<Partition>,
    pub definition: String,
}

impl Table {
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Column {
    pub name: String,
    pub position: u64,
    pub data_type: String,
    pub column_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub extra: String,
    pub generation_expression: Option<String>,
    pub character_set: Option<String>,
    pub collation: Option<String>,
    pub comment: Option<String>,
    pub visible: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ConstraintKind {
    PrimaryKey,
    Unique,
    ForeignKey,
    Check,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Constraint {
    pub name: String,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    pub referenced_schema: Option<String>,
    pub referenced_table: Option<String>,
    pub referenced_columns: Vec<String>,
    pub match_option: Option<String>,
    pub update_rule: Option<String>,
    pub delete_rule: Option<String>,
    pub expression: Option<String>,
    pub enforced: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Index {
    pub name: String,
    pub unique: bool,
    pub index_type: String,
    pub visible: Option<bool>,
    pub comment: Option<String>,
    pub terms: Vec<IndexTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct IndexTerm {
    pub position: u64,
    pub column: Option<String>,
    pub expression: Option<String>,
    pub prefix_length: Option<u64>,
    pub descending: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Partition {
    pub name: String,
    pub subpartition: Option<String>,
    pub method: Option<String>,
    pub expression: Option<String>,
    pub description: Option<String>,
    pub ordinal: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct View {
    pub schema: String,
    pub name: String,
    pub definition: String,
    pub check_option: String,
    pub updatable: bool,
    pub security_type: String,
    pub definer: String,
    pub character_set: String,
    pub collation: String,
    pub create_statement: String,
}

impl View {
    #[must_use]
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Routine {
    pub schema: String,
    pub name: String,
    pub kind: String,
    pub return_type: Option<String>,
    pub body: String,
    pub definition: Option<String>,
    pub deterministic: bool,
    pub sql_data_access: String,
    pub security_type: String,
    pub definer: String,
    pub comment: Option<String>,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Parameter {
    pub position: u64,
    pub mode: Option<String>,
    pub name: Option<String>,
    pub data_type: String,
    pub dtd_identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Trigger {
    pub schema: String,
    pub name: String,
    pub table: String,
    pub event: String,
    pub timing: String,
    pub orientation: String,
    pub statement: String,
    pub action_order: u64,
    pub sql_mode: String,
    pub definer: String,
    pub character_set: String,
    pub collation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct Event {
    pub schema: String,
    pub name: String,
    pub definer: String,
    pub time_zone: String,
    pub event_type: String,
    pub execute_at: Option<String>,
    pub interval_value: Option<String>,
    pub interval_field: Option<String>,
    pub starts: Option<String>,
    pub ends: Option<String>,
    pub status: String,
    pub on_completion: String,
    pub comment: Option<String>,
    pub definition: String,
}
