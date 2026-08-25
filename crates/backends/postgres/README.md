# dbmd-backend-postgres

This vertical backend crate reads PostgreSQL catalogs through a connection URL
and produces one normalized
`dbmd_core::SourceSnapshot<dbmd_backend_postgres::Catalog>`. Catalog queries use
`pg_catalog` directly where `information_schema` would discard backend
semantics.

Native PostgreSQL discriminator codes are translated at the introspection seam
into backend-owned semantic enums. Catalog consumers and templates never need
to know ACL letters, relation-kind codes, or other `pg_catalog` encodings;
render mapping owns their human-facing labels. Server-formatted identities and
SQL definitions remain strings where quoting, overload signatures, or grammar
are themselves meaningful.

The executable compatibility target for this matrix is PostgreSQL 18.4.

`Config` owns the committed PostgreSQL source fields and resolves the connection
URL through application-supplied environment expansion before constructing a
credential-redacting `PostgresSource`. `include_cluster_objects = true` opts in
to cluster-wide databases, roles/memberships, and tablespaces; it defaults to
false so a database source does not unexpectedly expose global metadata.

## Coverage

The adapter currently preserves:

- The connected database's owner, encoding, locale provider/settings, default
  tablespace, template/connection policy, database-wide configuration, and
  comment. Opt-in cluster coverage exposes the same contract for every database.
- Opt-in tablespaces with owner, options, privileges, and comments. Host
  filesystem locations are never acquired and render as `<redacted>`.
- User schemas and schema comments.
- Ordinary, partitioned, partition, inherited, and foreign-table relation
  kinds; tablespaces when explicitly assigned.
- Columns in ordinal order with formatted types, nullability, defaults,
  identity mode, effective collation, virtual/stored generated expressions,
  enum labels, inheritance provenance, and comments.
- Primary, unique, foreign-key, check, not-null, and exclusion constraint
  categories, including the server-normalized definition, enforcement,
  validation, locality, inheritance, deferrability, temporal `WITHOUT OVERLAPS`
  and `PERIOD` semantics, semantic match mode, referential actions, and
  canonical exclusion-operator identities.
- Index keys and expressions, effective ordering/null placement, qualified
  collations and operator classes, access method, predicate, included columns,
  `NULLS NOT DISTINCT`, validity/readiness, clustering, replica identity,
  effective owner, explicit tablespace, storage parameters, partitioned/parent
  index identity, constraint linkage, per-term operator-class parameters, and
  the complete server-normalized definition.
- First-class enum types with owner and ordered values.
- Standalone composite types with owner, comments, ordered attributes, formatted
  types, effective collations, attribute comments, and a reconstructable current
  definition.
- Domains with their underlying type, effective collation, default, nullability,
  owner, ordered check constraints and validation state, comments, and a
  reconstructable current definition.
- User-defined shell and base types with ownership, implementation functions,
  ABI/storage properties, category/preference, array identity, defaults,
  comments, and reconstructable definitions.
- User-defined range types with subtype, operator class, collation,
  canonicalization/difference functions, ownership, comments, reconstructable
  definitions, and their paired multirange identity/owner/comment.
- Sequences with owner, integer type, bounds, start/increment/cache/cycle
  behavior, persistence, owned-column linkage, comments, and a reconstructable
  current definition.
- Views with owner, persistence, security-barrier/invoker options, check-option
  mode, relation options, columns, comments, and definitions; materialized
  views additionally retain access method, explicit tablespace, storage
  parameters, population state, and attached indexes.
- Functions with overload-safe signatures, return type, language, volatility,
  parallel safety, security mode, comments, and definitions.
- Procedures with overload-safe signatures, complete server-formatted arguments,
  owner, language, security mode, local configuration, transform types,
  comments, and definitions.
- Normal, ordered-set, and hypothetical-set aggregates with overload-safe
  signatures, direct arguments, transition/final/combine/serialization and
  moving-aggregate support functions, state types and allocation hints, initial
  conditions, final-function mutation modes, sort operator, parallel safety,
  ownership, and comments.
