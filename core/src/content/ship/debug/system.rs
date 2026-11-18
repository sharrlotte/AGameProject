use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::constant::LAYER_SHIP_UI;
use crate::content::block::feature::ammo::{Ammo, AmmoState};
use crate::content::block::feature::turret::{TurretBarrel, TurretState};
use crate::content::block::BlockOwner;
use crate::content::ship::ai::AiRuntime;
use crate::content::ship::stat::ShipStats;
use crate::scene::widget::ButtonVisualState;
use crate::content::ship::{
    BehaviorGeometry, BoundingBox, Ship, ShipBehavior, behavior_geometry, ship_center,
};

use super::*;

pub fn ignore_hud_picking(
    mut commands: Commands,
    nodes: Query<(Entity, &Node), (Without<ButtonVisualState>, Without<Pickable>)>,
) {
    for (entity, _) in &nodes {
        commands.entity(entity).insert(Pickable::IGNORE);
    }
}

pub fn on_ship_click(
    click: On<PointerClick>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut events: MessageWriter<ShipDebugSelected>,
) {
    if click.button != PointerButton::Primary || keyboard.pressed(KeyCode::ControlLeft) {
        return;
    }

    events.write(ShipDebugSelected {
        entity: click.event().event_target(),
    });
}

pub fn clear_selection_on_escape(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    selection: Res<ShipDebugSelection>,
    mut events: MessageWriter<ShipDebugSelectionCleared>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        events.write(ShipDebugSelectionCleared);

        if !selection.selected.is_empty() {
            keyboard.clear_just_pressed(KeyCode::Escape);
        }
    }
}

pub fn apply_selection(
    mut selection: ResMut<ShipDebugSelection>,
    mut toggles: MessageReader<ShipDebugSelected>,
    mut clears: MessageReader<ShipDebugSelectionCleared>,
) {
    for event in toggles.read() {
        if !selection.selected.remove(&event.entity) {
            selection.selected.insert(event.entity);
        }
    }

    if clears.read().next().is_some() {
        selection.selected.clear();
    }
}

pub fn remove_despawned_ships(
    mut selection: ResMut<ShipDebugSelection>,
    mut removed: RemovedComponents<Ship>,
) {
    for entity in removed.read() {
        selection.selected.remove(&entity);
    }
}

pub fn update_ship_ui(
    mut ui_roots: Query<(&ShipUIRoot, &mut Visibility, &mut Text2d, &mut Transform)>,
    ships: Query<(
        &Ship,
        &AiRuntime,
        Option<&ShipBehavior>,
        &GlobalTransform,
        &ShipStats,
    )>,
    blocks: Query<(&BlockOwner, Option<&Ammo>, Option<&AmmoState>)>,
    selection: Res<ShipDebugSelection>,
) {
    let mut data: HashMap<Entity, String> = HashMap::new();

    for (owner, ammo, ammo_state) in &blocks {
        if !is_selected(&selection, owner.body) {
            continue;
        }

        let mut dump = String::new();

        if let Some(ammo) = ammo {
            dump.push_str(&format!("\n{ammo:?}"));
        }

        if let Some(ammo_state) = ammo_state {
            dump.push_str(&format!("\n{ammo_state:?}"));
        }

        if !dump.is_empty() {
            data.entry(owner.body)
                .and_modify(|current| current.push_str(&dump))
                .or_insert(dump);
        }
    }

    for (ui_root, mut visibility, mut text, mut transform) in &mut ui_roots {
        if !is_selected(&selection, ui_root.ship) {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }

        let Ok((ship, ai_runtime, behavior, global_transform, stats)) = ships.get(ui_root.ship)
        else {
            *visibility = Visibility::Hidden;
            continue;
        };

        *visibility = Visibility::Visible;

        let dump = data.get(&ui_root.ship).map_or("", String::as_str);
        let next_text = format!(
            "Ship: {ship:?}\nC: {}\nB: {behavior:?}\nCARGO {}/{} {:?}\nS: {stats:?}\n{dump}",
            ai_runtime.current.label(),
            stats.cargo_used, stats.cargo_capacity, stats.cargo_items,
        );
        if text.0 != next_text {
            text.0 = next_text;
        }

        let next_transform = Transform::from_rotation(global_transform.rotation().inverse())
            .with_translation(Vec2::ZERO.extend(LAYER_SHIP_UI));
        if transform.rotation != next_transform.rotation {
            transform.rotation = next_transform.rotation;
        }
        if transform.translation != next_transform.translation {
            transform.translation = next_transform.translation;
        }
    }
}

