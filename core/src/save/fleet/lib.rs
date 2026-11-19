use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use bevy::prelude::*;

use crate::config::GameConfig;
use crate::content::ship::ShipDesign;
use crate::save::design::ShipCatalog;
use crate::save::document::{DocumentError, data_dir, read_document, write_document};

use super::component::{FLEET_SCHEMA_VERSION, FleetSave};

pub fn fleet_path() -> PathBuf {
    data_dir().join("fleet.ron")
}

pub fn read_fleet_from(path: &Path) -> Option<Vec<ShipDesign>> {
    match read_document::<FleetSave>(path) {
        Ok(document) => Some(document.ships),
        Err(DocumentError::Io(error)) if error.kind() == ErrorKind::NotFound => None,
        Err(error) => {
            warn!("Failed to read deployed fleet: {error}");
            None
        }
    }
}

pub fn write_fleet_to(path: &Path, ships: &[ShipDesign]) -> Result<(), DocumentError> {
    write_document(
        path,
        &FleetSave {
            version: FLEET_SCHEMA_VERSION,
            ships: ships.to_vec(),
        },
    )
}

pub fn write_fleet(ships: &[ShipDesign]) -> Result<(), DocumentError> {
    write_fleet_to(&fleet_path(), ships)
}

pub fn seed_default_fleet(config: &GameConfig, catalog: &ShipCatalog) -> Vec<ShipDesign> {
    config
        .default_fleet
        .iter()
        .filter_map(|name| match catalog.designs().iter().find(|d| &d.name == name) {
            Some(design) => Some(design.clone()),
            None => {
                warn!("Default fleet design {name:?} not found; skipping");
                None
            }
        })
        .collect()
}

pub fn load_or_seed_fleet_from(
    config: &GameConfig,
    catalog: &ShipCatalog,
    path: &Path,
) -> Vec<ShipDesign> {
    match read_fleet_from(path) {
        Some(fleet) if !fleet.is_empty() => fleet,
        Some(_) => {
            warn!("Deployed fleet was empty; restoring config defaults");
            seed_default_fleet(config, catalog)
        }
        None => seed_default_fleet(config, catalog),
    }
}

pub fn load_or_seed_fleet(config: &GameConfig, catalog: &ShipCatalog) -> Vec<ShipDesign> {
    load_or_seed_fleet_from(config, catalog, &fleet_path())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::block::{BlockPlacement, Rot90};
    use crate::content::ship::ai::Doctrine;
    use std::env;
    use std::fs;
    use std::process;

    fn temp_path(name: &str) -> PathBuf {
        env::temp_dir().join(format!("fleet-{}-{}", process::id(), name))
    }

    fn design(name: &str) -> ShipDesign {
        let mut design = ShipDesign::new(vec![BlockPlacement::new(
            IVec2::ZERO,
            "hull",
            Rot90::R0,
        )]);
        design.name = name.to_string();
        design
    }

    fn catalog(designs: Vec<ShipDesign>) -> ShipCatalog {
        ShipCatalog { designs }
    }

    #[test]
    fn fleet_round_trips() {
        let dir = temp_path("round-trip");
        let path = dir.join("fleet.ron");
        let fleet = vec![design("Interceptor"), design("B2")];

        write_fleet_to(&path, &fleet).expect("write fleet");
        assert_eq!(read_fleet_from(&path), Some(fleet.clone()));

        let raider = design("Raider");
        write_fleet_to(&path, std::slice::from_ref(&raider)).expect("rewrite fleet");
        assert_eq!(read_fleet_from(&path), Some(vec![raider]));
        assert!(dir.join("fleet.bak").exists());

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn persisted_fleet_wins_over_config() {
        let dir = temp_path("precedence");
        let path = dir.join("fleet.ron");
        let interceptor = design("Interceptor");
        let b2 = design("B2");
        let config = GameConfig {
            default_fleet: vec!["Interceptor".to_string()],
            ..default()
        };
        write_fleet_to(&path, std::slice::from_ref(&b2)).expect("write fleet");

        let fleet = load_or_seed_fleet_from(&config, &catalog(vec![interceptor, b2.clone()]), &path);
        assert_eq!(fleet, vec![b2]);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn empty_fleet_is_reseeded() {
        let dir = temp_path("empty");
        let path = dir.join("fleet.ron");
        let interceptor = design("Interceptor");
        let config = GameConfig {
            default_fleet: vec!["Interceptor".to_string()],
            ..default()
        };
        write_fleet_to(&path, &[]).expect("write empty fleet");

        let fleet = load_or_seed_fleet_from(&config, &catalog(vec![interceptor.clone()]), &path);
        assert_eq!(fleet, vec![interceptor]);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn unknown_default_design_is_skipped() {
        let dir = temp_path("unknown");
        let path = dir.join("fleet.ron");
        let interceptor = design("Interceptor");
        let config = GameConfig {
            default_fleet: vec!["Interceptor".to_string(), "Missing".to_string()],
            ..default()
        };

        let fleet = load_or_seed_fleet_from(&config, &catalog(vec![interceptor.clone()]), &path);
        assert_eq!(fleet, vec![interceptor]);

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn upsert_replaces_by_name_and_appends_new() {
        let mut fleet = vec![design("Interceptor")];
        let mut edited = design("Interceptor");
        edited.doctrine = Doctrine::Brawler;
        upsert(&mut fleet, edited.clone());
        assert_eq!(fleet.len(), 1);
        assert_eq!(fleet[0].doctrine, Doctrine::Brawler);

        upsert(&mut fleet, design("B2"));
        assert_eq!(fleet.len(), 2);
    }

    #[test]
    fn remove_deletes_only_the_named_design() {
        let mut fleet = vec![design("Interceptor"), design("B2")];
        remove(&mut fleet, "Interceptor");
        assert_eq!(fleet.len(), 1);
        assert_eq!(fleet[0].name, "B2");
    }

    #[test]
    fn block_ids_are_ignored_when_comparing_designs() {
        let a = design("Interceptor");
        let mut b = a.clone();
        b.blocks[0].id = uuid::Uuid::now_v7();
        assert!(design_matches(&a, &b));

        b.doctrine = Doctrine::Sniper;
        assert!(!design_matches(&a, &b));
    }
}
