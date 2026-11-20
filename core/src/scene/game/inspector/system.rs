use std::collections::BTreeMap;
use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;


use crate::content::{
    block::{BlockRegistry, Health, Rot90, feature::component::Feature},
    body::BlockGrid,
    ship::{
        InspectedShip, ShipChanged, ShipStats,
        ai::{Brain, Capability, Doctrine, Readiness, Skill, Steering},
    },
};

#[cfg(feature = "dev")]
use crate::content::block::{
    BlockOwner,
    feature::ammo::{Ammo, AmmoState},
};

use super::*;

const DIAGRAM_MAX_PX: f32 = 140.0;
const CELL_MIN_PX: f32 = 6.0;
const CELL_MAX_PX: f32 = 24.0;
const BAR_WIDTH_PX: f32 = 110.0;
const PANEL_WIDTH_PX: f32 = 460.0;

pub fn spawn_inspector_panel(parent: &mut ChildSpawnerCommands) {
    parent
        .spawn((
            ShipInspectorRoot,
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                bottom: Val::Px(12.0),
                width: Val::Px(PANEL_WIDTH_PX),
                display: Display::None,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.08, 0.11, 0.92)),
            BorderColor::all(Color::srgba(0.35, 0.55, 0.75, 0.85)),
            Pickable::IGNORE,
        ))
        .with_children(|panel| {
            panel.spawn((
                InspectorText::Header,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(13.0),
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.93, 1.0)),
                Pickable::IGNORE,
            ));

            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|body| {
                    body.spawn((
                        InspectorBlockGrid,
                        Node {
                            position_type: PositionType::Relative,
                            width: Val::Px(DIAGRAM_MAX_PX),
                            height: Val::Px(DIAGRAM_MAX_PX),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.8)),
                        Pickable::IGNORE,
                    ));

                    body.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            flex_grow: 1.0,
                            min_width: Val::Px(0.0),
                            row_gap: Val::Px(4.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|stats| {
                        spawn_stat_bar(
                            stats,
                            "HP",
                            Color::srgb(0.25, 0.85, 0.45),
                            InspectorBar::Hp,
                            InspectorText::Hp,
                        );
                        spawn_stat_bar(
                            stats,
                            "PWR",
                            Color::srgb(0.25, 0.75, 0.95),
                            InspectorBar::Power,
                            InspectorText::Power,
                        );
                        spawn_stat_bar(
                            stats,
                            "BAT",
                            Color::srgb(0.95, 0.78, 0.25),
                            InspectorBar::Stored,
                            InspectorText::Stored,
                        );
                        spawn_stat_bar(
                            stats,
                            "AMO",
                            Color::srgb(0.95, 0.48, 0.22),
                            InspectorBar::Ammo,
                            InspectorText::Ammo,
                        );
                        stats.spawn((
                            InspectorText::Stats,
                            Text::new(""),
                            TextFont {
                                font_size: FontSize::Px(11.0),
                                ..default()
                            },
                            TextColor(Color::srgb(0.78, 0.84, 0.9)),
                            Node {
                                min_height: Val::Px(44.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                    });
                });

            #[cfg(feature = "dev")]
            panel.spawn((
                InspectorText::Dev,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(10.0),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.82, 0.45)),
                Node {
                    width: Val::Percent(100.0),
                    min_height: Val::Px(36.0),
                    ..default()
                },
                Pickable::IGNORE,
            ));
        });
}

fn spawn_stat_bar<B: Component, T: Component>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    fill_color: Color,
    bar_marker: B,
    text_marker: T,
) {
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.72, 0.8)),
                Node {
                    width: Val::Px(30.0),
                    ..default()
                },
                Pickable::IGNORE,
            ));
            row.spawn((
                Node {
                    width: Val::Px(BAR_WIDTH_PX),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.18, 0.22, 1.0)),
                Pickable::IGNORE,
            ))
            .with_children(|track| {
                track.spawn((
                    bar_marker,
                    Node {
                        width: Val::Px(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(fill_color),
                    Pickable::IGNORE,
                ));
            });
            row.spawn((
                text_marker,
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Pickable::IGNORE,
            ));
        });
}

