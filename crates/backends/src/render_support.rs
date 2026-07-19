use std::fmt::Write as _;

use dbmd_render::{inline_code, text, RenderObject};
use serde::Serialize;

use crate::relational::{
    ConstraintKind, IndexNullsOrder, IndexSortOrder, IndexTarget, IndexTerm, Namespace,
};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct NamespaceView {
    pub name: String,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct EnumView {
    pub qualified_name: String,
    pub file_name: String,
    pub comment: Option<String>,
    pub values: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TableView {
    pub qualified_name: String,
    pub file_name: String,
    pub comment: Option<String>,
    pub columns: Vec<ColumnView>,
    pub constraints: Vec<ConstraintView>,
    pub indexes: Vec<IndexView>,
    pub backend: TableDetailsView,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ColumnView {
    pub name: String,
    pub data_type: String,
    pub nullable: &'static str,
    pub default: String,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ConstraintView {
    pub name: String,
    pub kind: String,
    pub columns: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct IndexView {
    pub name: String,
    pub terms: String,
    pub unique: &'static str,
    pub origin: String,
    pub predicate: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TableDetailsView {
    pub title: &'static str,
    pub facts: Vec<FactView>,
    pub notices: Vec<&'static str>,
    pub definition: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FactView {
    pub label: &'static str,
    pub value: String,
}

impl FactView {
    pub fn new(label: &'static str, value: impl Into<String>) -> Self {
        Self {
            label,
            value: value.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ViewPresentation {
    pub qualified_name: String,
    pub file_name: String,
    pub comment: Option<String>,
    pub facts: Vec<FactView>,
    pub columns: Vec<ColumnView>,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct TriggerView {
    pub qualified_name: String,
    pub file_name: String,
    pub comment: Option<String>,
    pub event: String,
    pub target: String,
    pub facts: Vec<FactView>,
    pub when_expression: Option<String>,
    pub definition: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FunctionView {
    pub qualified_name: String,
    pub file_name: String,
    pub comment: Option<String>,
    pub facts: Vec<FactView>,
    pub definition: Option<String>,
}

pub(crate) fn directory_object(
    family: &str,
    template: &'static str,
    file_name: &str,
    data: impl Serialize,
) -> RenderObject {
    RenderObject::new(format!("{family}/{file_name}"), template, data)
}

pub(crate) fn namespaces(values: &[Namespace]) -> Vec<NamespaceView> {
    values
        .iter()
        .map(|namespace| NamespaceView {
            name: inline_code(&namespace.name),
            comment: namespace.comment.as_deref().map(text),
        })
        .collect()
}

pub(crate) const fn nullable(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "yes",
        Some(false) => "no",
        None => "unknown",
    }
}

pub(crate) const fn constraint_kind(kind: &ConstraintKind) -> &'static str {
    match kind {
        ConstraintKind::PrimaryKey => "primary_key",
        ConstraintKind::ForeignKey => "foreign_key",
        ConstraintKind::Unique => "unique",
        ConstraintKind::Check => "check",
        ConstraintKind::NotNull => "not_null",
        ConstraintKind::Exclusion => "exclusion",
    }
}

pub(crate) fn index_terms(terms: &[IndexTerm]) -> String {
    terms
        .iter()
        .map(|term| {
            let target = match &term.target {
                IndexTarget::Column(value) | IndexTarget::Expression(value) => inline_code(value),
                IndexTarget::RowId => inline_code("rowid"),
            };
            let mut rendered = format!(
                "{target} {}",
                match term.order {
                    IndexSortOrder::Ascending => "ascending",
                    IndexSortOrder::Descending => "descending",
                }
            );
            if let Some(collation) = &term.collation {
                let _ = write!(rendered, " collate {}", inline_code(collation));
            }
            if let Some(operator_class) = &term.operator_class {
                let _ = write!(rendered, " opclass {}", inline_code(operator_class));
            }
            if let Some(nulls_order) = term.nulls_order {
                let _ = write!(
                    rendered,
                    " nulls {}",
                    inline_code(match nulls_order {
                        IndexNullsOrder::First => "first",
                        IndexNullsOrder::Last => "last",
                    })
                );
            }
            rendered
        })
        .collect::<Vec<_>>()
        .join(", ")
}
