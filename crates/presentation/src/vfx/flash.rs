use bevy::prelude::*;
use data::DamageMessage;
use enemies::EnemiesConfig;

use crate::scene::SharedVisualMaterials;

#[derive(Component)]
pub struct HitFlash {
    pub timer: f32,
    pub base_material: Handle<StandardMaterial>,
    pub visual_entity: Entity,
}

pub fn update_hit_flash(
    time: Res<Time>,
    enemies_cfg: Res<EnemiesConfig>,
    materials: Res<SharedVisualMaterials>,
    mut damage_reader: MessageReader<DamageMessage>,
    mut flashes: Query<&mut HitFlash>,
    mut mesh_materials: Query<&mut MeshMaterial3d<StandardMaterial>>,
) {
    for msg in damage_reader.read() {
        if let Ok(mut flash) = flashes.get_mut(msg.target) {
            flash.timer = enemies_cfg.hit_flash_duration;
            if let Ok(mut mat) = mesh_materials.get_mut(flash.visual_entity) {
                mat.0 = materials.hit_flash.clone();
            }
        }
    }

    let dt = time.delta_secs();
    for mut flash in &mut flashes {
        if flash.timer > 0.0 {
            flash.timer = (flash.timer - dt).max(0.0);
            if flash.timer <= 0.0
                && let Ok(mut mat) = mesh_materials.get_mut(flash.visual_entity)
            {
                mat.0 = flash.base_material.clone();
            }
        }
    }
}
