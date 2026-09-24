use crate::{Source, Store};
use anyhow::Result;
use std::error;
use std::fmt::{self, Display, Formatter};
use std::path::PathBuf;

#[derive(Debug)]
pub struct JsonStore {
    path: PathBuf,
}

/// Something that went wrong opening a [`JsonStore`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JsonStoreError {
    /// The configured path did not name a `.json` file.
    InvalidFileExtension(String),
}

impl Display for JsonStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFileExtension(path) => write!(f, "{path} is not a .json file"),
        }
    }
}

impl error::Error for JsonStoreError {}

pub struct JsonStoreConfig {
    /// The path of the JSON file, including the filename and extension.
    path: String,
}

impl JsonStoreConfig {
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }
}

impl Store for JsonStore {
    type Config = JsonStoreConfig;

    fn get_or_create(config: Self::Config) -> Result<Self>
    where
        Self: Sized,
    {
        let path = PathBuf::from(config.path);

        if path
            .extension()
            .is_none_or(|extension| !extension.eq_ignore_ascii_case("json"))
        {
            return Err(JsonStoreError::InvalidFileExtension(path.display().to_string()).into());
        }

        if !path.exists() {
            std::fs::write(&path, "{}")?;
        }

        Ok(Self { path })
    }

    fn add(&self, source: Source) -> Result<()> {
        todo!()
    }

    fn list(&self) -> Result<Vec<Source>> {
        todo!()
    }

    fn remove(&self, url: &str) -> Result<bool> {
        todo!()
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "a test should fail loudly at the point the assumption breaks"
)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_json_store_get_or_create_invalid_not_json_extension() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("tmp.xml");
        let config = JsonStoreConfig::new(path.to_string_lossy().into_owned());

        let store = JsonStore::get_or_create(config);

        let error = store.expect_err("Expected get_or_create to fail");
        assert_eq!(
            error.downcast_ref::<JsonStoreError>(),
            Some(&JsonStoreError::InvalidFileExtension(
                path.to_string_lossy().into_owned()
            ))
        );
        assert!(!path.exists(), "a rejected path should not be written to");
    }

    #[test]
    fn test_json_store_get_or_create_valid() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("tmp.json");
        let config = JsonStoreConfig::new(path.to_string_lossy().into_owned());

        let store = JsonStore::get_or_create(config).expect("JSON store should be created successfully");

        assert_eq!(store.path, path);
        assert!(path.is_file(), "the document should have been created");
    }
}
