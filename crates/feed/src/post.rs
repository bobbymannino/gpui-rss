use chrono::{DateTime, Utc};
use feed_rs::model::Entry;

/// A single item published by a feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    /// Unique identifier of the post within its feed.
    id: String,
    /// Headline of the post.
    title: String,
    /// Location of the full post, if the feed provides one.
    link: Option<String>,
    /// Short description or excerpt of the post.
    summary: Option<String>,
    /// Name of whoever wrote the post.
    author: Option<String>,
    /// When the post was published, falling back to when it was last updated.
    published_at: Option<DateTime<Utc>>,
}

impl Post {
    /// Unique identifier of the post within its feed.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Headline of the post.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Location of the full post, if the feed provides one.
    #[must_use]
    pub fn link(&self) -> Option<&str> {
        self.link.as_deref()
    }

    /// Short description or excerpt of the post.
    #[must_use]
    pub fn summary(&self) -> Option<&str> {
        self.summary.as_deref()
    }

    /// Name of whoever wrote the post.
    #[must_use]
    pub fn author(&self) -> Option<&str> {
        self.author.as_deref()
    }

    /// When the post was published, falling back to when it was last updated.
    #[must_use]
    pub const fn published_at(&self) -> Option<DateTime<Utc>> {
        self.published_at
    }
}

impl From<Entry> for Post {
    fn from(entry: Entry) -> Self {
        Self {
            title: entry.title.map(|title| title.content).unwrap_or_default(),
            link: entry.links.into_iter().next().map(|link| link.href),
            summary: entry.summary.map(|summary| summary.content),
            author: entry.authors.into_iter().next().map(|author| author.name),
            published_at: entry.published.or(entry.updated),
            id: entry.id,
        }
    }
}
