use bevy::prelude::*;

use super::{ScrollTarget, ScrollbarThumb, ScrollbarTrack, ScrollViewport};

const SCROLLBAR_WIDTH: f32 = 6.0;
const TRACK_COLOR: Color = Color::srgb(0.06, 0.07, 0.08);
const THUMB_COLOR: Color = Color::srgb(0.35, 0.40, 0.48);

pub fn spawn_scroll_area<M: Component>(
    parent: &mut ChildSpawnerCommands,
    display: Display,
    row_gap: f32,
    marker: M,
    build_content: impl FnOnce(&mut ChildSpawnerCommands),
) -> Entity {
    let mut commands = parent.commands();

    let wrapper = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_height: Val::Px(0.0),
                display,
                ..default()
            },
            marker,
        ))
        .id();

    let viewport = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                flex_basis: Val::Px(0.0),
                min_width: Val::Px(0.0),
                min_height: Val::Px(0.0),
                row_gap: Val::Px(row_gap),
                overflow: Overflow::scroll_y(),
                ..default()
            },
            ScrollViewport,
            ScrollPosition::default(),
        ))
        .id();

    let track = commands
        .spawn((
            Node {
                width: Val::Px(SCROLLBAR_WIDTH),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(TRACK_COLOR),
            ScrollbarTrack,
            ScrollTarget(viewport),
            Pickable::IGNORE,
        ))
        .id();

    let thumb = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                top: Val::Percent(0.0),
                ..default()
            },
            BackgroundColor(THUMB_COLOR),
            ScrollbarThumb,
            ScrollTarget(viewport),
            Pickable::IGNORE,
        ))
        .id();

    commands.entity(viewport).with_children(build_content);
    commands.entity(track).add_child(thumb);
    commands.entity(wrapper).add_child(viewport).add_child(track);

    viewport
}
