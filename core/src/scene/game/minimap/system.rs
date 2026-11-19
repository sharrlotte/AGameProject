use std::collections::HashMap;
use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use crate::content::ship::Ship;
use crate::content::ship::ai::{AiConfigValue, Capability, TacticalBoard};
use crate::content::team::{DEFAULT_TEAM_ID, Team};
use crate::resource::{CameraTarget, ControlledShip, SceneState};

use super::component::{
    AnchorCandidate, MarkerSpec, MinimapAnchor, MinimapArrow, MinimapMarker, MinimapMarkerLayer,
    MinimapRing, MinimapRoot,
};
use super::config::{MinimapConfig, MinimapConfigValue};

const ALERT_COLOR: Color = Color::srgb(1.0, 0.55, 0.15);
const MINIMAP_BACKGROUND: Color = Color::srgba(0.02, 0.03, 0.05, 0.65);
const MINIMAP_BORDER: Color = Color::srgba(0.3, 0.5, 0.7, 0.8);
const RING_COLOR: Color = Color::srgba(0.35, 0.6, 0.85, 0.75);

pub fn setup_minimap(mut commands: Commands, config: Res<MinimapConfigValue>) {
    let cfg = config.0.clone();

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
                MinimapRing,
                Visibility::Hidden,
                Node {
                    position_type: PositionType::Absolute,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::MAX,
                    ..default()
                },
                BorderColor::all(RING_COLOR),
                Pickable::IGNORE,
            ));

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

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|center| {
                center.spawn((
                    MinimapArrow,
                    Visibility::Hidden,
                    Text::new("\u{25b2}"),
                    TextFont {
                        font_size: FontSize::Px(cfg.arrow_size_px),
                        ..default()
                    },
                    TextColor(Team::default().color(1.0)),
                    UiTransform::IDENTITY,
                    Pickable::IGNORE,
                ));
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
    config: Res<MinimapConfigValue>,
    board: Res<TacticalBoard>,
    ai_config: Res<AiConfigValue>,
    controlled: Res<ControlledShip>,
    camera_target: Res<CameraTarget>,
    camera: Single<&GlobalTransform, With<Camera2d>>,
    ships: Query<(Entity, &Ship, &Team, &GlobalTransform, Option<&Capability>)>,
    mut ui: ParamSet<(
        Query<&mut Node, With<MinimapRoot>>,
        Query<(&mut Node, &mut Visibility), With<MinimapRing>>,
        Query<(&mut UiTransform, &mut Visibility, &mut TextFont), With<MinimapArrow>>,
        Query<(&MinimapMarker, &mut Node, &mut BackgroundColor, &mut Visibility)>,
    )>,
) {
    let cfg = &config.0;
    let radius_px = cfg.radius_px();
    let dot_size = cfg.dot_size_px;

    if let Ok(mut root) = ui.p0().single_mut() {
        root.width = Val::Px(cfg.size_px);
        root.height = Val::Px(cfg.size_px);
        root.top = Val::Px(cfg.margin_px);
        root.right = Val::Px(cfg.margin_px);
    }

    let camera_position = camera.translation().truncate();
    let mut candidates = Vec::new();
    let mut allied = Vec::new();
    let mut colors: HashMap<Entity, Color> = HashMap::new();

    for (entity, ship, team, transform, capability) in &ships {
        let position = transform
            .transform_point(ship.core_position.extend(0.0))
            .truncate();
        colors.insert(entity, team.color(1.0));

        if !team.is_player() {
            continue;
        }

        let up = transform.up().truncate();
        let heading = if up.length_squared() > f32::EPSILON {
            up.y.atan2(up.x)
        } else {
            FRAC_PI_2
        };

        candidates.push(AnchorCandidate {
            entity,
            id: ship.id,
            position,
            heading,
            radar_range: capability.map_or(0.0, |cap| cap.max_radar_range),
        });
        allied.push((entity, position));
    }

    let anchor = resolve_minimap_anchor(
        controlled.controlling,
        camera_target.following,
        &mut candidates,
        camera_position,
    );
    let anchor_position = anchor.position();
    let anchor_entity = anchor.ship().map(|candidate| candidate.entity);

    if let Ok((mut node, mut visibility)) = ui.p1().single_mut() {
        let diameter = anchor
            .ship()
            .map_or(0.0, |candidate| cfg.radar_ring_diameter(candidate.radar_range));

        if diameter > 0.0 {
            node.width = Val::Px(diameter);
            node.height = Val::Px(diameter);
            node.left = Val::Px(radius_px - diameter * 0.5);
            node.top = Val::Px(radius_px - diameter * 0.5);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    if let Ok((mut transform, mut visibility, mut font)) = ui.p2().single_mut() {
        if let Some(candidate) = anchor.ship() {
            transform.rotation = Rot2::radians(FRAC_PI_2 - candidate.heading);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }

        font.font_size = FontSize::Px(cfg.arrow_size_px);
    }

    let mut specs = Vec::new();
    if let Some(plan) = board.teams.get(&DEFAULT_TEAM_ID) {
        let enemy_color = Team::new(1).color(1.0);
        let max_memory = ai_config.0.curves.memory_secs.level_10.max(0.001);
        let max_alert = ai_config.0.curves.alert_ttl_secs.level_10.max(0.001);

        for (entity, contact) in &plan.intel.contacts {
            if let Some(offset) = world_to_minimap(
                contact.position - anchor_position,
                cfg.world_radius,
                radius_px,
            ) {
                let color = colors.get(entity).copied().unwrap_or(enemy_color);
                specs.push(MarkerSpec { offset, color });
            }
        }

        for (entity, position) in &allied {
            if Some(*entity) == anchor_entity {
                continue;
            }
            if let Some(offset) = world_to_minimap(
                *position - anchor_position,
                cfg.world_radius,
                radius_px,
            ) {
                specs.push(MarkerSpec {
                    offset,
                    color: Team::default().color(1.0),
                });
            }
        }

        for alert in &plan.intel.alerts {
            if let Some(offset) = world_to_minimap(
                alert.position - anchor_position,
                cfg.world_radius,
                radius_px,
            ) {
                let alpha = (alert.ttl / max_alert).clamp(0.0, 1.0);
                specs.push(MarkerSpec {
                    offset,
                    color: ALERT_COLOR.with_alpha(alpha),
                });
            }
        }

        for seen in plan.intel.last_known.values() {
            if let Some(offset) = world_to_minimap(
                seen.position - anchor_position,
                cfg.world_radius,
                radius_px,
            ) {
                let alpha = (seen.ttl / max_memory).clamp(0.0, 1.0);
                specs.push(MarkerSpec {
                    offset,
                    color: enemy_color.with_alpha(alpha),
                });
            }
        }
    }

    specs.truncate(cfg.max_markers);

    let mut markers = ui.p3();
    for (marker, mut node, mut background, mut visibility) in markers.iter_mut() {
        let Some(spec) = specs.get(marker.slot) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        node.width = Val::Px(dot_size);
        node.height = Val::Px(dot_size);
        node.left = Val::Px(radius_px + spec.offset.x - dot_size * 0.5);
        node.top = Val::Px(radius_px + spec.offset.y - dot_size * 0.5);
        *background = BackgroundColor(spec.color);
        *visibility = Visibility::Visible;
    }
}

pub fn resolve_minimap_anchor(
    controlled: Option<Entity>,
    following: Option<Entity>,
    candidates: &mut [AnchorCandidate],
    camera_position: Vec2,
) -> MinimapAnchor {
    candidates.sort_by_key(|candidate| candidate.id);

    let find = |entity: Entity| {
        candidates
            .iter()
            .find(|candidate| candidate.entity == entity)
            .copied()
    };

    if let Some(anchor) = controlled.and_then(find) {
        return MinimapAnchor::Ship(anchor);
    }

    if let Some(anchor) = following.and_then(find) {
        return MinimapAnchor::Ship(anchor);
    }

    candidates.first().copied().map_or(
        MinimapAnchor::Camera(camera_position),
        MinimapAnchor::Ship,
    )
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
        Pickable::IGNORE,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    use uuid::Uuid;

    fn candidate(id: u128, entity: Entity) -> AnchorCandidate {
        AnchorCandidate {
            entity,
            id: Uuid::from_u128(id),
            position: Vec2::new(id as f32, 0.0),
            heading: FRAC_PI_2,
            radar_range: 2500.0,
        }
    }

    fn entity() -> Entity {
        World::new().spawn_empty().id()
    }

    #[test]
    fn controlled_ship_wins_anchor_chain() {
        let first = entity();
        let second = entity();
        let third = entity();
        let mut candidates = [
            candidate(1, first),
            candidate(2, second),
            candidate(3, third),
        ];

        let anchor =
            resolve_minimap_anchor(Some(third), Some(second), &mut candidates, Vec2::ZERO);

        assert_eq!(anchor.ship().map(|anchor| anchor.entity), Some(third));
    }

    #[test]
    fn followed_ship_is_used_when_not_controlling() {
        let first = entity();
        let second = entity();
        let mut candidates = [candidate(2, second), candidate(1, first)];

        let anchor = resolve_minimap_anchor(None, Some(second), &mut candidates, Vec2::ZERO);

        assert_eq!(anchor.ship().map(|anchor| anchor.entity), Some(second));
    }

    #[test]
    fn first_player_ship_by_id_is_fallback() {
        let lower = entity();
        let higher = entity();
        let mut candidates = [candidate(9, higher), candidate(4, lower)];

        let anchor = resolve_minimap_anchor(None, None, &mut candidates, Vec2::ZERO);

        assert_eq!(anchor.ship().map(|anchor| anchor.entity), Some(lower));
    }

    #[test]
    fn camera_position_is_last_resort() {
        let camera = Vec2::new(120.0, -40.0);
        let mut candidates = Vec::new();
        let anchor = resolve_minimap_anchor(None, None, &mut candidates, camera);
        assert_eq!(anchor, MinimapAnchor::Camera(camera));
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

        let edge = world_to_minimap(Vec2::new(600.0, 800.0), 1000.0, 50.0);
        assert!(edge.is_some());
    }
}
