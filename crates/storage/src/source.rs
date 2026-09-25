use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// An RSS feed the user has subscribed to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    /// Human readable name shown in the UI.
    name: String,
    /// The location the feed is fetched from.
    url: String,
    /// When the source was created.
    created_at: DateTime<Utc>,
    /// When the source was last updated.
    updated_at: DateTime<Utc>,
}

impl Source {
    /// A source subscribed to just now.
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        let now = Utc::now();

        Self {
            name: name.into(),
            url: url.into(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Human readable name shown in the UI.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The location the feed is fetched from.
    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// When the source was created.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// When the source was last updated.
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}
