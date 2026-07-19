//! Shared Markdown-ready presentation values for backend render preparation.

use dbmd_render::{inline_code, text, RenderObject};
use serde::Serialize;

use crate::Namespace;

/// A namespace prepared for a shared namespace template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct NamespaceView {
    /// Markdown-ready namespace name.
    pub name: String,
    /// Optional Markdown-ready comment.
    pub comment: Option<String>,
}

impl NamespaceView {
    /// Creates a prepared namespace.
    #[must_use]
    pub fn new(name: String, comment: Option<String>) -> Self {
        Self { name, comment }
    }
}

/// A table prepared for shared table templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct TableView {
    /// Markdown-ready qualified name.
    pub qualified_name: String,
    /// Safe object filename.
    pub file_name: String,
    /// Optional Markdown-ready comment.
    pub comment: Option<String>,
    /// Columns in catalog order.
    pub columns: Vec<ColumnView>,
    /// Constraints in deterministic backend order.
    pub constraints: Vec<ConstraintView>,
    /// Indexes in deterministic backend order.
    pub indexes: Vec<IndexView>,
    /// Backend-specific table facts.
    pub backend: TableDetailsView,
}

impl TableView {
    /// Starts a prepared-table builder.
    #[must_use]
    pub fn builder() -> TableViewBuilder {
        TableViewBuilder::default()
    }
}

/// Builder for [`TableView`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableViewBuilder {
    qualified_name: Option<String>,
    file_name: Option<String>,
    comment: Option<String>,
    columns: Vec<ColumnView>,
    constraints: Vec<ConstraintView>,
    indexes: Vec<IndexView>,
    backend: Option<TableDetailsView>,
}

impl TableViewBuilder {
    /// Sets the Markdown-ready qualified name.
    #[must_use]
    pub fn qualified_name(mut self, value: String) -> Self {
        self.qualified_name = Some(value);
        self
    }

    /// Sets the safe object filename.
    #[must_use]
    pub fn file_name(mut self, value: String) -> Self {
        self.file_name = Some(value);
        self
    }

    /// Sets the optional Markdown-ready comment.
    #[must_use]
    pub fn comment(mut self, value: Option<String>) -> Self {
        self.comment = value;
        self
    }

    /// Sets columns in catalog order.
    #[must_use]
    pub fn columns(mut self, value: Vec<ColumnView>) -> Self {
        self.columns = value;
        self
    }

    /// Sets constraints in deterministic backend order.
    #[must_use]
    pub fn constraints(mut self, value: Vec<ConstraintView>) -> Self {
        self.constraints = value;
        self
    }

    /// Sets indexes in deterministic backend order.
    #[must_use]
    pub fn indexes(mut self, value: Vec<IndexView>) -> Self {
        self.indexes = value;
        self
    }

    /// Sets backend-specific table details.
    #[must_use]
    pub fn backend(mut self, value: TableDetailsView) -> Self {
        self.backend = Some(value);
        self
    }

    /// Builds the prepared table.
    ///
    /// # Panics
    ///
    /// Panics when the qualified name, filename, or backend details were not set.
    #[must_use]
    pub fn build(self) -> TableView {
        TableView {
            qualified_name: self
                .qualified_name
                .expect("table presentation requires a qualified name"),
            file_name: self
                .file_name
                .expect("table presentation requires a filename"),
            comment: self.comment,
            columns: self.columns,
            constraints: self.constraints,
            indexes: self.indexes,
            backend: self
                .backend
                .expect("table presentation requires backend details"),
        }
    }
}

/// A column prepared for a shared column table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ColumnView {
    /// Markdown-ready column name.
    pub name: String,
    /// Markdown-ready data type.
    pub data_type: String,
    /// `yes`, `no`, or `unknown` nullability.
    pub nullable: &'static str,
    /// Markdown-ready default or an absence marker.
    pub default: String,
    /// Markdown-ready backend details.
    pub notes: String,
}

impl ColumnView {
    /// Starts a prepared-column builder.
    #[must_use]
    pub fn builder() -> ColumnViewBuilder {
        ColumnViewBuilder::default()
    }
}

/// Builder for [`ColumnView`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColumnViewBuilder {
    name: Option<String>,
    data_type: Option<String>,
    nullable: Option<&'static str>,
    default: Option<String>,
    notes: Option<String>,
}

