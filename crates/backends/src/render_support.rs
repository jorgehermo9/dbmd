use std::fmt::Write as _;

use dbmd_render::{inline_code, text, RenderNamespace};

use crate::relational::{
    ConstraintKind, IndexNullsOrder, IndexSortOrder, IndexTarget, IndexTerm, Namespace,
};

pub(crate) fn namespaces(values: &[Namespace]) -> Vec<RenderNamespace> {
    values
        .iter()
        .map(|namespace| RenderNamespace {
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