- Explicit user-defined casts with source/target types, invocation context,
  function/input-output/binary mechanism, conversion function, and extension
  ownership. Internally generated support casts are represented by their owning
  higher-level type contract instead of duplicated as independent DDL.
- Named encoding conversions with owner, source/target encodings, conversion
  function, default status, comments, and extension ownership.
- Binary and prefix operators with operand/result types, implementation,
  commutator/negator, selectivity estimators, hash/merge-join capability,
  comments, and extension ownership.
- Operator families and classes with access method, owner, input/key type,
  default status, family linkage, strategy operators, ordering/search purpose,
  sort families, and support functions.
- User-defined table/index access methods and handler identities.
- Procedural languages with owner, trusted/procedural status, handler, inline
  handler, validator, comments, and extension ownership.
- Type transforms with language, both optional conversion directions, and
  extension ownership.
- User-authored rewrite rules with target relation, event, `INSTEAD` behavior,
  enablement, comments, extension ownership, and server-normalized definition;
  implementation `_RETURN` rules remain represented through their owning view.
- Database-level event triggers with owner, event, tag filter, function,
  enablement, comments, extension ownership, and reconstructable definition.
- Extended planner statistics with owner, `ndistinct`/dependencies/MCV/
  expression kinds, ordered columns and expressions, statistics target,
  comments, extension ownership, and server definition.
- Foreign-data wrappers with owner, handler/validator identities, options,
  privileges, comments, and extension ownership; foreign servers with wrapper,
  type/version, options, privileges, comments, and extension ownership; and
  user mappings with server/user identity and options. Secret-shaped option
  keys are replaced with `<redacted>` during introspection, before they can
  enter a catalog snapshot or renderer.
- Text-search parsers, templates, dictionaries, and configurations, including
  implementation-function identities, dictionary template/options, ownership,
  comments, extension ownership, and ordered token-to-dictionary mappings.
- Traditional inheritance, partition keys/parents/bounds, row-level security,
  forced RLS, and policies.
- Triggers with multiple events and `UPDATE OF` columns, timing,
  row/statement orientation, `WHEN`, comments, called function and arguments,
  enablement state, constraint-trigger metadata, transition tables, parent
  trigger identity, and server-normalized definitions.
- Logical-replication publications with owner, actions, all-table/schema/table
  membership, explicit column lists, row filters, partition-root behavior,
  comments, and PostgreSQL 18 stored-generated-column publication mode.
- Logical-replication subscriptions with owner, enablement, binary/streaming/
  two-phase behavior, failure and execution policy, failover/slot/synchronous
  commit settings, ordered publications, origin filtering, pending skip LSN,
  and comments. Publisher connection strings are never acquired and render as
  `<redacted>`.
- User-created cluster roles with capability flags, login/connection policy,
  password-presence and expiration metadata, role-wide configuration, comments,
  and direct membership grants including PostgreSQL 18 admin/inherit/set
  options, plus per-role/per-database session defaults. Password values and
  verifiers are never acquired.
- Explicit grants across databases, schemas, relations and columns, sequences,
  routines, types/domains, procedural languages, large objects, foreign-data
  wrappers/servers, parameters, and opt-in tablespaces. Native ACL codes are
  normalized into typed object and privilege families with grantor, grantee,
  and grant-option semantics.
- Default privileges for tables, sequences, routines, types, schemas, and large
  objects, including owner and optional schema scope.
- Database-local security labels plus opt-in shared labels, represented by
  semantic object family, canonical identity, provider, and label.
- Large-object OID, owner, comment, and privileges. Binary content pages are
  never queried and render as omitted.
- User-defined collations with owner, provider (including PostgreSQL 18's
  builtin provider), locale/category values, determinism, encoding, ICU rules,
  version, and comments.
