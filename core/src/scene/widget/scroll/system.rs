use std::mem;

use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::hover::HoverMap,
    prelude::*,
};

use super::{Scroll, ScrollTarget, ScrollbarThumb, ScrollbarTrack, ScrollViewport};

const MIN_THUMB_PERCENT: f32 = 10.0;
const FIT_TOLERANCE: f32 = 1.0;

pub fn send_scroll_events(
    mut mouse_wheel_reader: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mouse_wheel in mouse_wheel_reader.read() {
        let mut delta = -Vec2::new(mouse_wheel.x, mouse_wheel.y);

        if mouse_wheel.unit == MouseScrollUnit::Line {
            delta *= 21.0;
        }

        if keyboard_input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
            mem::swap(&mut delta.x, &mut delta.y);
        }

        for pointer_map in hover_map.values() {
            for entity in pointer_map.keys().copied() {
                commands.trigger(Scroll { entity, delta });
            }
        }
    }
}

pub fn on_scroll_handler(
    mut scroll: On<Scroll>,
    mut query: Query<(&mut ScrollPosition, &Node, &ComputedNode)>,
) {
    let Ok((mut scroll_position, node, computed)) = query.get_mut(scroll.entity) else {
        return;
    };

    let max_offset = (computed.content_size() - computed.size()) * computed.inverse_scale_factor();

    let delta = &mut scroll.delta;
    if node.overflow.x == OverflowAxis::Scroll && delta.x != 0.0 {
        let previous = scroll_position.x;
        scroll_position.x = (previous + delta.x).clamp(0.0, max_offset.x.max(0.0));
        delta.x -= scroll_position.x - previous;
    }

    if node.overflow.y == OverflowAxis::Scroll && delta.y != 0.0 {
        let previous = scroll_position.y;
        scroll_position.y = (previous + delta.y).clamp(0.0, max_offset.y.max(0.0));
        delta.y -= scroll_position.y - previous;
    }

    if delta.x.abs() < f32::EPSILON && delta.y.abs() < f32::EPSILON {
        scroll.propagate(false);
    }
}

pub fn sync_scrollbars(
    viewports: Query<(Entity, &ComputedNode, &ScrollPosition), With<ScrollViewport>>,
    mut tracks: Query<(&ScrollTarget, &mut Node), With<ScrollbarTrack>>,
    mut thumbs: Query<(&ScrollTarget, &mut Node), With<ScrollbarThumb>>,
) {
    for (viewport, computed, scroll_position) in &viewports {
        let content_height = computed.content_size().y;
        let view_height = computed.size().y;
        let hidden = content_height <= view_height + FIT_TOLERANCE;
        let max_offset = (content_height - view_height).max(0.0) * computed.inverse_scale_factor();
        let thumb_height = if hidden || content_height <= 0.0 {
            100.0
        } else {
            (view_height / content_height * 100.0).clamp(MIN_THUMB_PERCENT, 100.0)
        };
        let ratio = if max_offset > 0.0 {
            (scroll_position.y / max_offset).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let thumb_top = (100.0 - thumb_height) * ratio;

        for (target, mut track) in &mut tracks {
            if target.0 != viewport {
                continue;
            }
            let display = if hidden { Display::None } else { Display::Flex };
            if track.display != display {
                track.display = display;
            }
        }

        for (target, mut thumb) in &mut thumbs {
            if target.0 != viewport {
                continue;
            }
            let height = Val::Percent(thumb_height);
            if thumb.height != height {
                thumb.height = height;
            }
            let top = Val::Percent(thumb_top);
            if thumb.top != top {
                thumb.top = top;
            }
        }
    }
}
