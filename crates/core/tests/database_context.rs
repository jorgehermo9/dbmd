use std::str::FromStr;

use dbmd_core::{
    Backend, DatabaseContext, DatabaseContextError, SourceId, SourceIdError, SourceSnapshot,
};

fn source(id: &str) -> SourceSnapshot {
    SourceSnapshot::new(
        SourceId::from_str(id).expect("test source ID should be valid"),
        Backend::Sqlite,
    )
}

#[test]
fn source_id_accepts_the_documented_slug_grammar() {
    let source_id = SourceId::from_str("analytics-prod_2");

    assert_eq!(
        source_id
            .expect("documented source ID should be valid")
            .as_str(),
        "analytics-prod_2"
    );
}

#[test]
fn source_id_rejects_empty_and_path_like_values() {
    assert_eq!(SourceId::from_str(""), Err(SourceIdError::Empty));
    assert!(matches!(
        SourceId::from_str("analytics/prod"),
        Err(SourceIdError::InvalidCharacter {
            character: '/',
            index: 9
        })
    ));
}

#[test]
fn database_context_preserves_selected_source_order_and_identity() {
    let mut analytics = source("analytics");
    analytics.display_name = Some("Analytics Warehouse".to_string());
    let application = source("app");

    let context = DatabaseContext::new(vec![analytics, application])
        .expect("distinct nonempty sources should form a database context");

    let source_ids = context
        .sources()
        .iter()
        .map(|source| source.id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(source_ids, ["analytics", "app"]);
    assert_eq!(
        context.sources()[0].display_name.as_deref(),
        Some("Analytics Warehouse")
    );
    assert_eq!(context.sources()[0].id.as_str(), "analytics");
}

#[test]
fn database_context_rejects_an_empty_source_selection() {
    let result = DatabaseContext::new(Vec::new());

    assert_eq!(result, Err(DatabaseContextError::Empty));
}

#[test]
fn database_context_rejects_duplicate_source_ids() {
    let result = DatabaseContext::new(vec![source("app"), source("app")]);

    assert_eq!(
        result,
        Err(DatabaseContextError::DuplicateSourceId(
            SourceId::from_str("app").expect("test source ID should be valid")
        ))
    );
}
