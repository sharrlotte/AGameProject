use std::collections::HashMap;

use bevy::prelude::*;

use crate::content::ship::Ship;
use crate::content::ship::ai::{Capability, TacticalBoard};
use crate::content::team::{DEFAULT_TEAM_ID, Team};
use crate::resource::{CameraTarget, ControlledShip, SceneState};

use super::component::{
    LostContact, MarkerSpec, MinimapMarker, MinimapMarkerLayer, MinimapRoot, MinimapState,
};
use super::config::{MinimapConfig, MinimapConfigValue};

const MINIMAP_BACKGROUND: Color = Color::srgba(0.02, 0.03, 0.05, 0.65);
const MINIMAP_BORDER: Color = Color::srgba(0.85, 0.85, 0.85, 1.0);
const GLOW_ALPHA: f32 = 0.65;

pub fn setup_minimap(mut commands: Commands, config: Res<MinimapConfigValue>) {
    let cfg = config.0.clone();
    commands.insert_resource(MinimapState::default());

    commands
        .spawn((
            MinimapRoot,
            DespawnOnExit(SceneState::Game),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(cfg.margin_px),
                right: Val::Px(cfg.margin_px),
                width: Val::Px(cfg.size_px),
                height: Val::Px(cfg.size_px),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::MAX,
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(MINIMAP_BACKGROUND),
            BorderColor::all(MINIMAP_BORDER),
            GlobalZIndex(10),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                MinimapMarkerLayer,
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|layer| {
                for slot in 0..cfg.max_markers {
                    spawn_marker(layer, &cfg, slot);
                }
            });
        });
}

pub fn sync_marker_pool(
    mut commands: Commands,
    config: Res<MinimapConfigValue>,
    layer: Single<Entity, With<MinimapMarkerLayer>>,
    markers: Query<(Entity, &MinimapMarker)>,
) {
    let cfg = &config.0;

    if markers.iter().count() == cfg.max_markers {
        return;
    }

    for (entity, _) in &markers {
        commands.entity(entity).despawn();
    }

    commands.entity(*layer).with_children(|layer| {
        for slot in 0..cfg.max_markers {
            spawn_marker(layer, cfg, slot);
        }
    });
}

#[allow(clippy::too_many_arguments)]
pub fn update_minimap(
    time: Res<Time>,
    config: Res<MinimapConfigValue>,
    board: Res<TacticalBoard>,
    controlled: Res<ControlledShip>,
    camera_target: Res<CameraTarget>,
    camera: Single<&GlobalTransform, With<Camera2d>>,
    ships: Query<(Entity, &Ship, &Team, &GlobalTransform, Option<&Capability>)>,
    mut state: ResMut<MinimapState>,
    mut colors: Local<HashMap<Entity, Color>>,
    mut player_radars: Local<HashMap<Entity, f32>>,
    mut current_contacts: Local<HashMap<Entity, (Vec2, Color)>>,
    mut render_specs: Local<Vec<MarkerSpec>>,
    mut root_query: Query<&mut Node, With<MinimapRoot>>,
    mut markers: Query<
        (
            &MinimapMarker,
            &mut Node,
            &mut BackgroundColor,
            &mut BoxShadow,
            &mut Visibility,
        ),
        Without<MinimapRoot>,
    >,
) {
    let cfg = &config.0;
    let radius_px = cfg.radius_px();
    let dot_size = cfg.dot_size_px;
    let center = camera.translation().truncate();

    if let Ok(mut root) = root_query.single_mut()
        && (config.is_changed() || root.width != Val::Px(cfg.size_px))
    {
        root.width = Val::Px(cfg.size_px);
        root.height = Val::Px(cfg.size_px);
        root.top = Val::Px(cfg.margin_px);
        root.right = Val::Px(cfg.margin_px);
    }

    colors.clear();
    player_radars.clear();
    current_contacts.clear();
    render_specs.clear();

    let ally_color = Team::default().color(1.0);
    let mut active_radar_range_candidates = false;

    for (entity, ship, team, transform, capability) in &ships {
        colors.insert(entity, team.color(1.0));
        if team.is_player()
            && let Some(cap) = capability
            && cap.max_radar_range > 0.0
        {
            player_radars.insert(entity, cap.max_radar_range);
            active_radar_range_candidates = true;
        }
        let _ = (ship, transform);
    }

    let active_radar = if active_radar_range_candidates {
        resolve_active_radar_range(
            controlled.controlling,
            camera_target.following,
            &player_radars,
        )
    } else {
        None
    };
    let world_radius = cfg.scaled_world_radius(active_radar);

    for (_, ship, team, transform, _) in &ships {
        if !team.is_player() {
            continue;
        }
        let pos = transform
            .transform_point(ship.core_position.extend(0.0))
            .truncate();
        if let Some(offset) = world_to_minimap(pos - center, world_radius, radius_px) {
            render_specs.push(MarkerSpec {
                offset,
                color: ally_color,
            });
        }
    }

    let state_ref = &mut *state;

    if let Some(plan) = board.teams.get(&DEFAULT_TEAM_ID) {
        let enemy_color = Team::new(1).color(1.0);
        for (entity, contact) in &plan.intel.contacts {
            let color = colors.get(entity).copied().unwrap_or(enemy_color);
            current_contacts.insert(*entity, (contact.position, color));
            state_ref.lost_flashes.retain(|lost| lost.entity != *entity);

            if let Some(offset) =
                world_to_minimap(contact.position - center, world_radius, radius_px)
            {
                render_specs.push(MarkerSpec { offset, color });
            }
        }
    }

    for (&entity, &(world_position, color)) in &state_ref.prev_contacts {
        if !current_contacts.contains_key(&entity)
            && !state_ref
                .lost_flashes
                .iter()
                .any(|lost| lost.entity == entity)
        {
            state_ref.lost_flashes.push(LostContact {
                entity,
                world_position,
                color,
                elapsed: 0.0,
            });
        }
    }

    state_ref.prev_contacts.clear();
    state_ref
        .prev_contacts
        .extend(current_contacts.iter().map(|(&k, &v)| (k, v)));

    if !state_ref.lost_flashes.is_empty() {
        let dt = time.delta_secs();
        for lost in &mut state_ref.lost_flashes {
            lost.elapsed += dt;
        }
        state_ref
            .lost_flashes
            .retain(|lost| lost.elapsed < cfg.lost_flash_secs);

        for lost in &state_ref.lost_flashes {
            let phase = (lost.elapsed / cfg.lost_flash_secs) * cfg.lost_flash_count as f32;
            if phase.fract() >= 0.5 {
                continue;
            }
            if let Some(offset) =
                world_to_minimap(lost.world_position - center, world_radius, radius_px)
            {
                render_specs.push(MarkerSpec {
                    offset,
                    color: lost.color,
                });
            }
        }
    }

    render_specs.truncate(cfg.max_markers);

    for (marker, mut node, mut background, mut shadow, mut visibility) in &mut markers {
        let Some(spec) = render_specs.get(marker.slot) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
                *shadow = BoxShadow::default();
            }
            continue;
        };

        node.width = Val::Px(dot_size);
        node.height = Val::Px(dot_size);
        node.left = Val::Px(radius_px + spec.offset.x - dot_size * 0.5);
        node.top = Val::Px(radius_px + spec.offset.y - dot_size * 0.5);
        *background = BackgroundColor(spec.color);
        *shadow = BoxShadow::new(
            spec.color.with_alpha(GLOW_ALPHA),
            Val::Px(0.0),
            Val::Px(0.0),
            Val::Px(1.0),
            Val::Px(cfg.glow_blur_px),
        );
        *visibility = Visibility::Visible;
    }
}

