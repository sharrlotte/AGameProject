use super::*;

#[test]
fn animation_ron_parses_and_is_valid() {
    let catalog: AnimCatalog = crate::assert_ron_config_parses!(AnimCatalog);
    catalog.validate().expect("animation catalog is valid");

    for role in AnimCatalog::REQUIRED_ROLES {
        assert!(catalog.clip(role).is_some(), "missing clip role {role}");
    }
    assert!(catalog.mask_group("lower").is_some());
    assert!(catalog.mask_group("upper").is_some());
    assert_eq!(catalog.ik.right_chain.len(), 3);
    assert_eq!(catalog.ik.left_chain.len(), 3);
    assert_eq!(catalog.locomotion.anchors.len(), 4);
    assert!((catalog.facing.blend_secs - 0.12).abs() < 1e-4);
}

#[test]
fn rejects_duplicate_anchor_angles() {
    let mut catalog = AnimCatalog::default();
    catalog.locomotion.anchors[1].angle_deg = 0.0;
    assert!(catalog.validate().is_err());
}

#[test]
fn rejects_out_of_range_anchor_angle() {
    let mut catalog = AnimCatalog::default();
    catalog.locomotion.anchors[0].angle_deg = 181.0;
    assert!(catalog.validate().is_err());
}

#[test]
fn rejects_missing_anchors_and_unknown_role() {
    let mut catalog = AnimCatalog::default();
    catalog.locomotion.anchors.clear();
    assert!(catalog.validate().is_err());

    let mut catalog = AnimCatalog::default();
    catalog.locomotion.anchors[0].role = "missing".to_string();
    assert!(catalog.validate().is_err());
}

#[test]
fn rejects_invalid_ik_and_pose_settings() {
    let mut catalog = AnimCatalog::default();
    catalog.ik.shoulder_settle = 1.5;
    assert!(catalog.validate().is_err());

    let mut catalog = AnimCatalog::default();
    catalog.aim.aim_height = 0.0;
    assert!(catalog.validate().is_err());

    let mut catalog = AnimCatalog::default();
    catalog.locomotion.phase_sync = false;
    assert!(catalog.validate().is_err());
}

#[test]
fn defaults_cover_cardinal_anchors() {
    let catalog = AnimCatalog::default();
    assert_eq!(catalog.locomotion.anchors.len(), 4);
    assert!(catalog.validate().is_ok());
}

#[test]
fn rejects_unknown_sprint_role() {
    let mut catalog = AnimCatalog::default();
    catalog.locomotion.anchors[0].sprint_role = Some("missing_sprint".to_string());
    assert!(catalog.validate().is_err());
}







