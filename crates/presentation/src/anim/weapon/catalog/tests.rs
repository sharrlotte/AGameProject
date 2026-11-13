use super::*;

#[test]
fn weapons_ron_parses_and_is_valid() {
    let catalog: WeaponCatalog = data::assert_ron_config_parses!(WeaponCatalog);
    catalog.validate().expect("weapon catalog is valid");

    let rifle = catalog
        .weapon("m16")
        .expect("m16 profile");
    assert_eq!(rifle.sockets.grip, "Grip");
    assert_eq!(rifle.sockets.foregrip, "Foregrip");
    assert!(rifle.sockets.named().contains(&"Magazine"));
    assert!(rifle.sockets.named().contains(&"Muzzle"));

    for id in ["m1911", "mp5", "mp7", "scar"] {
        assert!(catalog.weapon(id).is_some(), "missing weapon profile {id}");
    }
}

#[test]
fn socket_names_are_fixed_by_default() {
    let sockets = WeaponSockets::default();
    assert_eq!(sockets.named(), ["Grip", "Foregrip", "Magazine", "Muzzle"]);
}

#[test]
fn defaults_describe_one_weapon() {
    let catalog = WeaponCatalog::default();
    assert!(catalog.validate().is_ok());
    assert!(catalog.weapon("assault_rifle").is_some());
    assert_eq!(catalog.anchor.offset, [0.0, 1.0, 0.0]);
}

#[test]
fn rejects_duplicate_or_empty_ids() {
    let mut catalog = WeaponCatalog::default();
    catalog.weapons.push(WeaponProfile::default());
    assert!(catalog.validate().is_err());

    let mut catalog = WeaponCatalog::default();
    catalog.weapons[0].id.clear();
    assert!(catalog.validate().is_err());

    let mut catalog = WeaponCatalog::default();
    catalog.weapons.clear();
    assert!(catalog.validate().is_err());
}

#[test]
fn rejects_invalid_timelines_and_sockets() {
    let mut catalog = WeaponCatalog::default();
    catalog.weapons[0].recoil.recover_secs = 0.0;
    assert!(catalog.validate().is_err());

    let mut catalog = WeaponCatalog::default();
    catalog.weapons[0].reload.hand_return = 1.5;
    assert!(catalog.validate().is_err());

    let mut catalog = WeaponCatalog::default();
    catalog.weapons[0].swap.lower_secs = 0.0;
    catalog.weapons[0].swap.raise_secs = 0.0;
    assert!(catalog.validate().is_err());

    let mut catalog = WeaponCatalog::default();
    catalog.weapons[0].sway.frequency = 0.0;
    assert!(catalog.validate().is_err());

    let mut catalog = WeaponCatalog::default();
    catalog.weapons[0].sockets.foregrip.clear();
    assert!(catalog.validate().is_err());

    let mut catalog = WeaponCatalog::default();
    catalog.weapons[0].socket_overrides.push(SocketOverride {
        socket: "Grip".to_string(),
        offset: [0.0, 0.0, 0.0],
        rotation_deg: [0.0, 0.0, 0.0],
    });
    assert!(catalog.validate().is_ok());
}

#[test]
fn socket_override_lookup_matches_name() {
    let mut profile = WeaponProfile::default();
    profile.socket_overrides.push(SocketOverride {
        socket: "Foregrip".to_string(),
        offset: [0.0, 0.1, 0.0],
        rotation_deg: [0.0; 3],
    });
    assert!(profile.socket_override("Foregrip").is_some());
    assert!(profile.socket_override("Muzzle").is_none());
}

#[test]
fn converted_models_map_forward_neg_y_up_z_into_weapon_space() {
    let catalog: WeaponCatalog = data::assert_ron_config_parses!(WeaponCatalog);

    for id in ["m16", "m1911", "mp5", "scar"] {
        let weapon = catalog.weapon(id).expect(id);
        assert_eq!(weapon.model_space, ModelSpace::ForwardNegYUpZ, "{id}");
        let rotation = weapon.model_space.rotation();
        assert!((rotation * Vec3::NEG_Y - Vec3::NEG_Z).length() < 1e-4, "{id} forward");
        assert!((rotation * Vec3::Z - Vec3::Y).length() < 1e-4, "{id} up");
    }

    let mp7 = catalog.weapon("mp7").expect("mp7");
    assert_eq!(mp7.model_space, ModelSpace::Weapon);
}

#[test]
fn model_transform_combines_space_rotation_and_scale() {
    let profile = WeaponProfile {
        model_space: ModelSpace::ForwardNegYUpZ,
        model_scale: 2.0,
        ..WeaponProfile::default()
    };
    let transform = profile.model_transform();

    assert!((transform.rotation * Vec3::NEG_Y - Vec3::NEG_Z).length() < 1e-4);
    assert!((transform.scale - Vec3::splat(2.0)).length() < 1e-4);
}

#[test]
fn grip_min_support_separation_is_validated() {
    let mut catalog = WeaponCatalog::default();
    let mut grip = WeaponGripConfig::default();
    grip.min_support_separation = -0.1;
    catalog.weapons[0].grip = Some(grip);
    assert!(catalog.validate().is_err());
}
