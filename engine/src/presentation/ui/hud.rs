use bevy::prelude::*;
use crate::common::GameState;
use crate::combat::WeaponInventory;
use crate::message::resources::RunStats;
use crate::common::GameScene;
use crate::message::resources::ScrapBank;
use crate::common::SimUpdateSet;
use crate::world::{Health, LocalPlayer, Stamina};

use super::theme::spawn_progress_bar;
#[cfg(feature = "dev")]
use super::theme::{BTN_SECONDARY_HOVER, BTN_SECONDARY_IDLE, spawn_action_button};

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct HpBarFill;

#[derive(Component)]
struct HpLabelText;

#[derive(Component)]
struct StaminaBarFill;

#[derive(Component)]
struct WeaponLabelText;

#[derive(Component)]
struct ReloadBarBg;

#[derive(Component)]
struct ReloadBarFill;

#[derive(Component)]
struct RightStatsText;

#[derive(Component)]
struct ExtractionLabelText;

#[derive(Component)]
struct ExtractionBarFill;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(
                OnEnter(GameScene::Arena),
                spawn_hud.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                update_hud
                    .after(SimUpdateSet)
                    .run_if(in_state(GameState::Playing)),
            );

        #[cfg(feature = "dev")]
        app.add_systems(
            OnEnter(GameScene::AnimTest),
            spawn_hud.run_if(in_state(GameState::Playing)),
        );
    }
}

fn spawn_hud(
    mut commands: Commands,
    scene: Res<State<GameScene>>,
    existing: Query<(), With<HudRoot>>,
) {
    if !existing.is_empty() {
        return;
    }
    #[cfg(feature = "dev")]
    let is_anim_test = *scene.get() == GameScene::AnimTest;
    #[cfg(not(feature = "dev"))]
    let is_anim_test = false;
    commands
        .spawn((
            Name::new("HudRoot"),
            HudRoot,
            DespawnOnExit(GameState::Playing),
            DespawnOnExit(*scene.get()),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(14.0),
                    left: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(6.0),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.08, 0.11, 0.78)),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel.spawn((
                    HpLabelText,
                    Text::new("HP: 100 / 100"),
                    TextFont {
                        font_size: FontSize::Px(17.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.96, 0.98)),
                ));
                spawn_progress_bar(
                    panel,
                    220.0,
                    14.0,
                    Color::srgb(0.18, 0.08, 0.08),
                    Color::srgb(0.86, 0.24, 0.22),
                    HpBarFill,
                );
                spawn_progress_bar(
                    panel,
                    220.0,
                    6.0,
                    Color::srgb(0.10, 0.14, 0.18),
                    Color::srgb(0.28, 0.76, 0.95),
                    StaminaBarFill,
                );
                panel.spawn((
                    WeaponLabelText,
                    Text::new("[1] Assault Rifle  30 / 30"),
                    TextFont {
                        font_size: FontSize::Px(17.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.96, 0.88, 0.54)),
                ));
                panel
                    .spawn((
                        ReloadBarBg,
                        Node {
                            width: Val::Px(220.0),
                            height: Val::Px(6.0),
                            display: Display::None,
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.18, 0.16, 0.10)),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            ReloadBarFill,
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(1.0, 0.78, 0.22)),
                        ));
                    });
            });

            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(14.0),
                    right: Val::Px(16.0),
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(12.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.06, 0.08, 0.11, 0.78)),
                Pickable::IGNORE,
            ))
            .with_children(|panel| {
                panel.spawn((
                    RightStatsText,
                    Text::new("00:00  |  Scrap: +0 (Bank: 0)"),
                    TextFont {
                        font_size: FontSize::Px(18.0),
                        ..default()
                    },
                    TextColor(Color::srgb(0.95, 0.96, 0.98)),
                ));
            });

            #[cfg(feature = "dev")]
            spawn_dev_stage_controls(root, is_anim_test);

            if !is_anim_test {
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(18.0),
                        left: Val::Percent(50.0),
                        margin: UiRect::left(Val::Px(-150.0)),
                        width: Val::Px(300.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.06, 0.08, 0.11, 0.78)),
                    Pickable::IGNORE,
                ))
                .with_children(|panel| {
                    panel.spawn((
                        ExtractionLabelText,
                        Text::new("EXTRACTION PAD (North): 0.0 / 10.0s"),
                        TextFont {
                            font_size: FontSize::Px(15.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.55, 0.95, 0.85)),
                    ));
                    spawn_progress_bar(
                        panel,
                        268.0,
                        10.0,
                        Color::srgb(0.10, 0.16, 0.16),
                        Color::srgb(0.24, 0.92, 0.62),
                        ExtractionBarFill,
                    );
                });
            }
        });
}

