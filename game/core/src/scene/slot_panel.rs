use bevy::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::resource::{AppNotice, PauseState};
use crate::save::io::{LoadWorldEvent, SaveWorldEvent, list_slots};
use crate::save::schema::SlotInfo;
use crate::scene::layout::{
    SLOT_BUTTON_HEIGHT, SLOT_BUTTON_WIDTH, SLOT_ROW_HEIGHT, SLOT_ROW_WIDTH,
};
use crate::scene::widget::{ButtonVisuals, spawn_button};
use crate::simulation::SimulationSet;

/// Shared five-slot save/load list, used by Home and Pause.
pub struct SlotPanelPlugin;

impl Plugin for SlotPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RefreshSlotPanel>()
            .add_systems(Update, refresh_slot_panel.in_set(SimulationSet::Present));
    }
}

#[derive(Message)]
pub struct RefreshSlotPanel;

#[derive(Component)]
pub struct SlotPanel {
    pub allow_save: bool,
}

#[derive(Component)]
struct LoadSlotButton(pub u8);

fn on_load_slot(
    click: On<PointerClick>,
    buttons: Query<&LoadSlotButton>,
    mut pause: ResMut<NextState<PauseState>>,
    mut loads: MessageWriter<LoadWorldEvent>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    pause.set(PauseState::Playing);
    loads.write(LoadWorldEvent { slot: button.0 });
}

#[derive(Component)]
struct SaveSlotButton(pub u8);

fn on_save_slot(
    click: On<PointerClick>,
    buttons: Query<&SaveSlotButton>,
    mut saves: MessageWriter<SaveWorldEvent>,
    mut notice: ResMut<AppNotice>,
    mut refresh: MessageWriter<RefreshSlotPanel>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    saves.write(SaveWorldEvent { slot: button.0 });
    notice.show(format!("Saving slot {}…", button.0));
    refresh.write(RefreshSlotPanel);
}

/// Build the panel's children. Called during scene setup and on refresh.
pub fn build_slot_panel(parent: &mut ChildSpawnerCommands, allow_save: bool) {
    let slots = list_slots();
    parent.spawn((
        Text::new("Save Slots"),
        TextFont { font_size: FontSize::Px(18.0), ..default() },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
    ));

    for info in &slots {
        parent
            .spawn((
                Node {
                    width: Val::Px(SLOT_ROW_WIDTH),
                    min_height: Val::Px(SLOT_ROW_HEIGHT),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
            ))
            .with_children(|row| {
                row.spawn((
                    Text::new(slot_label(info)),
                    TextFont { font_size: FontSize::Px(14.0), ..default() },
                    TextColor(Color::WHITE),
                ));

                spawn_button(row, slot_button_node(), "Load", 12.0, Color::WHITE,
                    LoadSlotButton(info.slot), on_load_slot, ButtonVisuals::blue());

                if allow_save {
                    spawn_button(row, slot_button_node(), "Save", 12.0, Color::WHITE,
                        SaveSlotButton(info.slot), on_save_slot, ButtonVisuals::green());
                }
            });
    }
}

fn slot_button_node() -> Node {
    Node {
        width: Val::Px(SLOT_BUTTON_WIDTH),
        height: Val::Px(SLOT_BUTTON_HEIGHT),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(Val::Px(4.0)),
        ..default()
    }
}

fn slot_label(info: &SlotInfo) -> String {
    if !info.occupied {
        return format!("Slot {} — empty", info.slot);
    }
    format!(
        "Slot {} — {} — {} ships — {:.1}s — {}",
        info.slot, info.mission, info.ship_count, info.elapsed_secs,
        relative_time(info.saved_at_unix_ms)
    )
}

fn relative_time(saved_at_unix_ms: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(saved_at_unix_ms);
    let secs = now.saturating_sub(saved_at_unix_ms) / 1000;
    if secs < 60 {
        "just now".to_string()
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86_400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86_400)
    }
}

fn refresh_slot_panel(
    mut commands: Commands,
    mut events: MessageReader<RefreshSlotPanel>,
    panels: Query<(Entity, &SlotPanel)>,
) {
    if events.read().next().is_none() {
        return;
    }
    for (entity, panel) in &panels {
        commands.entity(entity).despawn_children();
        let allow_save = panel.allow_save;
        commands.entity(entity).with_children(move |parent| {
            build_slot_panel(parent, allow_save);
        });
    }
}
