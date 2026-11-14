use bevy::prelude::*;
use crate::common::EnemyKind;
use crate::enemies::{Enemy, FacingDirection, Fuse};

use crate::presentation::vfx::HitFlash;

use super::material::SharedVisualMaterials;
use super::mesh::SharedVisualMeshes;

#[derive(Component)]
pub struct VisualBody;

/// One mesh part of an enemy assembly, positioned in feet space.
struct PartSpec {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
}

pub fn attach_enemy_visuals(
    mut commands: Commands,
    meshes: Res<SharedVisualMeshes>,
    materials: Res<SharedVisualMaterials>,
    enemies: Query<(Entity, &Enemy), Added<Enemy>>,
) {
    for (entity, enemy) in &enemies {
        let parts = match enemy.kind {
            EnemyKind::Walker => simple_body(&meshes.walker, &materials.walker, enemy),
            EnemyKind::Runner => simple_body(&meshes.runner, &materials.runner, enemy),
            EnemyKind::Tank => tank_body(&meshes, &materials),
            EnemyKind::Spitter => spitter_body(&meshes, &materials),
            EnemyKind::Exploder => exploder_body(&meshes, &materials),
        };
        spawn_assembly(&mut commands, entity, enemy, parts);
    }
}

fn simple_body(
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    enemy: &Enemy,
) -> Vec<PartSpec> {
    vec![PartSpec {
        mesh: mesh.clone(),
        material: material.clone(),
        transform: Transform::from_xyz(0.0, enemy.radius + enemy.half_height, 0.0),
    }]
}

fn tank_body(meshes: &SharedVisualMeshes, materials: &SharedVisualMaterials) -> Vec<PartSpec> {
    let tank = &meshes.tank;
    let steel = &materials.tank.steel;
    let rust = &materials.tank.rust;
    let suit = &materials.tank.suit;
    let weak = &materials.tank.weak;

    vec![
        part(&tank.pelvis, suit, Vec3::new(0.0, 1.14, 0.0)),
        part(&tank.torso, suit, Vec3::new(0.0, 1.68, 0.0)),
        part(&tank.plate, steel, Vec3::new(0.0, 1.72, -0.40)),
        part(&tank.weak, weak, Vec3::new(0.0, 1.72, 0.40)),
        part(&tank.shoulder, steel, Vec3::new(-0.82, 2.12, 0.0)),
        part(&tank.shoulder, steel, Vec3::new(0.82, 2.12, 0.0)),
        part(&tank.head, steel, Vec3::new(0.0, 2.50, -0.02)),
        part(&tank.arm, suit, Vec3::new(-0.86, 1.56, 0.0)),
        part(&tank.arm, suit, Vec3::new(0.86, 1.56, 0.0)),
        part(&tank.leg, suit, Vec3::new(-0.24, 0.72, 0.0)),
        part(&tank.leg, suit, Vec3::new(0.24, 0.72, 0.0)),
        part(&tank.boot, rust, Vec3::new(-0.24, 0.12, -0.05)),
        part(&tank.boot, rust, Vec3::new(0.24, 0.12, -0.05)),
    ]
}

fn part(mesh: &Handle<Mesh>, material: &Handle<StandardMaterial>, at: Vec3) -> PartSpec {
    PartSpec {
        mesh: mesh.clone(),
        material: material.clone(),
        transform: Transform::from_translation(at),
    }
}

fn spitter_body(meshes: &SharedVisualMeshes, materials: &SharedVisualMaterials) -> Vec<PartSpec> {
    let spitter = &meshes.spitter;
    let skin = &materials.spitter.skin;
    let sac = &materials.spitter.sac;

    vec![
        part(&spitter.body, skin, Vec3::new(0.0, 1.00, 0.0)),
        part(&spitter.sac, sac, Vec3::new(0.0, 1.30, -0.24)),
        part(&spitter.sac, sac, Vec3::new(-0.13, 1.13, -0.22)),
        part(&spitter.sac, sac, Vec3::new(0.13, 1.13, -0.22)),
        part(&spitter.jaw, skin, Vec3::new(0.0, 1.55, -0.20)),
        part(&spitter.arm, skin, Vec3::new(-0.38, 1.02, 0.0)),
        part(&spitter.arm, skin, Vec3::new(0.38, 1.02, 0.0)),
        part(&spitter.leg, skin, Vec3::new(-0.16, 0.30, 0.0)),
        part(&spitter.leg, skin, Vec3::new(0.16, 0.30, 0.0)),
    ]
}

