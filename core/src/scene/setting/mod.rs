use bevy::{audio::Volume, post_process::bloom::Bloom, prelude::*};

use crate::config::GameSetting;
use crate::mission::BattleResult;
use crate::resource::{AppNotice, PendingLoad, SceneState, SettingState};
use crate::save::battle::{BattleSaveState, reset_campaign_data};
use crate::save::campaign::CampaignSave;
use crate::scene::widget::{ButtonVisuals, spawn_button, spawn_widget};
use crate::simulation::SimulationSet;

#[derive(Component)]
pub struct SettingRoot;

#[derive(Component)]
pub struct VolumeText;

#[derive(Component)]
pub struct BloomText;

#[derive(Component)]
struct VolumeButton(pub f32);

#[derive(Component)]
struct BloomButton;

#[derive(Component)]
struct CloseSettingsButton;

#[derive(Component)]
struct DataSection;

#[derive(Component)]
struct ResetPromptButton;

#[derive(Component)]
struct ConfirmResetButton;

#[derive(Component)]
struct CancelResetButton;

pub struct SettingScenePlugin;

impl Plugin for SettingScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SettingState>()
            .init_resource::<GameSetting>()
            .add_systems(OnEnter(SettingState::Open), setup_setting_ui)
            .add_systems(
                Update,
                sync_settings_to_ui
                    .run_if(in_state(SettingState::Open))
                    .in_set(SimulationSet::Input),
            )
            .add_systems(Update, apply_settings.in_set(SimulationSet::Input));
    }
}

fn on_volume(
    click: On<PointerClick>,
    buttons: Query<&VolumeButton>,
    mut setting: ResMut<GameSetting>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    setting.volume = (setting.volume + button.0).clamp(0.0, 1.0);
}

fn on_bloom(
    click: On<PointerClick>,
    buttons: Query<&BloomButton>,
    mut setting: ResMut<GameSetting>,
) {
    if buttons.contains(click.event().entity) {
        setting.bloom = !setting.bloom;
    }
}

fn on_close(
    click: On<PointerClick>,
    buttons: Query<&CloseSettingsButton>,
    mut next: ResMut<NextState<SettingState>>,
) {
    if buttons.contains(click.event().entity) {
        next.set(SettingState::Closed);
    }
}

fn spawn_data_idle(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Text::new("Data"),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        TextColor(Color::srgb(0.85, 0.85, 0.85)),
    ));
    spawn_button(
        parent,
        small_button_node(260.0, 42.0),
        "Reset Mission / Campaign",
        18.0,
        Color::WHITE,
        ResetPromptButton,
        on_reset_prompt,
        ButtonVisuals::red(),
    );
}

fn spawn_data_confirm(parent: &mut ChildSpawnerCommands) {
    parent.spawn((
        Text::new("Reset all mission & campaign progress?"),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.6, 0.6)),
    ));
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|row| {
            spawn_button(
                row,
                small_button_node(150.0, 40.0),
                "Confirm Reset",
                16.0,
                Color::WHITE,
                ConfirmResetButton,
                on_confirm_reset,
                ButtonVisuals::red(),
            );
            spawn_button(
                row,
                small_button_node(110.0, 40.0),
                "Cancel",
                16.0,
                Color::WHITE,
                CancelResetButton,
                on_cancel_reset,
                ButtonVisuals::dark(),
            );
        });
}

fn rebuild_data_section(commands: &mut Commands, sections: &Query<Entity, With<DataSection>>, confirm: bool) {
    for entity in sections {
        commands.entity(entity).despawn_children();
        commands.entity(entity).with_children(move |parent| {
            if confirm {
                spawn_data_confirm(parent);
            } else {
                spawn_data_idle(parent);
            }
        });
    }
}

fn on_reset_prompt(
    click: On<PointerClick>,
    buttons: Query<&ResetPromptButton>,
    sections: Query<Entity, With<DataSection>>,
    mut commands: Commands,
) {
    if buttons.contains(click.event().entity) {
        rebuild_data_section(&mut commands, &sections, true);
    }
}

fn on_cancel_reset(
    click: On<PointerClick>,
    buttons: Query<&CancelResetButton>,
    sections: Query<Entity, With<DataSection>>,
    mut commands: Commands,
) {
    if buttons.contains(click.event().entity) {
        rebuild_data_section(&mut commands, &sections, false);
    }
}

