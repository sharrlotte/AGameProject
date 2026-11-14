use bevy::prelude::*;
use crate::message::messages::DamageMessage;
use crate::enemies::EnemiesConfig;

use crate::presentation::scene::SharedVisualMaterials;

#[derive(Component)]
pub struct HitFlash {
    pub timer: f32,
    pub parts: Vec<(Entity, Handle<StandardMaterial>)>,
}

pub fn update_hit_flash(
    time: Res<Time>,
    enemies_cfg: Res<EnemiesConfig>,
    materials: Res<SharedVisualMaterials>,
    mut damage_reader: MessageReader<DamageMessage>,
    mut flashes: Query<&mut HitFlash>,
    mut mesh_materials: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    let flash_material = materials.hit_flash.clone();

    for msg in damage_reader.read() {
        if let Ok(mut flash) = flashes.get_mut(msg.target) {
            flash.timer = enemies_cfg.hit_flash_duration;
            for (part, _) in &flash.parts {
                if let Ok(mut material) = mesh_materials.get_mut(*part) {
                    material.0 = flash_material.clone();
                }
            }
        }
    }

    let dt = time.delta_secs();
    for mut flash in &mut flashes {
        if flash.timer <= 0.0 {
            continue;
        }
        flash.timer = (flash.timer - dt).max(0.0);
        if flash.timer > 0.0 {
            continue;
        }
        for (part, base) in &flash.parts {
            if let Ok(mut material) = mesh_materials.get_mut(*part) {
                material.0 = base.clone();
            }
        }
    }
}
