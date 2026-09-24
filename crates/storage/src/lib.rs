//! Persistence layer for RSS feeds.

mod json;
mod source;

pub use crate::source::Source;
use anyhow::Result;

/// A backend holding the [`Source`]s the user has subscribed to.
pub trait Store: Send + Sync {
    /// What this backend needs to open itself, such as a path or a connection
    /// string.
    type Config;

    /// Open the store described by `config`, creating the backing storage if
    /// it is not there yet.
    ///
    /// Opening an existing store leaves the sources in it alone.
    ///
    /// # Errors
    ///
    /// Fails if the backing storage cannot be reached, or cannot be created
    /// when it is missing.
    fn get_or_create(config: Self::Config) -> Result<Self>
    where
        Self: Sized;

    /// Every source, in the order they were added.
    ///
    /// # Errors
    ///
    /// Fails if the stored sources cannot be read.
    fn list(&self) -> Result<Vec<Source>>;

    /// Add `source` to the store.
    ///
    /// # Errors
    ///
    /// Fails if a source with the same URL is already stored, or if the store
    /// cannot be written.
    fn add(&self, source: Source) -> Result<()>;

    /// Remove the source with `url`, reporting whether there was one to remove.
    ///
    /// # Errors
    ///
    /// Fails if the store cannot be read or written.
    fn remove(&self, url: &str) -> Result<bool>;
}
