//! Pagination domain model providing common structures for cursor-style list responses.

use serde::{Deserialize, Serialize};

/// Stable data boundary for `CursorPage`, exposed by or reused within this module.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
}

impl<T> CursorPage<T> {
    /// Runs the `new` server-side flow while preserving input validation, error propagation, and state invariants.
    pub fn new(items: Vec<T>, next_cursor: Option<String>) -> Self {
        Self { items, next_cursor }
    }
}