impl ColumnViewBuilder {
    /// Sets the Markdown-ready column name.
    #[must_use]
    pub fn name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }

    /// Sets the Markdown-ready data type.
    #[must_use]
    pub fn data_type(mut self, value: String) -> Self {
        self.data_type = Some(value);
        self
    }

    /// Sets the nullability label.
    #[must_use]
    pub const fn nullable(mut self, value: &'static str) -> Self {
        self.nullable = Some(value);
        self
    }

    /// Sets the Markdown-ready default.
    #[must_use]
    pub fn default_value(mut self, value: String) -> Self {
        self.default = Some(value);
        self
    }

    /// Sets Markdown-ready backend notes.
    #[must_use]
    pub fn notes(mut self, value: String) -> Self {
        self.notes = Some(value);
        self
    }

    /// Builds the prepared column.
    ///
    /// # Panics
    ///
    /// Panics when any column presentation field was not set.
    #[must_use]
    pub fn build(self) -> ColumnView {
        ColumnView {
            name: self.name.expect("column presentation requires a name"),
            data_type: self
                .data_type
                .expect("column presentation requires a data type"),
            nullable: self
                .nullable
                .expect("column presentation requires nullability"),
            default: self
                .default
                .expect("column presentation requires a default display value"),
            notes: self
                .notes
                .expect("column presentation requires a notes display value"),
        }
    }
}

/// A constraint prepared for a shared constraint table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ConstraintView {
    /// Markdown-ready name or absence marker.
    pub name: String,
    /// Markdown-ready relational constraint category.
    pub kind: String,
    /// Markdown-ready ordered columns.
    pub columns: String,
    /// Markdown-ready backend semantics.
    pub details: String,
}

impl ConstraintView {
    /// Starts a prepared-constraint builder.
    #[must_use]
    pub fn builder() -> ConstraintViewBuilder {
        ConstraintViewBuilder::default()
    }
}

/// Builder for [`ConstraintView`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConstraintViewBuilder {
    name: Option<String>,
    kind: Option<String>,
    columns: Option<String>,
    details: Option<String>,
}

impl ConstraintViewBuilder {
    /// Sets the Markdown-ready name or absence marker.
    #[must_use]
    pub fn name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }

    /// Sets the Markdown-ready constraint category.
    #[must_use]
    pub fn kind(mut self, value: String) -> Self {
        self.kind = Some(value);
        self
    }

    /// Sets Markdown-ready ordered columns.
    #[must_use]
    pub fn columns(mut self, value: String) -> Self {
        self.columns = Some(value);
        self
    }

    /// Sets Markdown-ready backend details.
    #[must_use]
    pub fn details(mut self, value: String) -> Self {
        self.details = Some(value);
        self
    }

    /// Builds the prepared constraint.
    ///
    /// # Panics
    ///
    /// Panics when any constraint presentation field was not set.
    #[must_use]
    pub fn build(self) -> ConstraintView {
        ConstraintView {
            name: self.name.expect("constraint presentation requires a name"),
            kind: self.kind.expect("constraint presentation requires a kind"),
            columns: self
                .columns
                .expect("constraint presentation requires columns"),
            details: self
                .details
                .expect("constraint presentation requires details"),
        }
    }
}

/// An index prepared for a shared index table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct IndexView {
    /// Markdown-ready index name.
    pub name: String,
    /// Markdown-ready ordered key terms.
    pub terms: String,
    /// `yes` or `no` uniqueness.
    pub unique: &'static str,
    /// Markdown-ready origin and backend facts.
    pub origin: String,
    /// Markdown-ready predicate or absence marker.
    pub predicate: String,
}

impl IndexView {
    /// Starts a prepared-index builder.
    #[must_use]
    pub fn builder() -> IndexViewBuilder {
        IndexViewBuilder::default()
    }
}

/// Builder for [`IndexView`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IndexViewBuilder {
    name: Option<String>,
    terms: Option<String>,
    unique: Option<&'static str>,
    origin: Option<String>,
    predicate: Option<String>,
}

impl IndexViewBuilder {
    /// Sets the Markdown-ready index name.
    #[must_use]
    pub fn name(mut self, value: String) -> Self {
        self.name = Some(value);
        self
    }

    /// Sets Markdown-ready ordered key terms.
    #[must_use]
    pub fn terms(mut self, value: String) -> Self {
        self.terms = Some(value);
        self
    }

    /// Sets the uniqueness label.
    #[must_use]
    pub const fn unique(mut self, value: &'static str) -> Self {
        self.unique = Some(value);
        self
    }

    /// Sets Markdown-ready origin and backend facts.
    #[must_use]
    pub fn origin(mut self, value: String) -> Self {
        self.origin = Some(value);
        self
    }

    /// Sets the Markdown-ready predicate or absence marker.
    #[must_use]
    pub fn predicate(mut self, value: String) -> Self {
        self.predicate = Some(value);
        self
    }

