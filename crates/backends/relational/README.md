# dbmd-relational

`dbmd-relational` contains only relational leaf values and presentation helpers
whose semantics are genuinely equivalent across multiple dbmd backends. It does
not define a universal database catalog: tables, columns, triggers, functions,
and other aggregate schema objects remain owned by their concrete backend.

Shared catalog values currently cover namespaces, foreign-key targets and
actions, deferrability, and ascending/descending index order. Complete
constraint categories and index terms remain backend-owned so PostgreSQL
exclusion/operator-class/null-placement facts and SQLite row identifiers do not
widen a common model.

The crate is internal workspace support for backend implementations. New types
belong here only when both SQLite and PostgreSQL can use the same meaning without
discarding backend facts.