#[cfg(feature = "dev")]
fn spawn_dev_stage_controls(root: &mut ChildSpawnerCommands, is_anim_test: bool) {
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(14.0),
            left: Val::Percent(50.0),
            margin: UiRect::left(Val::Px(-180.0)),
            width: Val::Px(360.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(6.0),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|panel| {
        if is_anim_test {
            panel.spawn((
                Pickable::IGNORE,
                Text::new("ANIMATION TEST STAGE (No Enemies)"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                TextColor(Color::srgb(0.45, 0.92, 0.68)),
            ));
        }
        let label = if is_anim_test {
            "Exit Test Stage (F8)"
        } else {
            "Anim Test Stage (F8)"
        };
        spawn_action_button(
            panel,
            label,
            BTN_SECONDARY_IDLE,
            BTN_SECONDARY_HOVER,
            super::ToggleAnimTestSceneMessage,
        );
    });
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn update_hud(
    stats: Res<RunStats>,
    bank: Res<ScrapBank>,
    players: Query<(&Health, &Stamina, Option<&WeaponInventory>), With<LocalPlayer>>,
    mut hp_text: Query<
        &mut Text,
        (
            With<HpLabelText>,
            Without<WeaponLabelText>,
            Without<RightStatsText>,
            Without<ExtractionLabelText>,
        ),
    >,
    mut weapon_text: Query<
        &mut Text,
        (
            With<WeaponLabelText>,
            Without<HpLabelText>,
            Without<RightStatsText>,
            Without<ExtractionLabelText>,
        ),
    >,
    mut right_text: Query<
        &mut Text,
        (
            With<RightStatsText>,
            Without<HpLabelText>,
            Without<WeaponLabelText>,
            Without<ExtractionLabelText>,
        ),
    >,
    mut extract_text: Query<
        &mut Text,
        (
            With<ExtractionLabelText>,
            Without<HpLabelText>,
            Without<WeaponLabelText>,
            Without<RightStatsText>,
        ),
    >,
    mut hp_fill: Query<
        &mut Node,
        (
            With<HpBarFill>,
            Without<StaminaBarFill>,
            Without<ReloadBarBg>,
            Without<ReloadBarFill>,
            Without<ExtractionBarFill>,
        ),
    >,
    mut stamina_fill: Query<
        &mut Node,
        (
            With<StaminaBarFill>,
            Without<HpBarFill>,
            Without<ReloadBarBg>,
            Without<ReloadBarFill>,
            Without<ExtractionBarFill>,
        ),
    >,
    mut reload_bg: Query<
        &mut Node,
        (
            With<ReloadBarBg>,
            Without<HpBarFill>,
            Without<StaminaBarFill>,
            Without<ReloadBarFill>,
            Without<ExtractionBarFill>,
        ),
    >,
    mut reload_fill: Query<
        &mut Node,
        (
            With<ReloadBarFill>,
            Without<HpBarFill>,
            Without<StaminaBarFill>,
            Without<ReloadBarBg>,
            Without<ExtractionBarFill>,
        ),
    >,
    mut extract_fill: Query<
        &mut Node,
        (
            With<ExtractionBarFill>,
            Without<HpBarFill>,
            Without<StaminaBarFill>,
            Without<ReloadBarBg>,
            Without<ReloadBarFill>,
        ),
    >,
) {
    if let Some((health, stamina, inventory)) = players.iter().next() {
        let hp_pct = if health.max > 0.0 {
            (health.current / health.max * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        if let Some(mut txt) = hp_text.iter_mut().next() {
            txt.0 = format!("HP: {:.0} / {:.0}", health.current, health.max);
        }
        if let Some(mut node) = hp_fill.iter_mut().next() {
            node.width = Val::Percent(hp_pct);
        }

        let stam_pct = if stamina.max > 0.0 {
            (stamina.current / stamina.max * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        if let Some(mut node) = stamina_fill.iter_mut().next() {
            node.width = Val::Percent(stam_pct);
        }

        if let Some(inv) = inventory
            && let Some(active) = inv.active()
        {
            if let Some(mut txt) = weapon_text.iter_mut().next() {
                txt.0 = if let Some(remaining) = inv.reload_timer {
                    format!(
                        "[{}] {}   RELOADING ({:.1}s)",
                        inv.active_slot + 1,
                        active.weapon.name,
                        remaining
                    )
                } else {
                    format!(
                        "[{}] {}   {} / {}",
                        inv.active_slot + 1,
                        active.weapon.name,
                        active.ammo,
                        active.weapon.mag_size
                    )
                };
            }

            if let Some(mut bg) = reload_bg.iter_mut().next() {
                bg.display = if inv.reload_timer.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            if let Some(mut fill) = reload_fill.iter_mut().next() {
                let pct = match inv.reload_progress() {
                    Some(progress) => (progress * 100.0).clamp(0.0, 100.0),
                    None => 0.0,
                };
                fill.width = Val::Percent(pct);
            }
        }
    }

    let mins = (stats.elapsed_secs as u32) / 60;
    let secs = (stats.elapsed_secs as u32) % 60;
    if let Some(mut txt) = right_text.iter_mut().next() {
        txt.0 = format!(
            "Time: {:02}:{:02}   Kills: {}\nScrap: +{}   (Banked: {})",
            mins, secs, stats.kills, stats.run_scrap, bank.banked
        );
    }

    let hold = stats.extraction_hold_secs.max(0.01);
    let extract_pct = (stats.extraction_progress_secs / hold * 100.0).clamp(0.0, 100.0);
    if let Some(mut txt) = extract_text.iter_mut().next() {
        txt.0 = if stats.in_extraction_zone {
            format!(
                "EXTRACTING... {:.1}s / {:.1}s",
                stats.extraction_progress_secs, hold
            )
        } else if stats.extraction_progress_secs > 0.0 {
            format!(
                "EXTRACTION PAUSED ({:.1}s / {:.1}s)",
                stats.extraction_progress_secs, hold
            )
        } else {
            format!("EXTRACTION PAD (North): 0.0s / {:.1}s", hold)
        };
    }
    if let Some(mut node) = extract_fill.iter_mut().next() {
        node.width = Val::Percent(extract_pct);
    }
}







