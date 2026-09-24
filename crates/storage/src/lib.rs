//! Persistence layer for RSS feeds.
//!
//! The database is a single JSON document holding every [`Source`] the user
//! has subscribed to.

mod source;

use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

pub use crate::source::Source;

/// The contents of the JSON document backing the store.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Database {
    /// Every feed the user has subscribed to, in the order they were added.
    #[serde(default)]
    pub sources: Vec<Source>,
    /// When was the database created.
    pub created_at: DateTime<Utc>,
    /// When was the database last updated.
    pub updated_at: DateTime<Utc>,
}
