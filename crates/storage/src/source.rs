use chrono::{DateTime, Utc};
use feed::Post;
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
    /// Posts from the last successful fetch, in the order the feed lists them.
    #[serde(default)]
    posts: Vec<Post>,
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
            posts: Vec::new(),
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

    /// Posts from the last successful fetch, in the order the feed lists them.
    #[must_use]
    pub fn posts(&self) -> &[Post] {
        &self.posts
    }

    /// Replaces the posts with a freshly `fetched` set, keeping when each already known post was read.
    pub fn set_posts(&mut self, mut fetched: Vec<Post>) {
        for post in &mut fetched {
            if let Some(existing) = self.posts.iter().find(|existing| existing.id() == post.id()) {
                post.set_read_at(existing.read_at());
            }
        }

        self.posts = fetched;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    const RSS: &str = r#"<?xml version="1.0"?>
<rss version="2.0">
  <channel>
    <title>Example</title>
    <item><guid>a</guid><title>A</title></item>
    <item><guid>b</guid><title>B</title></item>
  </channel>
</rss>"#;

    #[test]
    fn set_posts_keeps_read_at_of_known_posts() {
        let read_at = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let mut source = Source::new("Example", "https://example.com/feed.xml");

        let mut posts = feed::parse(RSS.as_bytes()).expect("parse");
        for post in posts.iter_mut().filter(|post| post.id() == "a") {
            post.set_read_at(Some(read_at));
        }
        source.set_posts(posts);

        source.set_posts(feed::parse(RSS.as_bytes()).expect("parse"));

        let read: Vec<_> = source.posts().iter().map(|post| (post.id(), post.read_at())).collect();
        assert_eq!(read, [("a", Some(read_at)), ("b", None)]);
    }
}
