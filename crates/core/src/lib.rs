use std::collections::BTreeMap;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DatabaseSchema {
    pub name: String,
    pub tables: Vec<Table>,
    pub views: Vec<View>,
    pub functions: Vec<Function>,
}

impl DatabaseSchema {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tables: Vec::new(),
            views: Vec::new(),
            functions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub comment: Option<String>,
    pub columns: Vec<Column>,
    pub constraints: Vec<Constraint>,
    pub indexes: Vec<Index>,
    pub engine: TableEngine,
}

impl Table {
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub comment: Option<String>,
    pub backend: ColumnBackend,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum ColumnBackend {
    Common,
    Postgres(PostgresColumn),
    ClickHouse(ClickHouseColumn),
    Sqlite(SqliteColumn),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresColumn {
    pub enum_values: Vec<String>,
    pub identity: Option<String>,
    pub generated: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClickHouseColumn {
    pub codec: Option<String>,
    pub ttl: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SqliteColumn {
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Constraint {
    pub name: Option<String>,
    pub kind: ConstraintKind,
    pub columns: Vec<String>,
    pub expression: Option<String>,
    pub references: Option<ForeignKeyReference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintKind {
    PrimaryKey,
    ForeignKey,
    Unique,
    Check,
    Exclusion,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForeignKeyReference {
    pub schema: String,
    pub table: String,
    pub columns: Vec<String>,
    pub on_update: Option<String>,
    pub on_delete: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Index {
    pub name: String,
    pub kind: Option<String>,
    pub columns: Vec<String>,
    pub expression: Option<String>,
    pub unique: bool,
    pub backend: IndexBackend,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum IndexBackend {
    Common,
    Postgres(PostgresIndex),
    ClickHouse(ClickHouseIndex),
    Sqlite(SqliteIndex),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresIndex {
    pub method: String,
    pub predicate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClickHouseIndex {
    pub granularity: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SqliteIndex {
    pub origin: Option<String>,
    pub partial: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "backend", rename_all = "snake_case")]
pub enum TableEngine {
    Postgres(PostgresTable),
    ClickHouse(ClickHouseTable),
    Sqlite(SqliteTable),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PostgresTable {
    pub table_kind: PostgresTableKind,
    pub tablespace: Option<String>,
    pub inherits: Vec<String>,
    pub partition: Option<String>,
    pub row_level_security: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PostgresTableKind {
    Table,
    PartitionedTable,
    ForeignTable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClickHouseTable {
    pub engine: String,
    pub engine_params: Vec<String>,
    pub order_by: Vec<String>,
    pub partition_by: Option<String>,
    pub primary_key: Vec<String>,
    pub sample_by: Option<String>,
    pub ttl: Option<String>,
    pub settings: BTreeMap<String, String>,
}

impl ClickHouseTable {
    pub fn engine_clause(&self) -> String {
        if self.engine_params.is_empty() {
            self.engine.clone()
        } else {
            format!("{}({})", self.engine, self.engine_params.join(", "))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SqliteTable {
    pub without_rowid: bool,
    pub strict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct View {
    pub schema: String,
    pub name: String,
    pub definition: String,
    pub materialized: bool,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Function {
    pub schema: String,
    pub name: String,
    pub signature: String,
    pub definition: Option<String>,
    pub comment: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_clickhouse_engine_clause() {
        let table = ClickHouseTable {
            engine: "ReplacingMergeTree".to_string(),
            engine_params: vec!["version".to_string(), "is_deleted".to_string()],
            order_by: vec!["user_id".to_string(), "occurred_at".to_string()],
            partition_by: Some("toYYYYMM(occurred_at)".to_string()),
            primary_key: vec!["user_id".to_string()],
            sample_by: None,
            ttl: None,
            settings: BTreeMap::new(),
        };

        assert_eq!(
            table.engine_clause(),
            "ReplacingMergeTree(version, is_deleted)"
        );
    }
}
