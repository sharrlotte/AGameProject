use bevy::prelude::*;
use bevy::audio::{GlobalVolume, Volume};
use bevy::post_process::bloom::Bloom;

use crate::resource::SettingState;
use crate::scene::setting::resource::GameSetting;
use crate::scene::setting::component::*;

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
            GlobalZIndex(100),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Settings"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Volume Control
            parent.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                ..default()
            }).with_children(|row| {
                row.spawn((
                    Text::new("Volume"),
                    TextFont { font_size: 24.0, ..default() },
                ));
                
                // Decrease
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    BorderRadius::all(Val::Px(5.0)),
                    VolumeChange(-0.1),
                )).with_children(|btn| {
                    btn.spawn(Text::new("-"));
                });

                // Display Value
                row.spawn((
                    Text::new(format!("{:.0}%", setting.volume * 100.0)),
                     TextFont { font_size: 24.0, ..default() },
                     VolumeText,
                ));

                // Increase
                 row.spawn((
                    Button,
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    BorderRadius::all(Val::Px(5.0)),
                    VolumeChange(0.1),
                )).with_children(|btn| {
                    btn.spawn(Text::new("+"));
                });
            });

            // Bloom Control
             parent.spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(20.0),
                ..default()
            }).with_children(|row| {
                row.spawn((
                    Text::new("Bloom"),
                    TextFont { font_size: 24.0, ..default() },
                ));
                
                row.spawn((
                    Button,
                    Node {
                        width: Val::Px(100.0),
                        height: Val::Px(40.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                    BorderRadius::all(Val::Px(5.0)),
                    BloomButton,
                )).with_children(|btn| {
                    btn.spawn((
                        Text::new(if setting.bloom { "On" } else { "Off" }),
                        BloomText,
                    ));
                });
            });

            // Close Button
             parent.spawn((
                Button,
                Node {
                    width: Val::Px(150.0),
                    height: Val::Px(50.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    margin: UiRect::top(Val::Px(20.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
                BorderRadius::all(Val::Px(5.0)),
                CloseButton,
            )).with_children(|btn| {
                btn.spawn(Text::new("Close"));
            });
        });
}

pub fn cleanup_setting_ui(mut commands: Commands, query: Query<Entity, With<SettingRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

pub fn handle_close_button(
    mut interaction: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<CloseButton>)>,
    mut next_state: ResMut<NextState<SettingState>>,
) {
    for (interaction, mut bg) in &mut interaction {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(SettingState::Closed);
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.6, 0.3, 0.3)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.5, 0.2, 0.2)),
        }
    }
}

pub fn handle_volume_change(
    mut interaction: Query<(&Interaction, &VolumeChange, &mut BackgroundColor), (Changed<Interaction>, With<Button>)>,
    mut setting: ResMut<GameSetting>,
) {
    for (interaction, change, mut bg) in &mut interaction {
        match *interaction {
            Interaction::Pressed => {
                setting.volume = (setting.volume + change.0).clamp(0.0, 1.0);
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
        }
    }
}

pub fn handle_bloom_toggle(
    mut interaction: Query<(&Interaction, &mut BackgroundColor), (Changed<Interaction>, With<BloomButton>)>,
    mut setting: ResMut<GameSetting>,
) {
    for (interaction, mut bg) in &mut interaction {
        match *interaction {
            Interaction::Pressed => {
                setting.bloom = !setting.bloom;
            }
            Interaction::Hovered => *bg = BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
            Interaction::None => *bg = BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
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
            text.0 = if setting.bloom { "On".to_string() } else { "Off".to_string() };
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