- Installed extensions with owner, exported-object schema, relocatability,
  version, comments, configuration-table filters, and every extension-owned
  object represented by PostgreSQL's stable external address. Modeled object
  families retain nullable extension ownership; indexes and triggers inherit
  effective ownership from an extension-owned table when PostgreSQL represents
  them as internal dependents rather than direct extension members.
- Backend-owned render mapping and PostgreSQL source templates for both output
  layouts.

## Fixture evidence

Container-backed fixtures live under `tests/fixtures/`:

- `ordinary_table` — columns, identities, generated values, comments, checks,
  and expression/partial indexes.
- `relationships` — composite keys and fully specified foreign keys.
- `schema_objects` — schemas, enums, security-barrier/invoker views with local
  check options, unpopulated materialized views with access/storage parameters
  and attached indexes, functions, procedures, both output layouts, and
  deterministic introspection.
- `table_semantics` — inheritance, table and index partitioning/attachment,
  per-partition index options, RLS, and policies.
- `indexes_and_constraints` — covering indexes, opclasses and their parameters,
  exclusion operators, null semantics, storage parameters, effective ownership,
  constraint linkage, clustering, replica identity, and constraint enforcement
  state.
- `triggers` — row and statement triggers, multi-event ordering, view and
  constraint triggers, transition tables, enablement modes, arguments, comments,
  predicates, and cloned partition-trigger parent identity.
- `sequences` — type, bounds, increment/cache/cycle, persistence, ownership,
  comments, and both output layouts.
- `domains` — type modifiers, collation, default/nullability, named checks,
  validation, ownership, comments, and both output layouts.
- `composite_types` — ordered attributes, type modifiers, effective collations,
  type and attribute comments, ownership, and both output layouts.
- `type_system` — shell and internal-backed base types, implementation/storage
  properties, range and multirange pairing, comments, consuming table columns,
  and both output layouts.
- `extensions` — PostgreSQL 18.4 with pgvector 0.8.2, complete stable member
  addresses, `vector(3)`, HNSW and IVFFlat indexes with their operator classes,
  plus a test extension proving configuration-table filters and nullable/effective
  ownership across every modeled object family. Both layouts prove compact
  extension summaries and deterministic repeated introspection.
- `aggregates` — normal, moving, parallel, ordered-set, and hypothetical-set
  aggregates, their support-function identities and state contracts, comments,
  and both output layouts.
- `postgres_18` — virtual/stored generated columns, named not-null constraints,
  enforced/unenforced constraints, temporal unique/foreign keys,
  `PG_UNICODE_FAST` builtin collation semantics, logical-replication
  publications, `btree_gist` extension ownership/member attribution, and both
  output layouts.
- `type_operator_infrastructure` — function, input/output, and binary casts; a
  default encoding conversion; binary and prefix operators; a B-tree operator
  family/class with strategy and support members; a custom index access method;
  a trusted procedural language; a bidirectional transform and a procedure that
  consumes it; both output layouts; and deterministic repeated introspection.
- `advanced_schema_objects` — a replica-only rewrite rule, an always-enabled
  filtered event trigger, multivariate and expression statistics, all four
  text-search object families with ordered mappings, a disconnected logical
  subscription whose connection secret never enters the catalog, comments,
  both output layouts, and deterministic repeated introspection.
- `table_properties` — typed/unlogged/foreign-table storage properties plus a
  foreign-data wrapper, server, and public user mapping; secret option values
  are absent from catalog and rendered snapshots, both layouts are covered,
  and repeated introspection is deterministic.
- `roles` — isolated cluster fixture for login/capability policy, password-safe
  metadata, expiration, configuration, comments, PostgreSQL 18 membership
  options, both output layouts, and deterministic repeated introspection.
- `access_control` — semantic object/column/parameter privileges including
  PostgreSQL 18 `MAINTAIN`, every default-privilege family, large-object
  metadata, database-scoped role settings, both layouts, and proof that an
  actual secret large-object payload never enters the catalog or rendered
  output. The stock server has no security label provider, so the fixture
  verifies the empty-provider path.

## Template context

