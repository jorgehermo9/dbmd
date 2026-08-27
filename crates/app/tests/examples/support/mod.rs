//! Manifest-driven acceptance harness for user-facing example projects.

#![allow(
    dead_code,
    unused_imports,
    reason = "each example-suite integration binary uses a different backend slice"
)]

mod manifest;
mod runner;

pub use manifest::{Backend, Suite};
pub use runner::{
    assert_inventory_conforms, assert_inventory_conforms_at, load_suite, run_example, schema_sql,
};
