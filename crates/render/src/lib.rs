use dbmd_core::{SourceSnapshot, Table};
use minijinja::{context, Environment, UndefinedBehavior};
use serde_json::{json, Value};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("template error: {0}")]
    Template(#[from] minijinja::Error),
    #[error("failed to serialize render context: {0}")]
    Context(#[from] serde_json::Error),
    #[error("invalid internal render context: {0}")]
    InvalidContext(&'static str),
}

pub struct Renderer<'env> {
    env: Environment<'env>,
}

impl<'env> Renderer<'env> {
    pub fn embedded() -> Result<Self, RenderError> {
        let mut env = Environment::new();
        env.set_undefined_behavior(UndefinedBehavior::Strict);
        env.add_template(
            "database.md.j2",
            include_str!("../templates/database.md.j2"),
        )?;
        env.add_template("table.md.j2", include_str!("../templates/table.md.j2"))?;
        Ok(Self { env })
    }

    pub fn render_database(&self, source: &SourceSnapshot) -> Result<String, RenderError> {
        let tmpl = self.env.get_template("database.md.j2")?;
        let table_docs = source
            .tables
            .iter()
            .map(|table| self.render_table(table))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(tmpl.render(context! {
            name => source.display_name.as_deref().unwrap_or(source.id.as_str()),
            table_docs => table_docs,
            views => source.views,
            triggers => source.triggers,
            functions => source.functions,
        })?)
    }

    pub fn render_table(&self, table: &Table) -> Result<String, RenderError> {
        let tmpl = self.env.get_template("table.md.j2")?;
        Ok(tmpl.render(self.table_context(table)?)?)
    }

    fn table_context(&self, table: &Table) -> Result<Value, RenderError> {
        let mut value = serde_json::to_value(table)?;
        let object = value.as_object_mut().ok_or(RenderError::InvalidContext(
            "serialized table was not an object",
        ))?;
        object.insert("qualified_name".to_string(), json!(table.qualified_name()));

        if let dbmd_core::TableBackend::ClickHouse(clickhouse) = &table.backend {
            let backend = object
                .get_mut("backend")
                .and_then(Value::as_object_mut)
                .ok_or(RenderError::InvalidContext(
                    "serialized table backend was not an object",
                ))?;
            backend.insert(
                "engine_clause".to_string(),
                json!(clickhouse.engine_clause()),
            );
        }

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use dbmd_core::{
        ClickHouseColumn, ClickHouseTable, Column, ColumnBackend, Constraint, Index, Table,
        TableBackend,
    };

    use super::Renderer;

    #[test]
    fn renders_clickhouse_table_engine_details() {
        let mut settings = BTreeMap::new();
        settings.insert("index_granularity".to_string(), "8192".to_string());

        let table = Table {
            namespace: "analytics".to_string(),
            name: "events".to_string(),
            comment: Some("Raw event stream with deduplication".to_string()),
            columns: vec![Column {
                name: "event_type".to_string(),
                data_type: "LowCardinality(String)".to_string(),
                nullable: Some(false),
                default: None,
                comment: Some("Values: page_view, click, purchase".to_string()),
                backend: ColumnBackend::ClickHouse(ClickHouseColumn {
                    codec: None,
                    ttl: None,
                }),
            }],
            constraints: Vec::<Constraint>::new(),
            indexes: Vec::<Index>::new(),
            backend: TableBackend::ClickHouse(ClickHouseTable {
                engine: "ReplacingMergeTree".to_string(),
                engine_params: vec!["version".to_string(), "is_deleted".to_string()],
                order_by: vec!["user_id".to_string(), "occurred_at".to_string()],
                partition_by: Some("toYYYYMM(occurred_at)".to_string()),
                primary_key: vec!["user_id".to_string()],
                sample_by: None,
                ttl: Some("occurred_at + INTERVAL 90 DAY".to_string()),
                settings,
            }),
        };

        let out = Renderer::embedded().unwrap().render_table(&table).unwrap();

        assert!(out.contains("## `analytics.events`"));
        assert!(out.contains("`ReplacingMergeTree(version, is_deleted)`"));
        assert!(out.contains("**Primary key:** `user_id`"));
        assert!(out.contains("**Partition by:** `toYYYYMM(occurred_at)`"));
        assert!(out.contains("`index_granularity = 8192`"));
    }
}