pub fn rebuild_inspector_diagram(
    mut commands: Commands,
    inspected: Res<InspectedShip>,
    mut build_state: ResMut<InspectorBuildState>,
    mut ship_changed: MessageReader<ShipChanged>,
    grid_container: Single<Entity, With<InspectorBlockGrid>>,
    ships: Query<&BlockGrid>,
    registry: Res<BlockRegistry>,
) {
    let inspected_changed = inspected.0 != build_state.ship;
    let structure_changed = inspected
        .0
        .is_some_and(|ship| ship_changed.read().any(|ev| ev.ship == ship));

    if !inspected_changed && !structure_changed {
        return;
    }

    build_state.ship = inspected.0;
    let container = grid_container.entity();
    commands.entity(container).despawn_children();

    let Some(ship_entity) = inspected.0 else {
        return;
    };
    let Ok(grid) = ships.get(ship_entity) else {
        return;
    };

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for block in grid.blocks() {
        let Some(def) = registry.get_symbol(&block.placement.block_type) else {
            continue;
        };
        let size = def.get_block_size(block.placement.rotation);
        let w = size.x.round() as i32;
        let h = size.y.round() as i32;
        min_x = min_x.min(block.placement.position.x);
        min_y = min_y.min(block.placement.position.y);
        max_x = max_x.max(block.placement.position.x + w);
        max_y = max_y.max(block.placement.position.y + h);
    }

    if min_x >= max_x || min_y >= max_y {
        return;
    }

    let span_x = (max_x - min_x) as f32;
    let span_y = (max_y - min_y) as f32;
    let cell_px = (DIAGRAM_MAX_PX / span_x.max(span_y)).clamp(CELL_MIN_PX, CELL_MAX_PX);
    let offset_x = (DIAGRAM_MAX_PX - span_x * cell_px) * 0.5;
    let offset_y = (DIAGRAM_MAX_PX - span_y * cell_px) * 0.5;

    commands.entity(container).with_children(|parent| {
        for block in grid.blocks() {
            let placement = &block.placement;
            let Some(def) = registry.get_symbol(&placement.block_type) else {
                continue;
            };
            let size = def.get_block_size(placement.rotation);
            let w_px = size.x * cell_px;
            let h_px = size.y * cell_px;
            let left_px = offset_x + (placement.position.x - min_x) as f32 * cell_px;
            let top_px = offset_y + (max_y as f32 - placement.position.y as f32 - size.y) * cell_px;

            let xform = def.shape.xform();
            let total_turns = (placement.rotation.turns() + xform.rotate.turns()) % 4;
            let angle = -(total_turns as f32 * FRAC_PI_2);

            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(left_px),
                        top: Val::Px(top_px),
                        width: Val::Px(w_px),
                        height: Val::Px(h_px),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|slot| {
                    for feature in &def.features {
                        let image = match feature {
                            Feature::Texture(visual) => visual.image.clone(),
                            Feature::Turret(visual) => visual.image.clone(),
                            _ => continue,
                        };
                        let base_size = def.get_block_size(Rot90::R0);
                        slot.spawn((
                            InspectorBlockNode(block.entity),
                            ImageNode {
                                image,
                                flip_x: xform.flip_x,
                                flip_y: xform.flip_y,
                                ..default()
                            },
                            Node {
                                position_type: PositionType::Absolute,
                                width: Val::Px(base_size.x * cell_px),
                                height: Val::Px(base_size.y * cell_px),
                                ..default()
                            },
                            UiTransform::from_rotation(Rot2::radians(angle)),
                            Pickable::IGNORE,
                        ));
                    }
                });
        }
    });
}