PostgreSQL source entrypoints receive the common `source` envelope documented
by the [template product contract](../../../docs/product/features/templates.md).
`source.data` has this PostgreSQL-owned shape:

| Field | Type | Meaning and order |
| --- | --- | --- |
| `section_heading` | string | `##` without source nesting, `###` with nesting. |
| `object_heading` | string | Heading used by single-file object templates. |
| `detail_heading` | string | Heading used for table subsections in a single file. |
| `database` | object | Connected database creation/configuration contract. |
| `cluster_databases` | object[] | Opt-in cluster databases in name order. |
| `tablespaces` | object[] | Opt-in cluster tablespaces in name order, with locations redacted. |
| `namespaces` | namespace[] | User schemas in binary name order. |
| `enums` | enum[] | Enum types in schema/name order. |
| `composite_types` | object[] | Standalone composite types in schema/name order. |
| `domains` | object[] | Domain types in schema/name order. |
| `base_types` | object[] | User-defined base and shell types in schema/name order. |
| `range_types` | object[] | User-defined range types and their paired multiranges in schema/name order. |
| `sequences` | object[] | Sequences in schema/name order. |
| `tables` | table[] | Relations in schema/name order, including partitions and foreign tables. |
| `views` | view[] | Ordinary and materialized views together in schema/name order. |
| `triggers` | trigger[] | Triggers in target schema, target relation, and trigger-name order. |
| `functions` | function[] | Functions in schema, name, and overload-safe signature order. |
| `procedures` | object[] | Procedures in schema, name, and overload-safe signature order. |
| `aggregates` | object[] | Aggregates in schema, name, and overload-safe signature order. |
| `casts` | object[] | Explicit casts in source/target type order. |
| `conversions` | object[] | Encoding conversions in schema/name order. |
| `operators` | object[] | Operators in schema/name/operand order. |
| `operator_families` | object[] | Operator families in access-method/schema/name order. |
| `operator_classes` | object[] | Operator classes in access-method/schema/name order. |
| `access_methods` | object[] | User-defined access methods in name order. |
| `languages` | object[] | Procedural languages in name order. |
| `transforms` | object[] | Type/language transforms in deterministic order. |
| `rules` | object[] | User-authored rewrite rules in target schema/relation/name order. |
| `event_triggers` | object[] | Database-level event triggers in name order. |
| `statistics` | object[] | Extended statistics in schema/name order. |
| `foreign_data_wrappers` | object[] | Foreign-data wrappers in name order, with secret options redacted. |
| `foreign_servers` | object[] | Foreign servers in name order, with secret options redacted. |
| `user_mappings` | object[] | User mappings in server/user order, with secret options redacted. |
| `text_search_parsers` | object[] | Text-search parsers in schema/name order. |
| `text_search_templates` | object[] | Text-search templates in schema/name order. |
| `text_search_dictionaries` | object[] | Text-search dictionaries in schema/name order. |
| `text_search_configurations` | object[] | Text-search configurations and ordered mappings in schema/name order. |
| `publications` | object[] | Logical-replication publications in name order. |
| `subscriptions` | object[] | Logical-replication subscriptions in name order, with connection data redacted. |
| `roles` | object[] | User-created cluster roles in name order, without password material. |
| `role_database_settings` | object[] | Opt-in database-scoped role defaults in database/role order. |
| `privileges` | object[] | Semantic explicit grants in object/grantee/privilege order. |
| `default_privileges` | object[] | Default grants in owner/schema/object-family/grantee order. |
| `security_labels` | object[] | Provider labels in object/provider order. |
| `large_objects` | object[] | Large-object metadata in OID order; contents are always omitted. |
| `collations` | object[] | User-defined collations in schema/name order. |
| `extensions` | object[] | Installed extensions in name order, with configuration relations and stable member addresses. |

Extension-owned support objects are retained in the backend catalog but omitted
from ordinary object sections. Each extension renders a compact deterministic
count by native member kind. Application objects that use extension facilities
remain ordinary objects: for example, a table containing `vector(3)` and its
HNSW/IVFFlat indexes are not owned by pgvector and remain fully rendered.

