use chrono::DateTime;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;

/// An RSS feed the user has subscribed to.
///
/// A source is identified by its [`url`](Self::url), which is unique within a
/// [`Database`](crate::Database).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// Human readable name shown in the UI.
    pub name: String,
    /// The location the feed is fetched from.
    pub url: String,
    /// When the source was created.
    pub created_at: DateTime<Utc>,
    /// When the source was last updated.
    pub updated_at: DateTime<Utc>,
}
