use super::*;

#[test]
fn weapons_ron_parses_and_is_valid() {
    let catalog: WeaponCatalog = data::assert_ron_config_parses!(WeaponCatalog);
    catalog.validate().expect("weapon catalog is valid");

    let rifle = catalog
        .weapon("assault_rifle")
        .expect("assault rifle profile");
    assert_eq!(rifle.sockets.grip, "Grip");
    assert_eq!(rifle.sockets.foregrip, "Foregrip");
    assert!(rifle.sockets.named().contains(&"Magazine"));
    assert!(rifle.sockets.named().contains(&"Muzzle"));

    for id in ["shotgun", "sniper", "rpg"] {
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
