use bevy::prelude::*;
use bevy_rapier2d::prelude::CollisionEvent;

use super::*;
use crate::config::GameConfigValue;
use crate::constant::LAYER_DAMAGE_TEXT;
use crate::content::block::Health;
use crate::resource::SceneState;
use crate::simulation::SimulationSet;

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<DamageEvent>()
            .add_systems(OnEnter(SceneState::Game), spawn_damage_text_pool)
            .add_systems(
                FixedUpdate,
                handle_damage_collisions
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Damage),
            )
            .add_systems(
                FixedPostUpdate,
                handle_damage_events
                    .run_if(in_state(SceneState::Game))
                    .in_set(SimulationSet::Damage),
            )
            .add_systems(Update, tick_damage_text_pool.in_set(SimulationSet::Present));
    }
}

const DAMAGE_TEXT_POOL_SIZE: usize = 64;

fn spawn_damage_text_pool(mut commands: Commands) {
    let slots = (0..DAMAGE_TEXT_POOL_SIZE)
        .map(|_| {
            commands
                .spawn((
                    DespawnOnExit(SceneState::Game),
                    Visibility::Hidden,
                    Transform::default(),
                    Text2d::new(""),
                    TextFont {
                        font_size: FontSize::Px(20.0),
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.0, 0.0)),
                    DamageTextSlot { remaining: 0.0 },
                ))
                .id()
        })
        .collect();
    commands.insert_resource(DamageTextPool { slots, next: 0 });
}

fn tick_damage_text_pool(
    time: Res<Time>,
    mut slots: Query<(&mut DamageTextSlot, &mut Visibility)>,
) {
    for (mut slot, mut visibility) in &mut slots {
        if slot.remaining <= 0.0 {
            continue;
        }
        slot.remaining -= time.delta_secs();
        if slot.remaining <= 0.0 {
            *visibility = Visibility::Hidden;
        }
    }
}

pub fn handle_damage_events(
    mut events: MessageReader<DamageEvent>,
    mut healths: Query<(&mut Health, &GlobalTransform)>,
    mut pool: ResMut<DamageTextPool>,
    mut slots: Query<(
        &mut Transform,
        &mut Text2d,
        &mut DamageTextSlot,
        &mut Visibility,
    )>,
    config: Res<GameConfigValue>,
) {
    for event in events.read() {
        if let Ok((mut health, transform)) = healths.get_mut(event.target) {
            health.health -= event.damage;

            if pool.slots.is_empty() {
                continue;
            }
            let entity = pool.slots[pool.next % pool.slots.len()];
            pool.next = (pool.next + 1) % pool.slots.len();
            if let Ok((mut slot_transform, mut text, mut slot, mut visibility)) =
                slots.get_mut(entity)
            {
                slot_transform.translation =
                    transform.translation().truncate().extend(LAYER_DAMAGE_TEXT);
                text.0 = event.damage.to_string();
                slot.remaining = config.0.damage_number_lifetime;
                *visibility = Visibility::Visible;
            }
        }
    }
}

pub fn handle_damage_collisions(
    mut collision_events: MessageReader<CollisionEvent>,
    mut writer: MessageWriter<DamageEvent>,
    damages: Query<(Entity, &Damage)>,
    healths: Query<(Entity, &Health)>,
) {
    for event in collision_events.read() {
        if let CollisionEvent::Started(e1, e2, _) = event {
            if let Ok((damage, damagec)) = damages.get(*e1)
                && let Ok((health, _)) = healths.get(*e2)
            {
                writer.write(DamageEvent {
                    attacker: damage,
                    target: health,
                    damage: damagec.damage,
                });
            } else if let Ok((damage, damagec)) = damages.get(*e2)
                && let Ok((health, _)) = healths.get(*e1)
            {
                writer.write(DamageEvent {
                    attacker: damage,
                    target: health,
                    damage: damagec.damage,
                });
            }
        }
    }
}
