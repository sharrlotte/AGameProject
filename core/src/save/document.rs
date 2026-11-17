use std::error::Error;
use std::io;
use std::{fmt, fs, path::Path};

use ron::ser::{PrettyConfig, to_string_pretty};
use serde::{Serialize, de::DeserializeOwned};

pub trait RonDocument: Serialize + DeserializeOwned {
    const VERSION: u32;
    const KIND: &'static str;
}

#[derive(Debug)]
pub enum DocumentError {
    Io(io::Error),
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

impl Error for DocumentError {}

impl From<io::Error> for DocumentError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

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

pub fn read_document<T: RonDocument>(path: &Path) -> Result<T, DocumentError> {
    let text = fs::read_to_string(path)?;
    parse_document(&text)
}

pub fn write_document<T: RonDocument>(path: &Path, value: &T) -> Result<(), DocumentError> {
    let text = to_string_pretty(value, PrettyConfig::default())
        .map_err(|e| DocumentError::Parse(e.to_string()))?;
    atomic_write(path, text.as_bytes())?;
    Ok(())
}

pub fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
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