    /// Builds the prepared index.
    ///
    /// # Panics
    ///
    /// Panics when any index presentation field was not set.
    #[must_use]
    pub fn build(self) -> IndexView {
        IndexView {
            name: self.name.expect("index presentation requires a name"),
            terms: self.terms.expect("index presentation requires terms"),
            unique: self.unique.expect("index presentation requires uniqueness"),
            origin: self.origin.expect("index presentation requires an origin"),
            predicate: self
                .predicate
                .expect("index presentation requires a predicate display value"),
        }
    }
}

/// Backend-specific details displayed by the shared table template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct TableDetailsView {
    /// Backend section title.
    pub title: &'static str,
    /// Labeled backend facts.
    pub facts: Vec<FactView>,
    /// Important backend notices.
    pub notices: Vec<&'static str>,
    /// Optional fenced table definition.
    pub definition: Option<String>,
}

impl TableDetailsView {
    /// Starts a backend-specific table-details builder.
    #[must_use]
    pub fn builder() -> TableDetailsViewBuilder {
        TableDetailsViewBuilder::default()
    }
}

/// Builder for [`TableDetailsView`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TableDetailsViewBuilder {
    title: Option<&'static str>,
    facts: Vec<FactView>,
    notices: Vec<&'static str>,
    definition: Option<String>,
}

impl TableDetailsViewBuilder {
    /// Sets the backend section title.
    #[must_use]
    pub const fn title(mut self, value: &'static str) -> Self {
        self.title = Some(value);
        self
    }

    /// Sets labeled backend facts.
    #[must_use]
    pub fn facts(mut self, value: Vec<FactView>) -> Self {
        self.facts = value;
        self
    }

    /// Sets important backend notices.
    #[must_use]
    pub fn notices(mut self, value: Vec<&'static str>) -> Self {
        self.notices = value;
        self
    }

    /// Sets the optional fenced table definition.
    #[must_use]
    pub fn definition(mut self, value: Option<String>) -> Self {
        self.definition = value;
        self
    }

    /// Builds the prepared backend details.
    ///
    /// # Panics
    ///
    /// Panics when the backend title was not set.
    #[must_use]
    pub fn build(self) -> TableDetailsView {
        TableDetailsView {
            title: self.title.expect("table details require a backend title"),
            facts: self.facts,
            notices: self.notices,
            definition: self.definition,
        }
    }
}

/// One Markdown-ready labeled fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct FactView {
    /// Stable display label.
    pub label: &'static str,
    /// Markdown-ready value.
    pub value: String,
}

impl FactView {
    /// Creates a labeled presentation fact.
    #[must_use]
    pub fn new(label: &'static str, value: impl Into<String>) -> Self {
        Self {
            label,
            value: value.into(),
        }
    }
}

/// A view or materialized view prepared for shared templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct ViewPresentation {
    /// Markdown-ready qualified name.
    pub qualified_name: String,
    /// Safe object filename.
    pub file_name: String,
    /// Optional Markdown-ready comment.
    pub comment: Option<String>,
    /// Labeled backend facts.
    pub facts: Vec<FactView>,
    /// Columns in catalog order.
    pub columns: Vec<ColumnView>,
    /// Fenced view definition.
    pub definition: String,
}

impl ViewPresentation {
    /// Starts a prepared-view builder.
    #[must_use]
    pub fn builder() -> ViewPresentationBuilder {
        ViewPresentationBuilder::default()
    }
}

/// Builder for [`ViewPresentation`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ViewPresentationBuilder {
    qualified_name: Option<String>,
    file_name: Option<String>,
    comment: Option<String>,
    facts: Vec<FactView>,
    columns: Vec<ColumnView>,
    definition: Option<String>,
}

impl ViewPresentationBuilder {
    /// Sets the Markdown-ready qualified name.
    #[must_use]
    pub fn qualified_name(mut self, value: String) -> Self {
        self.qualified_name = Some(value);
        self
    }

    /// Sets the safe object filename.
    #[must_use]
    pub fn file_name(mut self, value: String) -> Self {
        self.file_name = Some(value);
        self
    }

    /// Sets the optional Markdown-ready comment.
    #[must_use]
    pub fn comment(mut self, value: Option<String>) -> Self {
        self.comment = value;
        self
    }

    /// Sets labeled backend facts.
    #[must_use]
    pub fn facts(mut self, value: Vec<FactView>) -> Self {
        self.facts = value;
        self
    }

    /// Sets columns in catalog order.
    #[must_use]
    pub fn columns(mut self, value: Vec<ColumnView>) -> Self {
        self.columns = value;
        self
    }

    /// Sets the fenced view definition.
    #[must_use]
    pub fn definition(mut self, value: String) -> Self {
        self.definition = Some(value);
        self
    }

