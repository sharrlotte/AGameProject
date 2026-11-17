use bevy::picking::hover::PickingInteraction;
use bevy::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::resource::{AppNotice, PauseState};
use crate::save::io::{LoadWorldEvent, SaveWorldEvent, list_slots};
use crate::save::schema::SlotInfo;
use crate::simulation::SimulationSet;

/// Shared five-slot save/load list, used by Home and Pause.
pub struct SlotPanelPlugin;

impl Plugin for SlotPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<RefreshSlotPanel>()
            .add_systems(Update, refresh_slot_panel.in_set(SimulationSet::Present))
            .add_systems(Update, handle_slot_buttons.in_set(SimulationSet::Input));
    }
}

#[derive(Message)]
pub struct RefreshSlotPanel;

#[derive(Component)]
pub struct SlotPanel {
    pub allow_save: bool,
}

#[derive(Component)]
pub struct SlotLoadButton(pub u8);

#[derive(Component)]
pub struct SlotSaveButton(pub u8);

/// Build the panel's children. Called during scene setup and on refresh.
pub fn build_slot_panel(parent: &mut ChildSpawnerCommands, allow_save: bool) {
    let slots = list_slots();
    parent.spawn((
        Text::new("Save Slots"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
    ));

    for info in &slots {
        parent
            .spawn((
                Node {
                    width: Val::Px(420.0),
                    min_height: Val::Px(34.0),
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
                    TextFont {
                        font_size: FontSize::Px(14.0),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                row.spawn((
                    Node {
                        width: Val::Px(60.0),
                        height: Val::Px(24.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.35, 0.55)),
                    SlotLoadButton(info.slot),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Load"),
                        TextFont {
                            font_size: FontSize::Px(12.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });

                if allow_save {
                    row.spawn((
                        Node {
                            width: Val::Px(60.0),
                            height: Val::Px(24.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.25, 0.5, 0.25)),
                        SlotSaveButton(info.slot),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("Save"),
                            TextFont {
                                font_size: FontSize::Px(12.0),
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                    });
                }
            });
    }
}

fn slot_label(info: &SlotInfo) -> String {
    if !info.occupied {
        return format!("Slot {} — empty", info.slot);
    }
    format!(
        "Slot {} — {} — {} ships — {:.1}s — {}",
        info.slot,
        info.mission,
        info.ship_count,
        info.elapsed_secs,
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

fn handle_slot_buttons(
    loads: Query<(&PickingInteraction, &SlotLoadButton), Changed<PickingInteraction>>,
    saves: Query<(&PickingInteraction, &SlotSaveButton), Changed<PickingInteraction>>,
    mut load_events: MessageWriter<LoadWorldEvent>,
    mut save_events: MessageWriter<SaveWorldEvent>,
    mut refresh: MessageWriter<RefreshSlotPanel>,
    mut pause: ResMut<NextState<PauseState>>,
    mut notice: ResMut<AppNotice>,
) {
    for (interaction, button) in &loads {
        if *interaction == PickingInteraction::Pressed {
            pause.set(PauseState::Playing);
            load_events.write(LoadWorldEvent { slot: button.0 });
        }
    }
    for (interaction, button) in &saves {
        if *interaction == PickingInteraction::Pressed {
            save_events.write(SaveWorldEvent { slot: button.0 });
            notice.show(format!("Saving slot {}…", button.0));
            refresh.write(RefreshSlotPanel);
        }
    }
}
