//! Backend-neutral source identity, snapshot envelopes, and aggregation invariants.

mod context;

pub use context::{DatabaseContext, DatabaseContextError, SourceId, SourceIdError, SourceSnapshot};
