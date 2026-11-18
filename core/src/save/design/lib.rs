use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::content::ship::ShipDesign;
use crate::save::document::{DocumentError, data_dir, parse_document, read_document, write_document};

pub const INTERCEPTOR: &str = include_str!("../../../assets/config/starter/interceptor.ron");
pub const RAIDER: &str = include_str!("../../../assets/config/starter/raider.ron");

pub fn designs_dir() -> PathBuf {
    designs_dir_in(&data_dir())
}

pub fn designs_dir_in(base: &Path) -> PathBuf {
    base.join("designs")
}

pub fn design_file_name(design: &ShipDesign) -> String {
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

pub fn load_or_seed_designs() -> Vec<ShipDesign> {
    load_or_seed_designs_in(&data_dir())
}

pub fn load_or_seed_designs_in(base: &Path) -> Vec<ShipDesign> {
    let dir = designs_dir_in(base);
    if let Err(e) = fs::create_dir_all(&dir) {
        warn!("Failed to create designs directory: {e}");
        return Vec::new();
    }

    for (name, text) in [("interceptor.ron", INTERCEPTOR), ("raider.ron", RAIDER)] {
        let path = dir.join(name);
        if let Err(e) = fs::write(&path, text) {
            warn!("Failed to refresh starter design {name}: {e}");
        }
    }

    let mut designs = read_designs_dir(&dir);
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

pub fn read_designs_dir(dir: &Path) -> Vec<ShipDesign> {
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

pub fn save_design_file(design: &ShipDesign) -> Result<(), DocumentError> {
    save_design_file_in(&data_dir(), design)
}

pub fn save_design_file_in(base: &Path, design: &ShipDesign) -> Result<(), DocumentError> {
    let file = format!("{}.ron", design_file_name(design));
    write_document(&designs_dir_in(base).join(file), design)
}

#[allow(dead_code)]
pub fn load_design_file(name: &str) -> Result<ShipDesign, DocumentError> {
    read_document(&designs_dir().join(format!("{name}.ron")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::block::{BlockPlacement, Rot90};
    use crate::save::design::DESIGN_SCHEMA_VERSION;

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
    fn rotated_block_roundtrips() {
        let design = ShipDesign::new(vec![
            BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
            BlockPlacement::new(IVec2::X, "hull", Rot90::R90),
        ]);
        let text = ron::ser::to_string_pretty(&design, ron::ser::PrettyConfig::default())
            .expect("design serializes");
        let restored = parse_document::<ShipDesign>(&text).expect("design parses");
        assert_eq!(restored.version, DESIGN_SCHEMA_VERSION);
        assert_eq!(restored.blocks.len(), 2);
        assert_eq!(restored.blocks[0].rotation, Rot90::R0);
        assert_eq!(restored.blocks[1].rotation, Rot90::R90);
    }

    #[test]
    fn file_name_is_sanitized() {
        let mut design = ShipDesign::new(Vec::new());
        design.name = "My Fancy Ship!".to_string();
        assert_eq!(design_file_name(&design), "my_fancy_ship");
    }
}