fn exploder_body(meshes: &SharedVisualMeshes, materials: &SharedVisualMaterials) -> Vec<PartSpec> {
    let exploder = &meshes.exploder;
    let skin = &materials.exploder.skin;
    let pustule = &materials.exploder.pustule;

    vec![
        part(&exploder.body, skin, Vec3::new(0.0, 0.82, 0.0)),
        part(&exploder.pustule, pustule, Vec3::new(0.0, 1.05, -0.42)),
        part(&exploder.pustule, pustule, Vec3::new(-0.32, 0.95, -0.28)),
        part(&exploder.pustule, pustule, Vec3::new(0.32, 0.95, -0.28)),
        part(&exploder.pustule, pustule, Vec3::new(0.0, 0.62, -0.44)),
        part(&exploder.pustule, pustule, Vec3::new(-0.40, 0.75, 0.05)),
        part(&exploder.pustule, pustule, Vec3::new(0.40, 0.75, 0.05)),
        part(&exploder.head, skin, Vec3::new(0.0, 1.32, -0.34)),
        part(&exploder.limb, skin, Vec3::new(-0.52, 0.88, 0.0)),
        part(&exploder.limb, skin, Vec3::new(0.52, 0.88, 0.0)),
        part(&exploder.limb, skin, Vec3::new(-0.22, 0.20, 0.0)),
        part(&exploder.limb, skin, Vec3::new(0.22, 0.20, 0.0)),
    ]
}

fn spawn_assembly(
    commands: &mut Commands,
    entity: Entity,
    enemy: &Enemy,
    parts: Vec<PartSpec>,
) {
    let feet_offset = -(enemy.radius + enemy.half_height);
    let root = commands
        .spawn((
            VisualBody,
            Transform::from_xyz(0.0, feet_offset, 0.0),
            Visibility::default(),
        ))
        .id();

    let mut flash_parts = Vec::with_capacity(parts.len());
    commands.entity(root).with_children(|parent| {
        for spec in parts {
            let child = parent
                .spawn((
                    Mesh3d(spec.mesh),
                    MeshMaterial3d(spec.material.clone()),
                    spec.transform,
                ))
                .id();
            flash_parts.push((child, spec.material));
        }
    });

    commands
        .entity(entity)
        .insert((
            Visibility::default(),
            HitFlash {
                timer: 0.0,
                parts: flash_parts,
            },
        ))
        .add_child(root);
}

pub fn sync_visual_facing(
    enemies: Query<(&FacingDirection, &Children), With<Enemy>>,
    mut visual_bodies: Query<&mut Transform, With<VisualBody>>,
) {
    for (facing, children) in &enemies {
        apply_facing_to_children(facing.0, children, &mut visual_bodies);
    }
}

fn apply_facing_to_children(
    dir: Vec3,
    children: &Children,
    visual_bodies: &mut Query<&mut Transform, With<VisualBody>>,
) {
    if dir.length_squared() <= 1e-4 {
        return;
    }
    let rot = Quat::from_rotation_arc(Vec3::NEG_Z, dir.normalize());
    for child in children.iter() {
        if let Ok(mut tf) = visual_bodies.get_mut(child) {
            tf.rotation = rot;
        }
    }
}

