use bevy::{audio::Volume, post_process::bloom::Bloom, prelude::*};

use crate::config::GameSetting;
use crate::resource::SettingState;
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

fn on_volume(
    click: On<PointerClick>,
    buttons: Query<&VolumeButton>,
    mut setting: ResMut<GameSetting>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    setting.volume = (setting.volume + button.0).clamp(0.0, 1.0);
}

#[derive(Component)]
struct BloomButton;

fn on_bloom(
    click: On<PointerClick>,
    buttons: Query<&BloomButton>,
    mut setting: ResMut<GameSetting>,
) {
    if buttons.contains(click.event().entity) {
        setting.bloom = !setting.bloom;
    }
}

#[derive(Component)]
struct CloseSettingsButton;

fn on_close(
    click: On<PointerClick>,
    buttons: Query<&CloseSettingsButton>,
    mut next: ResMut<NextState<SettingState>>,
) {
    if buttons.contains(click.event().entity) {
        next.set(SettingState::Closed);
    }
}

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

pub fn setup_setting_ui(mut commands: Commands, setting: Res<GameSetting>) {
    let mut bloom_button = None;

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
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