pub fn draw_ship_debug_gizmos(
    mut gizmos: Gizmos,
    selection: Res<ShipDebugSelection>,
    ships: Query<(
        &Ship,
        &Transform,
        &BoundingBox,
        Option<&ShipBehavior>,
        &Velocity,
        &ExternalForce,
    )>,
    turrets: Query<(&TurretState, &TurretBarrel, &BlockOwner)>,
    global_transforms: Query<&GlobalTransform>,
) {
    for entity in &selection.selected {
        let Ok((ship, transform, bounds, behavior, velocity, force)) = ships.get(*entity) else {
            continue;
        };

        let center = ship_center(transform, bounds);
        let core = transform
            .transform_point(ship.core_position.extend(0.0))
            .truncate();

        if let Some(behavior) = behavior {
            match behavior_geometry(behavior, transform, bounds) {
                BehaviorGeometry::Approach(approach) => {
                    gizmos.line_2d(
                        approach.current,
                        approach.target,
                        Color::srgba(0.0, 1.0, 1.0, 0.5),
                    );
                    gizmos.circle_2d(approach.target, 30.0, Color::srgb(0.0, 1.0, 1.0));
                }
                BehaviorGeometry::GoAround(around) => {
                    gizmos.line_2d(
                        center,
                        around.next_position,
                        Color::srgba(0.0, 1.0, 0.0, 0.5),
                    );
                    gizmos.circle_2d(
                        around.ring_center,
                        around.ring_radius,
                        Color::srgb(1.0, 1.0, 0.0),
                    );
                    gizmos.circle_2d(around.next_position, 30.0, Color::srgb(0.0, 1.0, 0.0));
                }
                BehaviorGeometry::AimAt { current, target } => {
                    gizmos.line_2d(current, target, Color::srgba(1.0, 0.5, 0.0, 0.5));
                    gizmos.circle_2d(target, 30.0, Color::srgb(1.0, 0.5, 0.0));
                }
            }
        }

        let half = bounds.size / 2.0;
        let corners = [
            Vec2::new(-half.x, -half.y),
            Vec2::new(half.x, -half.y),
            Vec2::new(half.x, half.y),
            Vec2::new(-half.x, half.y),
        ]
        .map(|corner| transform.transform_point(corner.extend(0.0)).truncate());
        gizmos.linestrip_2d(
            [corners[0], corners[1], corners[2], corners[3], corners[0]],
            Color::srgb(0.6, 0.7, 0.9),
        );
        gizmos.circle_2d(core, 8.0, Color::srgb(1.0, 1.0, 1.0));

        let heading = transform.local_y().truncate();
        gizmos.line_2d(center, center + heading * 80.0, Color::srgb(0.3, 0.7, 1.0));
        gizmos.line_2d(center, center + velocity.linear, Color::srgb(0.0, 1.0, 0.4));
        gizmos.line_2d(center, center + force.force * 0.01, Color::srgb(1.0, 0.4, 0.2));
    }

    for (state, barrel, owner) in &turrets {
        if !is_selected(&selection, owner.body) {
            continue;
        }

        let Ok(barrel_transform) = global_transforms.get(barrel.0) else {
            continue;
        };

        let origin = barrel_transform.translation().truncate();
        gizmos.line_2d(origin, state.aim_point, Color::srgba(1.0, 0.8, 0.0, 0.8));
        gizmos.circle_2d(state.aim_point, 12.0, Color::srgb(1.0, 0.8, 0.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::selectable::EntitySelectEvent;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ShipDebugSelection>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<ShipDebugSelected>()
            .add_message::<ShipDebugSelectionCleared>()
            .add_message::<EntitySelectEvent>()
            .add_systems(
                Update,
                (clear_selection_on_escape, apply_selection, remove_despawned_ships).chain(),
            );
        app
    }

    fn ship(app: &mut App) -> Entity {
        app.world_mut().spawn(Ship::default()).id()
    }

    fn toggle(app: &mut App, entity: Entity) {
        app.world_mut()
            .resource_mut::<Messages<ShipDebugSelected>>()
            .write(ShipDebugSelected { entity });
        app.update();
    }

    #[test]
    fn click_toggles_membership() {
        let mut app = test_app();
        let entity = ship(&mut app);

        toggle(&mut app, entity);
        assert!(app.world().resource::<ShipDebugSelection>().selected.contains(&entity));

        toggle(&mut app, entity);
        assert!(!app.world().resource::<ShipDebugSelection>().selected.contains(&entity));
    }

    #[test]
    fn several_ships_stay_selected() {
        let mut app = test_app();
        let first = ship(&mut app);
        let second = ship(&mut app);

        toggle(&mut app, first);
        toggle(&mut app, second);

        let selection = app.world().resource::<ShipDebugSelection>();
        assert!(selection.selected.contains(&first));
        assert!(selection.selected.contains(&second));
    }

    #[test]
    fn escape_clears_selection() {
        let mut app = test_app();
        let entity = ship(&mut app);
        toggle(&mut app, entity);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();
        app.update();

        assert!(app.world().resource::<ShipDebugSelection>().selected.is_empty());
    }

    #[test]
    fn despawn_removes_ship_from_selection() {
        let mut app = test_app();
        let entity = ship(&mut app);
        toggle(&mut app, entity);

        app.world_mut().entity_mut(entity).despawn();
        app.update();

        assert!(app.world().resource::<ShipDebugSelection>().selected.is_empty());
    }

    #[test]
    fn selection_does_not_emit_gameplay_select_event() {
        let mut app = test_app();
        let entity = ship(&mut app);

        toggle(&mut app, entity);

        assert!(app.world().resource::<Messages<EntitySelectEvent>>().is_empty());
    }
}
