use std::path::{Path, PathBuf};

use crate::save::document::{
    DocumentError, data_dir, read_document, remove_with_backup, write_document,
};

use super::component::CampaignSave;

pub fn campaign_save_path() -> PathBuf {
    campaign_save_path_in(&data_dir())
}

pub fn campaign_save_path_in(base: &Path) -> PathBuf {
    base.join("saves").join("campaign.ron")
}

pub fn load_campaign() -> CampaignSave {
    load_campaign_in(&data_dir())
}

pub fn load_campaign_in(base: &Path) -> CampaignSave {
    read_document::<CampaignSave>(&campaign_save_path_in(base)).unwrap_or_default()
}

pub fn save_campaign(campaign: &CampaignSave) -> Result<(), DocumentError> {
    write_document(&campaign_save_path(), campaign)
}

pub fn clear_campaign_save_in(base: &Path) {
    remove_with_backup(&campaign_save_path_in(base));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::save::battle::{
        BattleSave, CameraSave, ControlSave, RngSave, SAVE_SCHEMA_VERSION, WorldSave,
        battle_save_path_in, reset_campaign_data_in,
    };
    use crate::save::campaign::CAMPAIGN_SCHEMA_VERSION;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("agameproject_{label}_{nanos}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn campaign_roundtrip_and_reset_preserves_designs() {
        let root = temp_dir("campaign_reset");
        let designs_dir = root.join("designs");
        fs::create_dir_all(&designs_dir).unwrap();
        let design_file = designs_dir.join("my_ship.ron");
        fs::write(&design_file, "(version: 1)").unwrap();

        let campaign = CampaignSave {
            version: CAMPAIGN_SCHEMA_VERSION,
            current_mission: "quick-battle".to_string(),
            completed_missions: vec!["quick-battle".to_string()],
        };
        let battle = WorldSave {
            version: SAVE_SCHEMA_VERSION,
            saved_at_unix_ms: 1_700_000_000_000,
            battle: BattleSave {
                mission: "quick-battle".to_string(),
                elapsed_secs: 42.5,
                outcome: Some("Victory".to_string()),
            },
            camera: CameraSave::default(),
            control: ControlSave::default(),
            bodies: Vec::new(),
            drops: Vec::new(),
            rng: RngSave { seed: 1234 },
        };
        write_document(&campaign_save_path_in(&root), &campaign).unwrap();
        write_document(&battle_save_path_in(&root), &battle).unwrap();

        assert_eq!(load_campaign_in(&root), campaign);

        reset_campaign_data_in(&root);
        assert!(!battle_save_path_in(&root).exists());
        assert!(!campaign_save_path_in(&root).exists());
        assert!(design_file.exists());

        let _ = fs::remove_dir_all(&root);
    }
}
