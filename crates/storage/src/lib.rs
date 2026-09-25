//! Persistence layer for RSS feeds.

mod source;

pub use crate::source::Source;
use anyhow::{Context, Result};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// A list of [`Source`]s persisted as a JSON file on disk.
#[derive(Debug)]
pub struct JSONDatabase {
    filepath: PathBuf,
    sources: Vec<Source>,
}

impl JSONDatabase {
    /// Opens the database at `filepath`, loading any sources already saved there.
    ///
    /// A missing file is treated as an empty database; it is created on the first [`write`](Self::write).
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or does not contain valid JSON.
    pub fn new(filepath: impl Into<PathBuf>) -> Result<Self> {
        let filepath = filepath.into();
        let sources = load(&filepath)?;

        Ok(Self { filepath, sources })
    }

    /// The sources currently held in memory.
    #[must_use]
    pub fn sources(&self) -> &[Source] {
        &self.sources
    }

    /// Mutable access to the in-memory sources. Call [`write`](Self::write) to persist changes.
    pub const fn sources_mut(&mut self) -> &mut Vec<Source> {
        &mut self.sources
    }

    /// Reads the sources currently saved on disk, ignoring any unsaved in-memory changes.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or does not contain valid JSON.
    pub fn read(&self) -> Result<Vec<Source>> {
        load(&self.filepath)
    }

    /// Saves the in-memory sources to disk, creating parent directories as needed.
    ///
    /// The data is written to a temporary file first and then renamed into place,
    /// so a crash mid-write never leaves a truncated database behind.
    ///
    /// # Errors
    ///
    /// Returns an error if the sources cannot be serialized or the file cannot be written.
    pub fn write(&self) -> Result<()> {
        if let Some(parent) = self.filepath.parent() {
            fs::create_dir_all(parent).with_context(|| format!("failed to create directory {}", parent.display()))?;
        }

        let json = serde_json::to_string_pretty(&self.sources).context("failed to serialize sources")?;
        let tmp = self.filepath.with_extension("json.tmp");

        fs::write(&tmp, json).with_context(|| format!("failed to write {}", tmp.display()))?;
        fs::rename(&tmp, &self.filepath).with_context(|| format!("failed to replace {}", self.filepath.display()))?;

        Ok(())
    }
}

/// Loads sources from `path`, returning an empty list if the file does not exist.
fn load(path: &Path) -> Result<Vec<Source>> {
    match fs::read_to_string(path) {
        Ok(json) => serde_json::from_str(&json).with_context(|| format!("failed to parse {}", path.display())),
        Err(err) if err.kind() == ErrorKind::NotFound => Ok(Vec::new()),
        Err(err) => Err(err).with_context(|| format!("failed to read {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db = JSONDatabase::new(dir.path().join("db.json")).expect("open");

        assert!(db.sources().is_empty());
        assert!(db.read().expect("read").is_empty());
    }

    #[test]
    fn write_then_reopen_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("nested").join("db.json");

        let mut db = JSONDatabase::new(&path).expect("open");
        db.sources_mut()
            .push(Source::new("Example", "https://example.com/feed.xml"));
        db.write().expect("write");

        let reopened = JSONDatabase::new(&path).expect("open");
        assert_eq!(reopened.sources(), db.sources());
        assert_eq!(db.read().expect("read"), db.sources());
    }

    #[test]
    fn invalid_json_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("db.json");
        fs::write(&path, "not json").expect("write fixture");

        assert!(JSONDatabase::new(&path).is_err());
    }
}
