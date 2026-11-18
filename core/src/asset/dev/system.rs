use std::fs;
use std::path::Path;

use bevy::prelude::*;

use super::component::AssetNameIssue;
use super::lib::is_valid_asset_name;

pub fn validate_assets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
    let mut issues = Vec::new();
    collect_invalid_names(&root, &mut issues);

    for issue in issues {
        warn!("invalid asset name: {}", issue.0);
    }
}

fn collect_invalid_names(root: &Path, issues: &mut Vec<AssetNameIssue>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_invalid_names(&path, issues);
            continue;
        }

        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };

        if !is_valid_asset_name(name) {
            issues.push(AssetNameIssue(name.to_owned()));
        }
    }
}
