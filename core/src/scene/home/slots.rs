use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;
use uuid::Uuid;

use crate::resource::{AppNotice, SlotPickerState};
use crate::save::world::sqlite::{delete_slot_files, list_slots, saves_dir, slot_path};
use crate::save::world::{CreateSlotEvent, LoadSlotEvent, SlotSummary};
use crate::scene::widget::{ButtonVisuals, spawn_button};

#[derive(Resource, Default, Debug)]
pub struct SlotPickerUi {
    pub slots: Vec<SlotSummary>,
    pub selected: Option<Uuid>,
    pub confirm_delete: bool,
    pub name_buffer: String,
}

#[derive(Component)]
pub struct SlotPickerRoot;

#[derive(Component)]
pub struct SlotNameText;

#[derive(Component)]
struct SlotRow;

#[derive(Component)]
struct SlotSelectButton(Uuid);

#[derive(Component)]
struct SlotLoadButton;

#[derive(Component)]
struct SlotDeleteButton;

#[derive(Component)]
struct SlotConfirmDeleteButton;

#[derive(Component)]
struct SlotCancelDeleteButton;

#[derive(Component)]
struct SlotNewButton;

#[derive(Component)]
struct SlotCloseButton;

pub fn refresh_slot_list(mut ui: ResMut<SlotPickerUi>) {
    ui.slots = list_slots(&saves_dir());
    if ui.selected.is_none_or(|id| !ui.slots.iter().any(|slot| slot.slot_id == id)) {
        ui.selected = ui.slots.first().map(|slot| slot.slot_id);
    }
    ui.confirm_delete = false;
}

pub fn setup_picker(mut commands: Commands, ui: Res<SlotPickerUi>) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(16.0),
                position_type: PositionType::Absolute,
                overflow: Overflow::hidden(),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.97)),
            GlobalZIndex(100),
            SlotPickerRoot,
            DespawnOnExit(SlotPickerState::Open),
        ))
        .with_children(|parent| spawn_picker_content(parent, &ui));
}

pub fn rebuild_picker(
    mut commands: Commands,
    roots: Query<Entity, With<SlotPickerRoot>>,
    ui: &SlotPickerUi,
) {
    for root in &roots {
        commands.entity(root).despawn_children();
        commands
            .entity(root)
            .with_children(|parent| spawn_picker_content(parent, ui));
    }
}

fn spawn_picker_content(parent: &mut ChildSpawnerCommands, ui: &SlotPickerUi) {
    parent.spawn((
        Text::new("Save Slots"),
        TextFont {
            font_size: FontSize::Px(40.0),
            ..default()
        },
        TextColor(Color::WHITE),
    ));

    parent
        .spawn(Node {
            width: Val::Px(720.0),
            max_height: Val::Percent(55.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            overflow: Overflow::scroll_y(),
            ..default()
        })
        .with_children(|list| {
            if ui.slots.is_empty() {
                list.spawn((
                    Text::new("No worlds yet — create one below."),
                    TextFont {
                        font_size: FontSize::Px(18.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.75, 0.8)),
                ));
            }
            for slot in &ui.slots {
                spawn_slot_row(list, slot, ui);
            }
        });

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|row| {
            row.spawn((
                Text::new("World name:"),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            let name = if ui.name_buffer.is_empty() {
                "Frontier".to_string()
            } else {
                ui.name_buffer.clone()
            };
            row.spawn((
                Text::new(name),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.9, 1.0)),
                SlotNameText,
            ));
            spawn_button(
                row,
                small_node(120.0, 40.0),
                "New World",
                16.0,
                Color::WHITE,
                SlotNewButton,
                on_new_world,
                ButtonVisuals::green(),
            );
        });

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|row| {
            spawn_button(
                row,
                small_node(120.0, 44.0),
                "Close",
                18.0,
                Color::WHITE,
                SlotCloseButton,
                on_close_picker,
                ButtonVisuals::red(),
            );
        });
}

