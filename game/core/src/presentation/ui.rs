use bevy::prelude::*;

use crate::GameState;
use crate::sim::RunStats;
use crate::sim::combat::WeaponInventory;
use crate::sim::player::{Health, LocalPlayer, Stamina};

const BTN_IDLE_COLOR: Color = Color::srgb(0.18, 0.45, 0.78);
const BTN_HOVER_COLOR: Color = Color::srgb(0.25, 0.56, 0.90);

#[derive(Component)]
pub struct RunHudRoot;

#[derive(Component)]
pub struct RunHudText;

#[derive(Component)]
pub struct GameOverUiRoot;

#[derive(Component)]
pub struct RestartButton;

/// Spawns the minimal corner status readout while playing.
pub fn spawn_run_hud(mut commands: Commands) {
    commands
        .spawn((
            Name::new("RunHudRoot"),
            RunHudRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(14.0),
                left: Val::Px(16.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.06, 0.08, 0.11, 0.72)),
        ))
        .with_children(|parent| {
            parent.spawn((
                RunHudText,
                Text::new("Initializing..."),
                TextFont {
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.95, 0.98)),
            ));
        });
}

pub fn despawn_run_hud(mut commands: Commands, roots: Query<Entity, With<RunHudRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

/// Updates the temporary corner readout for HP, stamina, active weapon, ammo/reload, and kills.
pub fn update_run_hud(
    stats: Res<RunStats>,
    players: Query<(&Health, &Stamina, &WeaponInventory), With<LocalPlayer>>,
    mut texts: Query<&mut Text, With<RunHudText>>,
) {
    let Some((health, stamina, inventory)) = players.iter().next() else {
        return;
    };
    let Some(mut text) = texts.iter_mut().next() else {
        return;
    };

    let active = inventory.active();
    let ammo_line = match inventory.reload_timer {
        Some(rem) => format!("RELOADING ({rem:.1}s)"),
        None => format!("{} / {}", active.ammo, active.spec.mag_size),
    };

    text.0 = format!(
        "HP: {:.0} / {:.0}   Stamina: {:.0}%\n[{}] {}   Ammo: {}\nKills: {}   Time: {:.0}s\nWASD Move | Shift Sprint | LMB Shoot | R Reload | 1/2 Swap",
        health.current,
        health.max,
        (stamina.current / stamina.max) * 100.0,
        inventory.active_slot + 1,
        active.spec.name,
        ammo_line,
        stats.kills,
        stats.elapsed_secs,
    );
}

/// Spawns the Game Over overlay with summary stats and a Restart button.
pub fn spawn_game_over_ui(mut commands: Commands, stats: Res<RunStats>) {
    commands
        .spawn((
            Name::new("GameOverUiRoot"),
            GameOverUiRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.06, 0.78)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: FontSize::Px(52.0),
                    ..default()
                },
                TextColor(Color::srgb(0.95, 0.25, 0.22)),
            ));

            parent.spawn((
                Text::new(format!(
                    "Kills: {}   |   Survived: {:.1}s",
                    stats.kills, stats.elapsed_secs
                )),
                TextFont {
                    font_size: FontSize::Px(22.0),
                    ..default()
                },
                TextColor(Color::srgb(0.90, 0.92, 0.96)),
            ));

            parent
                .spawn((
                    RestartButton,
                    Node {
                        padding: UiRect::axes(Val::Px(28.0), Val::Px(14.0)),
                        border: UiRect::all(Val::Px(2.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(BTN_IDLE_COLOR),
                    BorderColor::all(Color::srgb(0.65, 0.82, 1.0)),
                ))
                .observe(
                    |_click: On<PointerClick>, mut next_state: ResMut<NextState<GameState>>| {
                        next_state.set(GameState::Playing);
                    },
                )
                .observe(
                    |over: On<PointerOver>,
                     mut bgs: Query<&mut BackgroundColor, With<RestartButton>>| {
                        if let Ok(mut bg) = bgs.get_mut(over.entity) {
                            bg.0 = BTN_HOVER_COLOR;
                        }
                    },
                )
                .observe(
                    |out: On<PointerOut>,
                     mut bgs: Query<&mut BackgroundColor, With<RestartButton>>| {
                        if let Ok(mut bg) = bgs.get_mut(out.entity) {
                            bg.0 = BTN_IDLE_COLOR;
                        }
                    },
                )
                .with_children(|btn| {
                    btn.spawn((
                        Pickable::IGNORE,
                        Text::new("Restart (Enter)"),
                        TextFont {
                            font_size: FontSize::Px(24.0),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

pub fn despawn_game_over_ui(mut commands: Commands, roots: Query<Entity, With<GameOverUiRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

/// Allows restarting from Game Over via Enter/Space as a keyboard shortcut.
pub fn handle_restart_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next_state.set(GameState::Playing);
    }
}