    /// Builds the prepared view.
    ///
    /// # Panics
    ///
    /// Panics when the qualified name, filename, or definition were not set.
    #[must_use]
    pub fn build(self) -> ViewPresentation {
        ViewPresentation {
            qualified_name: self
                .qualified_name
                .expect("view presentation requires a qualified name"),
            file_name: self
                .file_name
                .expect("view presentation requires a filename"),
            comment: self.comment,
            facts: self.facts,
            columns: self.columns,
            definition: self
                .definition
                .expect("view presentation requires a definition"),
        }
    }
}

/// A trigger prepared for shared templates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub struct TriggerView {
    /// Markdown-ready qualified identity.
    pub qualified_name: String,
    /// Safe object filename.
    pub file_name: String,
    /// Optional Markdown-ready comment.
    pub comment: Option<String>,
    /// Markdown-ready timing and event description.
    pub event: String,
    /// Markdown-ready trigger target.
    pub target: String,
    /// Labeled backend facts.
    pub facts: Vec<FactView>,
    /// Optional Markdown-ready predicate.
    pub when_expression: Option<String>,
    /// Fenced trigger definition.
    pub definition: String,
}

impl TriggerView {
    /// Starts a prepared-trigger builder.
    #[must_use]
    pub fn builder() -> TriggerViewBuilder {
        TriggerViewBuilder::default()
    }
}

/// Builder for [`TriggerView`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TriggerViewBuilder {
    qualified_name: Option<String>,
    file_name: Option<String>,
    comment: Option<String>,
    event: Option<String>,
    target: Option<String>,
    facts: Vec<FactView>,
    when_expression: Option<String>,
    definition: Option<String>,
}

impl TriggerViewBuilder {
    /// Sets the Markdown-ready qualified identity.
    #[must_use]
    pub fn qualified_name(mut self, value: String) -> Self {
        self.qualified_name = Some(value);
        self
    }

    /// Sets the safe object filename.
    #[must_use]
    pub fn file_name(mut self, value: String) -> Self {
        self.file_name = Some(value);
        self
    }

    /// Sets the optional Markdown-ready comment.
    #[must_use]
    pub fn with_comment(mut self, comment: Option<String>) -> Self {
        self.comment = comment;
        self
    }

    /// Sets the Markdown-ready timing and event description.
    #[must_use]
    pub fn event(mut self, value: String) -> Self {
        self.event = Some(value);
        self
    }

    /// Sets the Markdown-ready trigger target.
    #[must_use]
    pub fn target(mut self, value: String) -> Self {
        self.target = Some(value);
        self
    }

    /// Sets labeled backend facts.
    #[must_use]
    pub fn with_facts(mut self, facts: Vec<FactView>) -> Self {
        self.facts = facts;
        self
    }

    /// Sets the optional Markdown-ready predicate.
    #[must_use]
    pub fn with_when_expression(mut self, when_expression: Option<String>) -> Self {
        self.when_expression = when_expression;
        self
    }

    /// Sets the fenced trigger definition.
    #[must_use]
    pub fn definition(mut self, value: String) -> Self {
        self.definition = Some(value);
        self
    }

    /// Builds the prepared trigger.
    ///
    /// # Panics
    ///
    /// Panics when the identity, filename, event, target, or definition were not set.
    #[must_use]
    pub fn build(self) -> TriggerView {
        TriggerView {
            qualified_name: self
                .qualified_name
                .expect("trigger presentation requires a qualified name"),
            file_name: self
                .file_name
                .expect("trigger presentation requires a filename"),
            comment: self.comment,
            event: self.event.expect("trigger presentation requires an event"),
            target: self.target.expect("trigger presentation requires a target"),
            facts: self.facts,
            when_expression: self.when_expression,
            definition: self
                .definition
                .expect("trigger presentation requires a definition"),
        }
    }
}

/// Creates a directory-layout object manifest entry.
#[must_use]
pub fn directory_object(
    family: &str,
    template: &'static str,
    file_name: &str,
    data: impl Serialize,
) -> RenderObject {
    RenderObject::new(format!("{family}/{file_name}"), template, data)
}

/// Prepares namespaces for shared templates without changing their order.
#[must_use]
pub fn namespaces(values: &[Namespace]) -> Vec<NamespaceView> {
    values
        .iter()
        .map(|namespace| {
            NamespaceView::new(
                inline_code(&namespace.name),
                namespace.comment.as_deref().map(text),
            )
        })
        .collect()
}

/// Converts optional catalog nullability to its presentation label.
#[must_use]
pub const fn nullable(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "yes",
        Some(false) => "no",
        None => "unknown",
    }
}