fn spawn_slot_row(parent: &mut ChildSpawnerCommands, slot: &SlotSummary, ui: &SlotPickerUi) {
    let selected = ui.selected == Some(slot.slot_id);
    let border = if selected {
        Color::srgb(0.3, 0.8, 1.0)
    } else {
        Color::srgb(0.25, 0.28, 0.32)
    };

    parent
        .spawn((
            Node {
                width: Val::Percent(100.0),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BorderColor::all(border),
            BackgroundColor(Color::srgb(0.10, 0.12, 0.15)),
            SlotRow,
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(format!(
                    "{}\nPlay time: {}   Saved: {}",
                    slot.name,
                    format_play_time(slot.play_time_secs),
                    format_saved(slot.updated_at_ms)
                )),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            spawn_button(
                row,
                small_node(110.0, 40.0),
                "Select",
                16.0,
                Color::WHITE,
                SlotSelectButton(slot.slot_id),
                on_select_slot,
                ButtonVisuals::blue(),
            );
            spawn_button(
                row,
                small_node(90.0, 40.0),
                "Load",
                16.0,
                Color::WHITE,
                SlotLoadButton,
                on_load_slot,
                ButtonVisuals::green(),
            );

            if ui.confirm_delete && selected {
                spawn_button(
                    row,
                    small_node(90.0, 40.0),
                    "Confirm",
                    16.0,
                    Color::WHITE,
                    SlotConfirmDeleteButton,
                    on_confirm_delete,
                    ButtonVisuals::red(),
                );
                spawn_button(
                    row,
                    small_node(80.0, 40.0),
                    "Cancel",
                    16.0,
                    Color::WHITE,
                    SlotCancelDeleteButton,
                    on_cancel_delete,
                    ButtonVisuals::dark(),
                );
            } else {
                spawn_button(
                    row,
                    small_node(90.0, 40.0),
                    "Delete",
                    16.0,
                    Color::WHITE,
                    SlotDeleteButton,
                    on_delete_prompt,
                    ButtonVisuals::red(),
                );
            }
        });
}

pub fn handle_name_input(
    mut keys: MessageReader<KeyboardInput>,
    mut ui: ResMut<SlotPickerUi>,
    mut texts: Query<&mut Text, With<SlotNameText>>,
) {
    let mut changed = false;
    for event in keys.read() {
        if event.state != ButtonState::Pressed {
            continue;
        }
        match &event.logical_key {
            Key::Character(ch) => {
                if ui.name_buffer.chars().count() < 32 {
                    ui.name_buffer.push_str(ch.as_str());
                    changed = true;
                }
            }
            Key::Backspace => {
                ui.name_buffer.pop();
                changed = true;
            }
            _ => {}
        }
    }
    if changed {
        let name = if ui.name_buffer.is_empty() {
            "Frontier".to_string()
        } else {
            ui.name_buffer.clone()
        };
        for mut text in &mut texts {
            text.0 = name.clone();
        }
    }
}

fn on_select_slot(
    click: On<PointerClick>,
    buttons: Query<&SlotSelectButton>,
    mut ui: ResMut<SlotPickerUi>,
    commands: Commands,
    roots: Query<Entity, With<SlotPickerRoot>>,
) {
    let Ok(button) = buttons.get(click.event().entity) else {
        return;
    };
    ui.selected = Some(button.0);
    ui.confirm_delete = false;
    rebuild_picker(commands, roots, &ui);
}

fn on_load_slot(
    click: On<PointerClick>,
    buttons: Query<&SlotLoadButton>,
    ui: Res<SlotPickerUi>,
    mut loads: MessageWriter<LoadSlotEvent>,
    mut picker: ResMut<NextState<SlotPickerState>>,
) {
    if !buttons.contains(click.event().entity) {
        return;
    }
    let Some(slot) = ui
        .slots
        .iter()
        .find(|slot| Some(slot.slot_id) == ui.selected)
        .cloned()
    else {
        warn!("Load clicked with no selected slot");
        return;
    };
    loads.write(LoadSlotEvent { slot });
    picker.set(SlotPickerState::Closed);
}

fn on_new_world(
    click: On<PointerClick>,
    buttons: Query<&SlotNewButton>,
    ui: Res<SlotPickerUi>,
    mut creates: MessageWriter<CreateSlotEvent>,
    mut picker: ResMut<NextState<SlotPickerState>>,
) {
    if !buttons.contains(click.event().entity) {
        return;
    }
    let name = if ui.name_buffer.trim().is_empty() {
        "Frontier".to_string()
    } else {
        ui.name_buffer.trim().to_string()
    };
    creates.write(CreateSlotEvent { name });
    picker.set(SlotPickerState::Closed);
}

fn on_delete_prompt(
    click: On<PointerClick>,
    buttons: Query<&SlotDeleteButton>,
    mut ui: ResMut<SlotPickerUi>,
    commands: Commands,
    roots: Query<Entity, With<SlotPickerRoot>>,
) {
    if !buttons.contains(click.event().entity) {
        return;
    }
    ui.confirm_delete = true;
    rebuild_picker(commands, roots, &ui);
}

fn on_cancel_delete(
    click: On<PointerClick>,
    buttons: Query<&SlotCancelDeleteButton>,
    mut ui: ResMut<SlotPickerUi>,
    commands: Commands,
    roots: Query<Entity, With<SlotPickerRoot>>,
) {
    if !buttons.contains(click.event().entity) {
        return;
    }
    ui.confirm_delete = false;
    rebuild_picker(commands, roots, &ui);
}

fn on_confirm_delete(
    click: On<PointerClick>,
    buttons: Query<&SlotConfirmDeleteButton>,
    mut ui: ResMut<SlotPickerUi>,
    commands: Commands,
    roots: Query<Entity, With<SlotPickerRoot>>,
    mut notice: ResMut<AppNotice>,
) {
    if !buttons.contains(click.event().entity) {
        return;
    }
    let Some(id) = ui.selected else {
        return;
    };
    if let Err(e) = delete_slot_files(&slot_path(id)) {
        warn!("Failed to delete save slot {id}: {e}");
        notice.show(format!("Delete failed: {e}"));
    } else {
        ui.slots.retain(|slot| slot.slot_id != id);
        ui.selected = ui.slots.first().map(|slot| slot.slot_id);
        notice.show("Save slot deleted");
    }
    ui.confirm_delete = false;
    rebuild_picker(commands, roots, &ui);
}

fn on_close_picker(
    click: On<PointerClick>,
    buttons: Query<&SlotCloseButton>,
    mut picker: ResMut<NextState<SlotPickerState>>,
) {
    if buttons.contains(click.event().entity) {
        picker.set(SlotPickerState::Closed);
    }
}

pub fn slot_picker_input(
    keys: Res<ButtonInput<KeyCode>>,
    picker: Res<State<SlotPickerState>>,
    mut next: ResMut<NextState<SlotPickerState>>,
) {
    if *picker.get() == SlotPickerState::Open && keys.just_pressed(KeyCode::Escape) {
        next.set(SlotPickerState::Closed);
    }
}

fn small_node(width: f32, height: f32) -> Node {
    Node {
        width: Val::Px(width),
        height: Val::Px(height),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    }
}

fn format_play_time(secs: f32) -> String {
    let total = secs.max(0.0) as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

fn format_saved(updated_at_ms: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(updated_at_ms);
    let elapsed = now.saturating_sub(updated_at_ms) / 1000;
    match elapsed {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{} min ago", elapsed / 60),
        3600..=86399 => format!("{} h ago", elapsed / 3600),
        _ => format!("{} d ago", elapsed / 86400),
    }
}