PostgreSQL directory objects are declared in connected-database, opt-in cluster
database/tablespace, enum, composite-type, table, domain, base/shell-type,
range/multirange-type, sequence, view, trigger,
function, procedure, aggregate, cast, conversion, operator, operator-family,
operator-class, access-method, language, transform, rewrite-rule, event-trigger,
extended-statistics, foreign-data-wrapper, foreign-server, user-mapping,
text-search parser/template/dictionary/configuration, publication, subscription,
role, role-database-setting, privilege, default-privilege, security-label, large-object, collation,
then extension order. Their directories are `database/`,
`cluster-databases/`, `tablespaces/`, `enums/`, `composite-types/`, `tables/`,
`domains/`, `base-types/`, `range-types/`,
`sequences/`, `views/`, `triggers/`, `functions/`, `procedures/`, `aggregates/`,
`casts/`, `conversions/`, `operators/`, `operator-families/`,
`operator-classes/`, `access-methods/`, `languages/`, `transforms/`,
`rules/`, `event-triggers/`, `statistics/`, `foreign-data-wrappers/`,
`foreign-servers/`, `user-mappings/`, `text-search-parsers/`,
`text-search-templates/`, `text-search-dictionaries/`,
`text-search-configurations/`, `publications/`, `subscriptions/`, `roles/`,
`role-database-settings/`, `privileges/`, `default-privileges/`, `security-labels/`, `large-objects/`,
`collations/`, and `extensions/`.
Trigger identity and filenames include their target relation;
function filenames include the identity-argument signature.

Presentation object fields are:

- Namespace: `name`, nullable `comment`.
- Enum: `qualified_name`, `file_name`, nullable `comment`, and `values`.
- Table: `qualified_name`, `file_name`, nullable `comment`, `columns`,
  `constraints`, `indexes`, and `backend`. `backend.title` is `PostgreSQL`;
  `backend.facts` contains kind, storage/partition/inheritance details and RLS
  policies; `backend.notices` carries enabled/forced RLS state;
  `backend.definition` is currently null.
- Column: `name`, `data_type`, `nullable` (`yes`, `no`, or `unknown`),
  `default` (`-` when absent), and `notes` for comments, identity, generated
  expression kind, and enum labels.
- Constraint: `name`, `kind`, `columns`, and `details`; details preserve the
  normalized definition plus enforcement, validation, inheritance, and temporal
  state.
- Index: `name`, `terms`, `unique`, `origin`, and `predicate`; `origin` includes
  PostgreSQL access method, included columns, null-distinctness, validity,
  readiness, clustering, and replica-identity facts.
- View: `qualified_name`, `file_name`, nullable `comment`, `facts` containing
  `view` or `materialized_view`, `columns`, and fenced `definition`.
- Trigger: `qualified_name`, `file_name`, nullable `comment`, combined `event`,
  `target`, `facts`, nullable `when_expression`, and fenced `definition`.
  Facts preserve orientation, function, enablement, arguments, constraint and
  transition-table metadata, and nullable parent-trigger identity.
- Composite type, domain, sequence, function, procedure, aggregate, cast,
  conversion, operator, operator family/class, access method, language,
  transform, rewrite rule, event trigger, extended statistics, foreign-data
  wrapper, foreign server, user mapping, text-search parser/template/dictionary/
  configuration, publication, and collation:
  `qualified_name`, `file_name`,
  nullable `comment`, `facts`, and
  nullable fenced `definition`. Facts preserve return type, language,
  volatility, parallel safety, and security mode.

In a directory object template the selected object is `object`; `heading`,
`detail_heading`, and `source` are also available. In a single-file source
template the same objects are under `source.data` and the backend entrypoint
chooses how to iterate or include them. Values are Markdown-ready; templates
must not re-derive PostgreSQL catalog semantics.

Raw server definitions remain a fidelity backstop for represented objects.
