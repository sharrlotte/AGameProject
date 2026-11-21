use bevy::prelude::*;
use bevy::{audio::Volume, post_process::bloom::Bloom};

use crate::config::GameSettings;
use crate::scene::widget::{ButtonVisuals, spawn_button, spawn_widget};
use crate::ui::{Modal, ModalRoot, ModalStack, modal_is};
use crate::simulation::SimulationSet;

#[derive(Component)]
pub struct SettingRoot;

#[derive(Component)]
pub struct VolumeText;

#[derive(Component)]
pub struct BloomText;

#[derive(Component)]
pub struct AiSkillText;

#[derive(Component)]
struct VolumeButton(pub f32);

#[derive(Component)]
struct AiSkillButton(pub i8);

#[derive(Component)]
struct BloomButton;

#[derive(Component)]
struct CloseSettingsButton;

pub struct SettingScenePlugin;

impl Plugin for SettingScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameSettings>()
            .add_systems(
                Update,
                (
                    sync_settings_to_ui
                        .run_if(modal_is(Modal::Settings))
                        .in_set(SimulationSet::Input),
                    sync_setting_ui.in_set(SimulationSet::Present),
                ),
            )
            .add_systems(Update, apply_settings.in_set(SimulationSet::Input));
    }
}

fn on_volume(
    click: On<PointerClick>,
    buttons: Query<&VolumeButton>,
    mut setting: ResMut<GameSettings>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    setting.volume = (setting.volume + button.0).clamp(0.0, 1.0);
}

fn on_bloom(
    click: On<PointerClick>,
    buttons: Query<&BloomButton>,
    mut setting: ResMut<GameSettings>,
) {
    if buttons.contains(click.event().entity) {
        setting.bloom = !setting.bloom;
    }
}

fn on_ai_skill(
    click: On<PointerClick>,
    buttons: Query<&AiSkillButton>,
    mut setting: ResMut<GameSettings>,
) {
    let Ok(button) = buttons.get(click.event().entity) else {
        return;
    };
    setting.ai_level = (setting.ai_level as i16 + button.0 as i16).clamp(1, 10) as u8;
}

fn on_close(
    click: On<PointerClick>,
    buttons: Query<&CloseSettingsButton>,
    mut stack: ResMut<ModalStack>,
) {
    if buttons.contains(click.event().entity) {
        stack.remove(Modal::Settings);
    }
}

fn sync_setting_ui(
    mut commands: Commands,
    stack: Res<ModalStack>,
    roots: Query<(Entity, &ModalRoot)>,
    setting: Res<GameSettings>,
) {
    let open = stack.contains(Modal::Settings);
    let existing = roots.iter().find(|(_, root)| root.0 == Modal::Settings).map(|(e, _)| e);
    match (open, existing) {
        (true, None) => setup_setting_ui(&mut commands, &setting),
        (false, Some(entity)) => commands.entity(entity).despawn(),
        _ => {}
    }
}

pub fn setup_setting_ui(commands: &mut Commands, setting: &GameSettings) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.99)),
            SettingRoot,
            ModalRoot(Modal::Settings),
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

            parent
                .spawn(Node {
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("AI Skill"),
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
                        AiSkillButton(-1),
                        on_ai_skill,
                        ButtonVisuals::dark(),
                    );

                    row.spawn((
                        Text::new(format!("{}", setting.ai_level.clamp(1, 10))),
                        TextFont {
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                        AiSkillText,
                    ));

                    spawn_button(
                        row,
                        small_button_node(40.0, 40.0),
                        "+",
                        20.0,
                        Color::WHITE,
                        AiSkillButton(1),
                        on_ai_skill,
                        ButtonVisuals::dark(),
                    );
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
    setting: Res<GameSettings>,
    mut volume_text: Query<&mut Text, (With<VolumeText>, Without<BloomText>, Without<AiSkillText>)>,
    mut bloom_text: Query<&mut Text, (With<BloomText>, Without<VolumeText>, Without<AiSkillText>)>,
    mut ai_skill_text: Query<&mut Text, (With<AiSkillText>, Without<VolumeText>, Without<BloomText>)>,
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
        for mut text in &mut ai_skill_text {
            text.0 = format!("{}", setting.ai_level.clamp(1, 10));
        }
    }
}

pub fn apply_settings(
    setting: Res<GameSettings>,
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
