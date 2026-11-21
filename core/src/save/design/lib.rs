use bevy::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::config::GameConfig;
use crate::content::block::BlockRegistry;
use crate::content::ship::ShipDesign;
use crate::save::document::{DocumentError, data_dir, parse_document, read_document, write_document};

pub const INTERCEPTOR: &str = include_str!("../../../assets/config/starter/interceptor.ron");
pub const RAIDER: &str = include_str!("../../../assets/config/starter/raider.ron");
pub const B2: &str = include_str!("../../../assets/config/starter/b2.ron");
pub const BLACKBIRD: &str = include_str!("../../../assets/config/starter/blackbird.ron");
pub const AEGIS: &str = include_str!("../../../assets/config/starter/aegis.ron");
pub const VANGUARD: &str = include_str!("../../../assets/config/starter/vanguard.ron");

pub fn sandbox_designs_dir_in(base: &Path) -> PathBuf {
    base.join("sandbox_designs")
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

pub fn starter_designs() -> Vec<ShipDesign> {
    [INTERCEPTOR, RAIDER, B2, BLACKBIRD, AEGIS, VANGUARD]
        .iter()
        .filter_map(|text| match parse_document::<ShipDesign>(text) {
            Ok(design) => Some(design),
            Err(e) => {
                warn!("Starter design failed to parse: {e}");
                None
            }
        })
        .collect()
}

pub fn starter_fleet(config: &GameConfig, designs: &[ShipDesign]) -> Vec<ShipDesign> {
    config
        .sim
        .default_fleet
        .iter()
        .filter_map(|name| match designs.iter().find(|design| &design.name == name) {
            Some(design) => Some(design.clone()),
            None => {
                warn!("Default fleet design {name:?} not found; skipping");
                None
            }
        })
        .collect()
}

pub fn load_or_seed_designs(registry: &BlockRegistry) -> Vec<ShipDesign> {
    load_or_seed_designs_in(&data_dir(), registry)
}

pub fn load_or_seed_designs_in(base: &Path, registry: &BlockRegistry) -> Vec<ShipDesign> {
    let dir = sandbox_designs_dir_in(base);
    if let Err(e) = fs::create_dir_all(&dir) {
        warn!("Failed to create sandbox designs directory: {e}");
        return Vec::new();
    }

    for (name, text) in starters() {
        let path = dir.join(name);
        if let Err(e) = fs::write(&path, text) {
            warn!("Failed to refresh starter design {name}: {e}");
        }
    }

    let mut designs = read_designs_dir(&dir, registry);
    if designs.is_empty() {
        designs = starter_designs();
    }

    designs.sort_by(|a, b| a.name.cmp(&b.name));
    designs
}

pub fn read_designs_dir(dir: &Path, registry: &BlockRegistry) -> Vec<ShipDesign> {
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
            Ok(design) => match design.validate(registry) {
                Ok(()) => designs.push(design),
                Err(errors) => warn!("Skipping design {:?}: {}", path.file_name(), errors.join("; ")),
            },
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
    write_document(&sandbox_designs_dir_in(base).join(file), design)
}

pub fn upsert(ships: &mut Vec<ShipDesign>, design: ShipDesign) {
    if let Some(existing) = ships.iter_mut().find(|entry| entry.name == design.name) {
        *existing = design;
    } else {
        ships.push(design);
    }
}

pub fn remove(ships: &mut Vec<ShipDesign>, name: &str) {
    ships.retain(|design| design.name != name);
}

pub fn design_matches(design: &ShipDesign, other: &ShipDesign) -> bool {
    design.name == other.name
        && design.doctrine == other.doctrine
        && design.blocks.len() == other.blocks.len()
        && design
            .blocks
            .iter()
            .zip(&other.blocks)
            .all(|(a, b)| a.position == b.position && a.block_type == b.block_type && a.rotation == b.rotation)
}

fn starters() -> [(&'static str, &'static str); 6] {
    [
        ("interceptor.ron", INTERCEPTOR),
        ("raider.ron", RAIDER),
        ("b2.ron", B2),
        ("blackbird.ron", BLACKBIRD),
        ("aegis.ron", AEGIS),
        ("vanguard.ron", VANGUARD),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BlocksConfig;
    use crate::content::block::{BlockPlacement, BlockRegistry, Rot90};
    use crate::save::design::DESIGN_SCHEMA_VERSION;
    use bevy::asset::{AssetApp, AssetPlugin, AssetServer};
    use bevy::prelude::{AudioSource, Image};

    #[test]
    fn starters_are_valid_documents() {
        for text in [INTERCEPTOR, RAIDER, B2, BLACKBIRD, AEGIS, VANGUARD] {
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
    fn starters_pass_validate_ship() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Image>()
            .init_asset::<AudioSource>();
        let server = app.world().resource::<AssetServer>().clone();

        let config: BlocksConfig = ron::from_str(include_str!(
            "../../../assets/config/base/block.ron"
        ))
        .expect("shipped block config parses");
        let mut registry = BlockRegistry::default();
        for (key, def) in config.blocks.as_map() {
            let def = def
                .clone()
                .into_def(&server, key.clone())
                .unwrap_or_else(|error| panic!("block {key} failed to build: {error}"));
            registry.register(key, def);
        }

        for text in [INTERCEPTOR, RAIDER, B2, BLACKBIRD, AEGIS, VANGUARD] {
            let design = parse_document::<ShipDesign>(text).expect("starter parses");
            registry
                .validate_ship(&design.blocks)
                .unwrap_or_else(|errors| panic!("starter {} invalid: {errors:?}", design.name));
        }
    }

    #[test]
    fn rotated_block_and_doctrine_roundtrip_and_v2_rejected() {
        use crate::content::ship::ai::Doctrine;

        let mut design = ShipDesign::new(vec![
            BlockPlacement::new(IVec2::ZERO, "core", Rot90::R0),
            BlockPlacement::new(IVec2::X, "hull", Rot90::R90),
        ]);
        design.doctrine = Doctrine::Sniper;
        let text = ron::ser::to_string_pretty(&design, ron::ser::PrettyConfig::default())
            .expect("design serializes");
        let restored = parse_document::<ShipDesign>(&text).expect("design parses");
        assert_eq!(restored.version, DESIGN_SCHEMA_VERSION);
        assert_eq!(restored.doctrine, Doctrine::Sniper);
        assert_eq!(restored.blocks.len(), 2);
        assert_eq!(restored.blocks[0].rotation, Rot90::R0);
        assert_eq!(restored.blocks[1].rotation, Rot90::R90);

        let v2 = text.replace(&format!("version: {DESIGN_SCHEMA_VERSION}"), "version: 2");
        assert!(parse_document::<ShipDesign>(&v2).is_err());
    }

    #[test]
    fn file_name_is_sanitized() {
        let mut design = ShipDesign::new(Vec::new());
        design.name = "My Fancy Ship!".to_string();
        assert_eq!(design_file_name(&design), "my_fancy_ship");
    }
}
