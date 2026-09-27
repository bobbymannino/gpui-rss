//! Fetching and decoding of RSS, Atom and JSON feeds.

mod post;

use anyhow::{Context, Result};

pub use crate::post::Post;

/// Downloads the feed at `url` and decodes it into its posts.
///
/// This blocks the current thread on network I/O, so run it on a background thread.
///
/// # Errors
///
/// Returns an error if the request fails, the server responds with a non-success status, or the response is not a
/// valid feed.
pub fn fetch(url: &str) -> Result<Vec<Post>> {
    let body = ureq::get(url)
        .call()
        .with_context(|| format!("failed to fetch {url}"))?
        .body_mut()
        .read_to_vec()
        .with_context(|| format!("failed to read response from {url}"))?;

    parse(&body).with_context(|| format!("failed to decode feed from {url}"))
}

/// Decodes a feed document into its posts, in the order they appear in the document.
///
/// RSS 0.x/1.0/2.0, Atom and JSON Feed documents are all supported.
///
/// # Errors
///
/// Returns an error if `bytes` is not a valid feed.
pub fn parse(bytes: &[u8]) -> Result<Vec<Post>> {
    let feed = feed_rs::parser::parse(bytes).context("invalid feed document")?;

    Ok(feed.entries.into_iter().map(Post::from).collect())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;

    const RSS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Example</title>
    <link>https://example.com</link>
    <description>An example feed</description>
    <item>
      <guid>https://example.com/first</guid>
      <title>First post</title>
      <link>https://example.com/first</link>
      <description>The first post.</description>
      <author>jane@example.com (Jane)</author>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
    </item>
    <item>
      <title>Second post</title>
    </item>
  </channel>
</rss>"#;

    const ATOM: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Example</title>
  <id>urn:example</id>
  <updated>2024-01-02T00:00:00Z</updated>
  <entry>
    <id>urn:example:1</id>
    <title>Atom post</title>
    <link href="https://example.com/atom"/>
    <updated>2024-01-02T00:00:00Z</updated>
    <summary>An atom post.</summary>
  </entry>
</feed>"#;

    #[test]
    fn parses_rss_items() {
        let posts = parse(RSS.as_bytes()).expect("parse");

        assert_eq!(posts.len(), 2);

        let first = &posts[0];
        assert_eq!(first.id(), "https://example.com/first");
        assert_eq!(first.title(), "First post");
        assert_eq!(first.link(), Some("https://example.com/first"));
        assert_eq!(first.summary(), Some("The first post."));
        assert!(first.author().is_some());
        assert_eq!(
            first.published_at(),
            Some(Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap())
        );

        let second = &posts[1];
        assert_eq!(second.title(), "Second post");
        assert_eq!(second.link(), None);
        assert_eq!(second.published_at(), None);
    }

    #[test]
    fn parses_atom_entries_falling_back_to_updated() {
        let posts = parse(ATOM.as_bytes()).expect("parse");

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].title(), "Atom post");
        assert_eq!(posts[0].link(), Some("https://example.com/atom"));
        assert_eq!(
            posts[0].published_at(),
            Some(Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap())
        );
    }

    #[test]
    fn invalid_document_is_an_error() {
        assert!(parse(b"not a feed").is_err());
    }
}
