use super::*;

fn authored() -> EnemiesConfig {
    crate::assert_ron_config_parses!(EnemiesConfig)
}

#[test]
fn enemies_ron_parses_and_validates() {
    let enemies = authored();
    enemies.validate().expect("authored roster is valid");
    assert_eq!(enemies.archetypes.len(), 5);
    assert_eq!(
        enemies
            .archetype(EnemyKind::Walker)
            .expect("walker")
            .pierce_cost,
        1.0
    );
    assert_eq!(
        enemies
            .archetype(EnemyKind::Runner)
            .expect("runner")
            .pierce_cost,
        0.6
    );
    assert_eq!(
        enemies.archetype(EnemyKind::Tank).expect("tank").pierce_cost,
        4.0
    );
    assert!(enemies.archetype(EnemyKind::Spitter).is_some());
    assert!(enemies.archetype(EnemyKind::Exploder).is_some());
}

#[test]
fn duplicate_archetype_is_rejected() {
    let mut enemies = authored();
    let walker = enemies
        .archetype(EnemyKind::Walker)
        .expect("walker archetype")
        .clone();
    enemies.archetypes.push(walker);
    let error = enemies.validate().expect_err("duplicate must be rejected");
    assert!(error.contains("Walker"), "unexpected error: {error}");
}

#[test]
fn missing_walker_is_rejected() {
    let mut enemies = authored();
    enemies.archetypes.retain(|arch| arch.kind != EnemyKind::Walker);
    let error = enemies.validate().expect_err("walker is required");
    assert!(error.contains("Walker"), "unexpected error: {error}");
}

#[test]
fn invalid_armor_arc_is_rejected() {
    let mut enemies = authored();
    let tank = enemies
        .archetypes
        .iter_mut()
        .find(|arch| arch.kind == EnemyKind::Tank)
        .expect("tank archetype");
    for behavior in &mut tank.behaviors {
        if let Behavior::Armor { arc_deg, .. } = behavior {
            *arc_deg = 0.0;
        }
    }
    let error = enemies.validate().expect_err("arc must be positive");
    assert!(error.contains("arc_deg"), "unexpected error: {error}");
}

#[test]
fn invalid_ranged_pool_is_rejected() {
    let mut enemies = authored();
    let spitter = enemies
        .archetypes
        .iter_mut()
        .find(|arch| arch.kind == EnemyKind::Spitter)
        .expect("spitter archetype");
    for behavior in &mut spitter.behaviors {
        if let Behavior::Ranged { pool_radius, .. } = behavior {
            *pool_radius = 0.0;
        }
    }
    let error = enemies.validate().expect_err("pool radius must be positive");
    assert!(error.contains("pool_radius"), "unexpected error: {error}");
}

#[test]
fn invalid_detonate_radius_is_rejected() {
    let mut enemies = authored();
    let exploder = enemies
        .archetypes
        .iter_mut()
        .find(|arch| arch.kind == EnemyKind::Exploder)
        .expect("exploder archetype");
    for behavior in &mut exploder.behaviors {
        if let Behavior::Detonate { radius, .. } = behavior {
            *radius = 0.0;
        }
    }
    let error = enemies.validate().expect_err("blast radius must be positive");
    assert!(error.contains("radius"), "unexpected error: {error}");
}

#[test]
fn invalid_pierce_cost_is_rejected() {
    for invalid in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        let mut enemies = authored();
        let walker = enemies
            .archetypes
            .iter_mut()
            .find(|arch| arch.kind == EnemyKind::Walker)
            .expect("walker archetype");
        walker.pierce_cost = invalid;
        let error = enemies
            .validate()
            .expect_err("zero, negative, or non-finite pierce_cost must be rejected");
        assert!(
            error.contains("pierce_cost"),
            "expected pierce_cost error for {invalid}, got: {error}"
        );
    }
}
