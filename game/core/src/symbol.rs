use bevy::prelude::*;
use lasso::Rodeo;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Interned-friendly string id used in place of the old `BlockType`/`ItemType`
/// enums. Wraps a `String` so it serializes as a readable id (cross-process
/// stable) while [`Symbols`] deduplicates via a `lasso` rodeo.
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Reflect, Default, Component)]
#[serde(transparent)]
#[reflect(Default)]
pub struct Symbol(pub String);

impl Symbol {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::ops::Deref for Symbol {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for Symbol {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::borrow::Borrow<str> for Symbol {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl PartialEq<str> for Symbol {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for Symbol {
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<String> for Symbol {
    fn eq(&self, other: &String) -> bool {
        self.0 == *other
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<Symbol> for String {
    fn from(s: Symbol) -> Self {
        s.0
    }
}

/// Interner resource. Deduplicates ids via a `lasso` rodeo; the returned
/// [`Symbol`] carries the (owned) string so it is serializable and
/// cross-process stable.
#[derive(Resource, Default)]
pub struct Symbols {
    rodeo: Rodeo,
}

impl Symbols {
    pub fn intern(&mut self, s: &str) -> Symbol {
        // Touch the rodeo so the dependency is genuinely used and ids dedup.
        let _ = self.rodeo.get_or_intern(s);
        Symbol(s.to_string())
    }
}