fn on_confirm_reset(
    click: On<PointerClick>,
    buttons: Query<&ConfirmResetButton>,
    sections: Query<Entity, With<DataSection>>,
    mut commands: Commands,
    mut campaign: ResMut<CampaignSave>,
    mut battle_save: ResMut<BattleSaveState>,
    mut pending: ResMut<PendingLoad>,
    mut battle: ResMut<BattleResult>,
    mut notice: ResMut<AppNotice>,
) {
    if !buttons.contains(click.event().entity) {
        return;
    }
    reset_campaign_data();
    campaign.reset();
    battle_save.has_save = false;
    *pending = PendingLoad::default();
    *battle = BattleResult::default();
    notice.show("Campaign data reset");
    rebuild_data_section(&mut commands, &sections, false);
}

pub fn setup_setting_ui(
    mut commands: Commands,
    setting: Res<GameSetting>,
    scene: Res<State<SceneState>>,
) {
    let mut bloom_button = None;
    let is_home = *scene.get() == SceneState::Home;

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.99)),
            SettingRoot,
            DespawnOnExit(SettingState::Open),
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: FontSize::Px(40.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Volume"),
                        TextFont {
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                    ));

                    spawn_button(
                        row,
                        small_button_node(40.0, 40.0),
                        "-",
                        20.0,
                        Color::WHITE,
                        VolumeButton(-0.1),
                        on_volume,
                        ButtonVisuals::dark(),
                    );

                    row.spawn((
                        Text::new(format!("{:.0}%", setting.volume * 100.0)),
                        TextFont {
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                        VolumeText,
                    ));

                    spawn_button(
                        row,
                        small_button_node(40.0, 40.0),
                        "+",
                        20.0,
                        Color::WHITE,
                        VolumeButton(0.1),
                        on_volume,
                        ButtonVisuals::dark(),
                    );
                });

            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Bloom"),
                        TextFont {
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                    ));

                    bloom_button = Some(spawn_widget(
                        row,
                        small_button_node(100.0, 40.0),
                        BloomButton,
                        on_bloom,
                        ButtonVisuals::dark(),
                    ));
                });

            if is_home {
                parent
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(10.0),
                            margin: UiRect::top(Val::Px(12.0)),
                            ..default()
                        },
                        DataSection,
                    ))
                    .with_children(spawn_data_idle);
            }

            spawn_button(
                parent,
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    border_radius: BorderRadius::all(Val::Px(5.0)),
                    ..default()
                },
                "Close",
                20.0,
                Color::WHITE,
                CloseSettingsButton,
                on_close,
                ButtonVisuals::red(),
            );
        });

    if let Some(entity) = bloom_button {
        commands.entity(entity).with_children(|button| {
            button.spawn((
                Text::new(if setting.bloom { "On" } else { "Off" }),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                BloomText,
            ));
        });
    }
}

fn small_button_node(width: f32, height: f32) -> Node {
    Node {
        width: Val::Px(width),
        height: Val::Px(height),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(Val::Px(5.0)),
        ..default()
    }
}

pub fn sync_settings_to_ui(
    setting: Res<GameSetting>,
    mut volume_text: Query<&mut Text, (With<VolumeText>, Without<BloomText>)>,
    mut bloom_text: Query<&mut Text, (With<BloomText>, Without<VolumeText>)>,
) {
    if setting.is_changed() {
        for mut text in &mut volume_text {
            text.0 = format!("{:.0}%", setting.volume * 100.0);
        }
        for mut text in &mut bloom_text {
            text.0 = if setting.bloom {
                "On".to_string()
            } else {
                "Off".to_string()
            };
        }
    }
}

pub fn apply_settings(
    setting: Res<GameSetting>,
    mut volume: ResMut<GlobalVolume>,
    mut camera_query: Query<&mut Bloom, With<Camera>>,
) {
    if setting.is_changed() {
        volume.volume = Volume::Linear(setting.volume);

        for mut bloom in &mut camera_query {
            bloom.intensity = if setting.bloom { 0.3 } else { 0.0 };
        }
    }
}
