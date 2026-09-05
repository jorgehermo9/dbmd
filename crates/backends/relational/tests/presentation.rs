use dbmd_relational::{
    presentation::{namespaces, nullable},
    ForeignKeyAction, ForeignKeyDeferrability, ForeignKeyInitialTiming, ForeignKeyMatch,
    ForeignKeyReference, Namespace,
};

#[test]
fn namespaces_preserve_catalog_order_and_prepare_markdown_safe_values() {
    let prepared = namespaces(&[
        Namespace::new("zeta|raw", Some("line one\nline two".to_string())),
        Namespace::new("alpha", None),
    ]);

    assert_eq!(prepared[0].name, "`zeta\\|raw`");
    assert_eq!(prepared[0].comment.as_deref(), Some("line one<br>line two"));
    assert_eq!(prepared[1].name, "`alpha`");
    assert_eq!(prepared[1].comment, None);
}

#[test]
fn optional_nullability_has_explicit_yes_no_and_unknown_labels() {
    assert_eq!(nullable(Some(true)), "yes");
    assert_eq!(nullable(Some(false)), "no");
    assert_eq!(nullable(None), "unknown");
}

#[test]
fn foreign_key_reference_defaults_and_overrides_are_semantic() {
    let default = ForeignKeyReference::new("public", "accounts", vec!["id".to_string()]);
    assert_eq!(default.on_update, ForeignKeyAction::NoAction);
    assert_eq!(default.on_delete, ForeignKeyAction::NoAction);
    assert_eq!(default.match_type, None);
    assert_eq!(default.deferrability, ForeignKeyDeferrability::default());

    let configured = default
        .with_actions(ForeignKeyAction::Cascade, ForeignKeyAction::SetNull)
        .with_match_type(Some(ForeignKeyMatch::Full))
        .with_deferrability(ForeignKeyDeferrability::new(
            true,
            ForeignKeyInitialTiming::Deferred,
        ));
    assert_eq!(configured.on_update, ForeignKeyAction::Cascade);
    assert_eq!(configured.on_delete, ForeignKeyAction::SetNull);
    assert_eq!(configured.match_type, Some(ForeignKeyMatch::Full));
    assert_eq!(
        configured.deferrability,
        ForeignKeyDeferrability::new(true, ForeignKeyInitialTiming::Deferred)
    );
}
