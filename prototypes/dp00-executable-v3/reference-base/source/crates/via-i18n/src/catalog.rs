//! The generated catalog table.
//!
//! `build.rs` reads `assets/i18n/*.json` and writes `$OUT_DIR/catalog.rs`,
//! which is `include!`d here. Nothing in this module is hand-written.

use crate::key::{Entry, Key};

include!(concat!(env!("OUT_DIR"), "/catalog.rs"));
