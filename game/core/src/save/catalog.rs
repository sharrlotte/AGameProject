use bevy::prelude::*;
use std::{fs, path::Path};

use crate::content::ship::ShipDesign;
use crate::save::document::{DocumentError, parse_document, read_document, write_document};
use crate::save::io::data_dir;
use crate::save::schema::DESIGN_SCHEMA_VERSION;

crate::ron_document!(ShipDesign, DESIGN_SCHEMA_VERSION, "ship design");

const INTERCEPTOR: &str = include_str!("../../assets/config/starter/interceptor.ron");
const RAIDER: &str = include_str!("../../assets/config/starter/raider.ron");

/// Ship designs on disk, one self-contained RON file per design.
pub struct ShipCatalog;

impl ShipCatalog {
    pub fn designs_dir() -> std::path::PathBuf {
        data_dir().join("designs")
    }

    /// File stem used for a design. The name is the only user-facing identity.
    pub fn file_name(design: &ShipDesign) -> String {
        design
            .name
            .trim()
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect::<String>()
            .trim_matches('_')
            .to_string()
    }

    /// Load every design, seeding the bundled starters on first run.
    pub fn load_or_seed() -> Vec<ShipDesign> {
        let dir = Self::designs_dir();
        if let Err(e) = fs::create_dir_all(&dir) {
            warn!("Failed to create designs directory: {e}");
            return Vec::new();
        }

        // Starter files are rewritten on every run so a schema upgrade cannot
        // leave stale v1 profiles on disk. User-created designs are untouched.
        for (name, text) in [("interceptor.ron", INTERCEPTOR), ("raider.ron", RAIDER)] {
            let path = dir.join(name);
            if let Err(e) = fs::write(&path, text) {
                warn!("Failed to refresh starter design {name}: {e}");
            }
        }

        let mut designs = Self::read_dir(&dir);
        if designs.is_empty() {
            for (name, text) in [("interceptor.ron", INTERCEPTOR), ("raider.ron", RAIDER)] {
                match parse_document::<ShipDesign>(text) {
                    Ok(design) => designs.push(design),
                    Err(e) => warn!("Starter design {name} failed to parse: {e}"),
                }
            }
        }

        designs.sort_by(|a, b| a.name.cmp(&b.name));
        designs
    }

    pub fn read_dir(dir: &Path) -> Vec<ShipDesign> {
        let mut designs = Vec::new();
        let Ok(entries) = fs::read_dir(dir) else {
            return designs;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("ron") {
                continue;
            }
            match read_document::<ShipDesign>(&path) {
                Ok(design) => designs.push(design),
                Err(e) => warn!("Skipping design {:?}: {e}", path.file_name()),
            }
        }

        designs.sort_by(|a, b| a.name.cmp(&b.name));
        designs
    }

    pub fn save(design: &ShipDesign) -> Result<(), DocumentError> {
        let file = format!("{}.ron", Self::file_name(design));
        write_document(&Self::designs_dir().join(file), design)
    }

    pub fn load(name: &str) -> Result<ShipDesign, DocumentError> {
        read_document(&Self::designs_dir().join(format!("{name}.ron")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::block::BlockPlacement;

    #[test]
    fn starters_are_valid_documents() {
        for text in [INTERCEPTOR, RAIDER] {
            let design = parse_document::<ShipDesign>(text).expect("starter parses");
            assert_eq!(design.version, DESIGN_SCHEMA_VERSION);
            assert!(!design.blocks.is_empty());
            assert!(
                design
                    .blocks
                    .iter()
                    .any(|b: &BlockPlacement| b.block_type == "core")
            );
        }
    }

    #[test]
    fn file_name_is_sanitized() {
        let mut design = ShipDesign::new(Vec::new());
        design.name = "My Fancy Ship!".to_string();
        assert_eq!(ShipCatalog::file_name(&design), "my_fancy_ship");
    }
}