/// Scales the armed Exploder visual so the fuse telegraph reads from a distance.
pub fn pulse_fuses(
    time: Res<Time>,
    fuses: Query<(&Fuse, &Children), With<Enemy>>,
    mut visual_bodies: Query<&mut Transform, With<VisualBody>>,
) {
    let elapsed = time.elapsed_secs();
    for (fuse, children) in &fuses {
        let scale = if fuse.armed {
            let progress = 1.0 - (fuse.timer / fuse.fuse.max(0.05)).clamp(0.0, 1.0);
            let frequency = 6.0 + 18.0 * progress;
            1.0 + 0.12 * (elapsed * frequency).sin().abs()
        } else {
            1.0
        };
        for child in children.iter() {
            if let Ok(mut tf) = visual_bodies.get_mut(child) {
                tf.scale = Vec3::splat(scale);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::EnemiesConfig;

    fn app_with_assembly() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>();
        let meshes = SharedVisualMeshes::from_world(app.world_mut());
        let materials = SharedVisualMaterials::from_world(app.world_mut());
        app.insert_resource(meshes)
            .insert_resource(materials)
            .add_systems(Update, (attach_enemy_visuals, sync_visual_facing));
        app
    }

    fn spawn_tank(app: &mut App) -> Entity {
        spawn_kind(app, EnemyKind::Tank)
    }

    fn spawn_kind(app: &mut App, kind: EnemyKind) -> Entity {
        let config: EnemiesConfig = crate::assert_ron_config_parses!(EnemiesConfig);
        let arch = config
            .archetype(kind)
            .expect("authored archetype")
            .clone();
        app.world_mut()
            .spawn((
                Enemy::from_archetype(&arch),
                FacingDirection(Vec3::NEG_Z),
                Transform::default(),
            ))
            .id()
    }

    #[test]
    fn assembled_enemy_kinds_start_on_authored_materials() {
        for kind in [
            EnemyKind::Walker,
            EnemyKind::Runner,
            EnemyKind::Tank,
            EnemyKind::Spitter,
            EnemyKind::Exploder,
        ] {
            let mut app = app_with_assembly();
            let entity = spawn_kind(&mut app, kind);
            app.update();

            let flash = app
                .world()
                .get::<HitFlash>(entity)
                .unwrap_or_else(|| panic!("{kind:?} carries hit flash"));
            assert!(!flash.parts.is_empty(), "{kind:?} must assemble at least one part");
            for (part, base) in &flash.parts {
                let material = app
                    .world()
                    .get::<MeshMaterial3d<StandardMaterial>>(*part)
                    .unwrap_or_else(|| panic!("{kind:?} part material"));
                assert_ne!(material.0, Handle::default(), "{kind:?} default material");
                assert_eq!(material.0, *base, "{kind:?} part must start on its base material");
            }
        }
    }

    #[test]
    fn tank_assembly_uses_authored_materials_for_every_part() {
        let mut app = app_with_assembly();
        let tank = spawn_tank(&mut app);
        app.update();

        let flash = app
            .world()
            .get::<HitFlash>(tank)
            .expect("tank carries hit flash");
        assert_eq!(flash.parts.len(), 13, "tank part count");

        for (part, base) in &flash.parts {
            let material = app
                .world()
                .get::<MeshMaterial3d<StandardMaterial>>(*part)
                .expect("part material");
            assert_ne!(material.0, Handle::default(), "default material handle");
            assert_eq!(material.0, *base, "part must start on its base material");
        }
    }

    #[test]
    fn facing_rotates_the_visual_root() {
        let mut app = app_with_assembly();
        let tank = spawn_tank(&mut app);
        app.update();

        app.world_mut()
            .get_mut::<FacingDirection>(tank)
            .expect("facing")
            .0 = Vec3::X;
        app.update();

        let children = app.world().get::<Children>(tank).expect("root child");
        let root = children
            .iter()
            .find(|child| app.world().get::<VisualBody>(*child).is_some())
            .expect("visual body child");
        let rotation = app.world().get::<Transform>(root).expect("root transform").rotation;
        let expected = Quat::from_rotation_arc(Vec3::NEG_Z, Vec3::X);
        assert!(
            rotation.dot(expected).abs() > 1.0 - 1e-5,
            "rotation {rotation:?} expected {expected:?}"
        );
    }
}
