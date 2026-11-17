use bevy::{audio::Volume, picking::hover::PickingInteraction, post_process::bloom::Bloom, prelude::*};

use crate::config::GameSetting;
use crate::resource::SettingState;
use crate::simulation::SimulationSet;

#[derive(Component)]
pub struct SettingRoot;

#[derive(Component)]
pub struct VolumeChange(pub f32);

#[derive(Component)]
pub struct VolumeText;

#[derive(Component)]
pub struct BloomButton;

#[derive(Component)]
pub struct BloomText;

#[derive(Component)]
pub struct CloseButton;

pub struct SettingScenePlugin;

impl Plugin for SettingScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SettingState>()
            .init_resource::<GameSetting>()
            .add_systems(OnEnter(SettingState::Open), setup_setting_ui)
            .add_systems(
                Update,
                (
                    handle_close_button,
                    handle_volume_change,
                    handle_bloom_toggle,
                    sync_settings_to_ui,
                )
                    .run_if(in_state(SettingState::Open))
                    .in_set(SimulationSet::Input),
            )
            .add_systems(Update, apply_settings.in_set(SimulationSet::Input));
    }
}

pub fn setup_setting_ui(mut commands: Commands, setting: Res<GameSetting>) {
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
            // Title
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: FontSize::Px(40.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Volume Control
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

                    // Decrease
                    row.spawn((

                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(5.0)),
                            ..default()
                        },

                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        VolumeChange(-0.1),
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("-"));
                    });

                    // Display Value
                    row.spawn((
                        Text::new(format!("{:.0}%", setting.volume * 100.0)),
                        TextFont {
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                        VolumeText,
                    ));

                    // Increase
                    row.spawn((

                        Node {
                            width: Val::Px(40.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(5.0)),
                            ..default()
                        },

                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        VolumeChange(0.1),
                    ))
                    .with_children(|btn| {
                        btn.spawn(Text::new("+"));
                    });
                });

            // Bloom Control
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

                    row.spawn((

                        Node {
                            width: Val::Px(100.0),
                            height: Val::Px(40.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border_radius: BorderRadius::all(Val::Px(5.0)),
                            ..default()
                        },

                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        BloomButton,
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new(if setting.bloom { "On" } else { "Off" }),
                            BloomText,
                        ));
                    });
                });

            // Close Button
            parent
                .spawn((

                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(20.0)),
                        border_radius: BorderRadius::all(Val::Px(5.0)),
                        ..default()
                    },

                    BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                    CloseButton,
                ))
                .with_children(|btn| {
                    btn.spawn(Text::new("Close"));
                });
        });
}

pub fn handle_close_button(
    mut interaction: Query<
        (&PickingInteraction, &mut BackgroundColor),
        (Changed<PickingInteraction>, With<CloseButton>),
    >,
    mut next_state: ResMut<NextState<SettingState>>,
) {
    for (interaction, mut bg) in &mut interaction {
        match *interaction {
            PickingInteraction::Pressed => {
                next_state.set(SettingState::Closed);
            }
            PickingInteraction::Hovered => *bg = BackgroundColor(Color::srgb(0.6, 0.3, 0.3)),
            PickingInteraction::None => *bg = BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
        }
    }
}

pub fn handle_volume_change(
    mut interaction: Query<
        (&PickingInteraction, &VolumeChange, &mut BackgroundColor),
        Changed<PickingInteraction>,
    >,
    mut setting: ResMut<GameSetting>,
) {
    for (interaction, change, mut bg) in &mut interaction {
        match *interaction {
            PickingInteraction::Pressed => {
                setting.volume = (setting.volume + change.0).clamp(0.0, 1.0);
            }
            PickingInteraction::Hovered => *bg = BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
            PickingInteraction::None => *bg = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        }
    }
}

pub fn handle_bloom_toggle(
    mut interaction: Query<
        (&PickingInteraction, &mut BackgroundColor),
        (Changed<PickingInteraction>, With<BloomButton>),
    >,
    mut setting: ResMut<GameSetting>,
) {
    for (interaction, mut bg) in &mut interaction {
        match *interaction {
            PickingInteraction::Pressed => {
                setting.bloom = !setting.bloom;
            }
            PickingInteraction::Hovered => *bg = BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
            PickingInteraction::None => *bg = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        }
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