pub fn refresh_inspector_ui(
    inspected: Res<InspectedShip>,
    mut root_nodes: ParamSet<(
        Single<(&mut Visibility, &mut Node), With<ShipInspectorRoot>>,
        Query<(&InspectorBar, &mut Node)>,
    )>,
    ships: Query<(
        &ShipStats,
        &Readiness,
        &Brain,
        &Doctrine,
        &Capability,
        Option<&Skill>,
        Option<&Steering>,
    )>,
    healths: Query<&Health>,
    mut block_nodes: Query<(&InspectorBlockNode, &mut ImageNode)>,
    mut texts: Query<(&InspectorText, &mut Text)>,
    #[cfg(feature = "dev")] ammo_blocks: Query<(&BlockOwner, Option<&Ammo>, Option<&AmmoState>)>,
) {
    let Some(ship_entity) = inspected.0 else {
        let (mut root_vis, mut root_node) = root_nodes.p0().into_inner();
        hide_root(&mut root_vis, &mut root_node);
        return;
    };

    let Ok((stats, readiness, brain, doctrine, cap, _skill, _steering)) = ships.get(ship_entity)
    else {
        let (mut root_vis, mut root_node) = root_nodes.p0().into_inner();
        hide_root(&mut root_vis, &mut root_node);
        return;
    };

    {
        let (mut root_vis, mut root_node) = root_nodes.p0().into_inner();
        *root_vis = Visibility::Visible;
        root_node.display = Display::Flex;
    }

    for (node, mut image) in &mut block_nodes {
        let Ok(health) = healths.get(node.0) else {
            continue;
        };
        let ratio = if health.max_health > 0.0 {
            (health.health / health.max_health).clamp(0.0, 1.0)
        } else {
            1.0
        };
        image.color = if ratio > 0.75 {
            Color::WHITE
        } else if ratio > 0.4 {
            Color::srgb(1.0, 0.85, 0.3)
        } else {
            Color::srgb(1.0, 0.35, 0.35)
        };
    }

    let hp_ratio = if stats.max_health > 0.0 {
        (stats.health / stats.max_health).clamp(0.0, 1.0)
    } else {
        readiness.hp_ratio.clamp(0.0, 1.0)
    };
    let power_ratio = stats.power_supply.clamp(0.0, 1.0);
    let stored_ratio = if stats.power_capacity > 0.0 {
        (stats.power_stored / stats.power_capacity).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let ammo_ratio = readiness.ammo_ratio.clamp(0.0, 1.0);

    for (bar, mut node) in &mut root_nodes.p1() {
        let ratio = match bar {
            InspectorBar::Hp => hp_ratio,
            InspectorBar::Power => power_ratio,
            InspectorBar::Stored => stored_ratio,
            InspectorBar::Ammo => ammo_ratio,
        };
        node.width = Val::Px(BAR_WIDTH_PX * ratio);
    }

    let cargo_breakdown = if stats.cargo_items.is_empty() {
        String::new()
    } else {
        let sorted: BTreeMap<&String, &u32> = stats.cargo_items.iter().collect();
        let joined = sorted
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join(" ");
        format!(" ({joined})")
    };

    #[cfg(feature = "dev")]
    let dev_summary = {
        let mut ammo_count = 0usize;
        let mut ammo_total = 0u32;
        for (owner, ammo, ammo_state) in &ammo_blocks {
            if owner.body != ship_entity {
                continue;
            }
            if ammo.is_some() || ammo_state.is_some() {
                ammo_count += 1;
            }
            if let Some(state) = ammo_state {
                ammo_total += state.current();
            }
        }
        let skill_label = _skill.map_or(0, |s| s.level());
        format!(
            "DEV | Skill: {skill_label} | Ammo: {ammo_total} ({ammo_count} blocks)\nSteering: {:?}",
            _steering
        )
    };

    for (kind, mut text) in &mut texts {
        text.0 = match kind {
            InspectorText::Header => format!(
                "{}  |  {}  |  {}",
                cap.role.label(),
                doctrine.label(),
                brain.current.label(),
            ),
            InspectorText::Hp => format!(
                "{:.0}/{:.0} ({:.0}%)",
                stats.health,
                stats.max_health,
                hp_ratio * 100.0
            ),
            InspectorText::Power => format!(
                "+{:.0}/-{:.0} ({:.0}%)",
                stats.power_production,
                stats.power_demand,
                power_ratio * 100.0
            ),
            InspectorText::Stored => format!("{:.0}/{:.0}", stats.power_stored, stats.power_capacity),
            InspectorText::Ammo => format!("{:.0}%", ammo_ratio * 100.0),
            InspectorText::Stats => format!(
                "CARGO {}/{}{}\nMASS {:.0}  THRUST {:.0}  TURN {:.1}\nRANGE {:.0}-{:.0}  RADAR {:.0}",
                stats.cargo_used(),
                stats.cargo_capacity,
                cargo_breakdown,
                stats.mass,
                stats.thrust_power,
                stats.rotate_speed,
                stats.min_range,
                stats.max_range,
                cap.max_radar_range,
            ),
            #[cfg(feature = "dev")]
            InspectorText::Dev => dev_summary.clone(),
        };
    }
}

fn hide_root(visibility: &mut Visibility, node: &mut Node) {
    if *visibility != Visibility::Hidden {
        *visibility = Visibility::Hidden;
    }
    if node.display != Display::None {
        node.display = Display::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::ship::Ship;

    #[test]
    fn diagram_does_not_rebuild_without_selection_or_ship_changed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<InspectedShip>()
            .init_resource::<InspectorBuildState>()
            .init_resource::<BlockRegistry>()
            .add_message::<ShipChanged>()
            .add_systems(Update, rebuild_inspector_diagram);

        let container = app.world_mut().spawn((InspectorBlockGrid, Node::default())).id();
        let ship = app
            .world_mut()
            .spawn((Ship::default(), BlockGrid::new()))
            .id();
        app.world_mut().resource_mut::<InspectedShip>().0 = Some(ship);
        app.update();

        let marker = app.world_mut().spawn_empty().id();
        app.world_mut().entity_mut(container).add_child(marker);

        app.update();
        assert!(app.world().get_entity(marker).is_ok());

        app.world_mut()
            .resource_mut::<Messages<ShipChanged>>()
            .write(ShipChanged { ship });
        app.update();
        assert!(app.world().get_entity(marker).is_err());
    }

    #[test]
    fn diagram_clears_when_inspected_ship_is_none() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<InspectedShip>()
            .init_resource::<InspectorBuildState>()
            .init_resource::<BlockRegistry>()
            .add_message::<ShipChanged>()
            .add_systems(Update, rebuild_inspector_diagram);

        let container = app.world_mut().spawn((InspectorBlockGrid, Node::default())).id();
        let ship = app
            .world_mut()
            .spawn((Ship::default(), BlockGrid::new()))
            .id();
        app.world_mut().resource_mut::<InspectedShip>().0 = Some(ship);
        app.update();

        let marker = app.world_mut().spawn_empty().id();
        app.world_mut().entity_mut(container).add_child(marker);

        app.world_mut().resource_mut::<InspectedShip>().0 = None;
        app.update();
        assert!(app.world().get_entity(marker).is_err());
    }
}
