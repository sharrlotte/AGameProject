use std::{fmt, fs, path::Path};

use serde::{Serialize, de::DeserializeOwned};

/// A top-level file format that carries its own schema version.
///
/// Every persisted document (world saves, ship designs) implements this trait
/// so `read_document`/`write_document` can reject incompatible versions before
/// any gameplay data is trusted.
pub trait RonDocument: Serialize + DeserializeOwned {
    const VERSION: u32;
    const KIND: &'static str;
}

/// Explicit version/data envelope for documents that do not store a `version`
/// field inline.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Serialize, serde::Deserialize)]
pub struct Versioned<T> {
    pub version: u32,
    pub data: T,
}

#[derive(Debug)]
pub enum DocumentError {
    Io(std::io::Error),
    Parse(String),
    Version {
        kind: &'static str,
        expected: u32,
        found: u32,
    },
}

impl fmt::Display for DocumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::Parse(e) => write!(f, "parse error: {e}"),
            Self::Version {
                kind,
                expected,
                found,
            } => write!(
                f,
                "incompatible {kind} schema: expected version {expected}, found {found}"
            ),
        }
    }
}

impl std::error::Error for DocumentError {}

impl From<std::io::Error> for DocumentError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// Parse a versioned RON document from text, rejecting unknown versions.
pub fn parse_document<T: RonDocument>(text: &str) -> Result<T, DocumentError> {
    #[derive(serde::Deserialize)]
    struct Header {
        version: u32,
    }

    let header: Header = ron::from_str(text).map_err(|e| DocumentError::Parse(e.to_string()))?;
    if header.version != T::VERSION {
        return Err(DocumentError::Version {
            kind: T::KIND,
            expected: T::VERSION,
            found: header.version,
        });
    }

    ron::from_str(text).map_err(|e| DocumentError::Parse(e.to_string()))
}

/// Read and validate a versioned RON document.
pub fn read_document<T: RonDocument>(path: &Path) -> Result<T, DocumentError> {
    let text = fs::read_to_string(path)?;
    parse_document(&text)
}

/// Serialize a versioned RON document and write it atomically.
pub fn write_document<T: RonDocument>(path: &Path, value: &T) -> Result<(), DocumentError> {
    let text = ron::ser::to_string_pretty(value, ron::ser::PrettyConfig::default())
        .map_err(|e| DocumentError::Parse(e.to_string()))?;
    atomic_write(path, text.as_bytes())?;
    Ok(())
}

/// Write through a temporary file, keeping the previous contents as `.bak`.
pub fn atomic_write(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if path.exists() {
        let backup = path.with_extension("bak");
        fs::copy(path, backup)?;
    }

    let tmp = path.with_extension("tmp");
    fs::write(&tmp, contents)?;
    fs::rename(tmp, path)
}

/// Implement [`RonDocument`] for a persisted type.
#[macro_export]
macro_rules! ron_document {
    ($ty:ty, $version:expr, $kind:expr) => {
        impl $crate::save::document::RonDocument for $ty {
            const VERSION: u32 = $version;
            const KIND: &'static str = $kind;
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Sample {
        version: u32,
        value: u32,
    }

    ron_document!(Sample, 2, "sample");

    #[test]
    fn rejects_wrong_version() {
        let text = "(version: 1, value: 5)";
        assert!(matches!(
            parse_document::<Sample>(text),
            Err(DocumentError::Version { found: 1, .. })
        ));
    }

    #[test]
    fn round_trips_and_keeps_backup() {
        let dir = std::env::temp_dir().join(format!("ron-doc-{}", std::process::id()));
        let file = dir.join("sample.ron");
        let first = Sample {
            version: 2,
            value: 1,
        };
        write_document(&file, &first).unwrap();
        assert_eq!(read_document::<Sample>(&file).unwrap(), first);

        let second = Sample {
            version: 2,
            value: 2,
        };
        write_document(&file, &second).unwrap();
        assert_eq!(read_document::<Sample>(&file).unwrap(), second);
        assert_eq!(
            read_document::<Sample>(&dir.join("sample.bak")).unwrap(),
            first
        );
        let _ = fs::remove_dir_all(dir);
    }
}