pub fn resolve_active_radar_range(
    controlled: Option<Entity>,
    following: Option<Entity>,
    player_radars: &HashMap<Entity, f32>,
) -> Option<f32> {
    controlled
        .and_then(|entity| player_radars.get(&entity).copied())
        .or_else(|| following.and_then(|entity| player_radars.get(&entity).copied()))
        .or_else(|| player_radars.values().copied().reduce(f32::max))
}

pub fn world_to_minimap(offset: Vec2, world_radius: f32, radius_px: f32) -> Option<Vec2> {
    if !world_radius.is_finite() || world_radius <= 0.0 {
        return None;
    }

    let normalized = offset / world_radius;

    if normalized.length_squared() > 1.0 {
        return None;
    }

    Some(Vec2::new(normalized.x, -normalized.y) * radius_px)
}

fn spawn_marker(layer: &mut ChildSpawnerCommands, cfg: &MinimapConfig, slot: usize) {
    layer.spawn((
        MinimapMarker { slot },
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Px(cfg.dot_size_px),
            height: Val::Px(cfg.dot_size_px),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(Color::NONE),
        BoxShadow::default(),
        Pickable::IGNORE,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_active_radar_range_follows_priority_chain() {
        let ship_a = Entity::from_bits(1);
        let ship_b = Entity::from_bits(2);
        let ship_c = Entity::from_bits(3);

        let mut radars = HashMap::new();
        radars.insert(ship_a, 1200.0);
        radars.insert(ship_b, 1800.0);
        radars.insert(ship_c, 2500.0);

        assert_eq!(
            resolve_active_radar_range(Some(ship_a), Some(ship_b), &radars),
            Some(1200.0)
        );
        assert_eq!(
            resolve_active_radar_range(None, Some(ship_b), &radars),
            Some(1800.0)
        );
        assert_eq!(resolve_active_radar_range(None, None, &radars), Some(2500.0));
        assert_eq!(
            resolve_active_radar_range(None, None, &HashMap::new()),
            None
        );
    }

    #[test]
    fn world_to_minimap_is_north_up() {
        let offset = world_to_minimap(Vec2::new(0.0, 100.0), 1000.0, 50.0);
        assert_eq!(offset, Some(Vec2::new(0.0, -5.0)));

        let offset = world_to_minimap(Vec2::new(100.0, 0.0), 1000.0, 50.0);
        assert_eq!(offset, Some(Vec2::new(5.0, 0.0)));
    }

    #[test]
    fn world_to_minimap_clips_outside_radius() {
        let outside = world_to_minimap(Vec2::new(0.0, 1001.0), 1000.0, 50.0);
        assert_eq!(outside, None);

        let edge = world_to_minimap(Vec2::new(600.0, 800.0), 1000.0, 50.0)
            .expect("edge point inside radius");
        assert!((edge - Vec2::new(30.0, -40.0)).length() < 1e-4);
    }
}
