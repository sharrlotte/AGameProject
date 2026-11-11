use bevy::prelude::*;
use data::{
    DeployRequestedMessage, GameState, LastRunOutcome, OpenUpgradesRequestedMessage,
    PurchaseUpgradeMessage, ScrapBank, UpgradeKind, UpgradeLevels, UpgradeModifiers,
    UpgradesChangedMessage,
};
use extraction::{UpgradeLevelsExt, UpgradesConfig};

const BTN_IDLE_COLOR: Color = Color::srgb(0.18, 0.45, 0.78);
const BTN_HOVER_COLOR: Color = Color::srgb(0.25, 0.56, 0.90);
const BTN_SECONDARY_IDLE: Color = Color::srgb(0.22, 0.30, 0.40);
const BTN_SECONDARY_HOVER: Color = Color::srgb(0.30, 0.40, 0.52);

#[derive(Component)]
struct MenuRoot;

#[derive(Component)]
struct MenuButtonColors {
    idle: Color,
    hover: Color,
}

#[derive(Component)]
struct UpgradeBankText;

#[derive(Component)]
struct UpgradeRowText(UpgradeKind);

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), spawn_loading_ui)
            .add_systems(OnExit(GameState::Loading), despawn_menu_ui)
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over_ui)
            .add_systems(OnExit(GameState::GameOver), despawn_menu_ui)
            .add_systems(OnEnter(GameState::Upgrades), spawn_upgrades_ui)
            .add_systems(OnExit(GameState::Upgrades), despawn_menu_ui)
            .add_systems(
                Update,
                handle_deploy_shortcut.run_if(|state: Res<State<GameState>>| {
                    matches!(state.get(), GameState::GameOver | GameState::Upgrades)
                }),
            )
            .add_systems(
                Update,
                refresh_upgrades_ui.run_if(in_state(GameState::Upgrades)),
            );
    }
}

fn spawn_action_button<E: Message + Clone>(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    idle: Color,
    hover: Color,
    message: E,
) {
    parent
        .spawn((
            MenuButtonColors { idle, hover },
            Node {
                padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(idle),
            BorderColor::all(Color::srgb(0.65, 0.82, 1.0)),
        ))
        .observe(move |_click: On<PointerClick>, mut writer: MessageWriter<E>| {
            writer.write(message.clone());
        })
        .observe(
            |over: On<PointerOver>,
             mut buttons: Query<(&MenuButtonColors, &mut BackgroundColor)>| {
                if let Ok((colors, mut bg)) = buttons.get_mut(over.entity) {
                    bg.0 = colors.hover;
                }
            },
        )
        .observe(
            |out: On<PointerOut>,
             mut buttons: Query<(&MenuButtonColors, &mut BackgroundColor)>| {
                if let Ok((colors, mut bg)) = buttons.get_mut(out.entity) {
                    bg.0 = colors.idle;
                }
            },
        )
        .with_children(|btn| {
            btn.spawn((
                Pickable::IGNORE,
                Text::new(label),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn spawn_overlay_root<'a>(commands: &'a mut Commands, name: &'static str) -> EntityCommands<'a> {
    commands.spawn((
        Name::new(name),
        MenuRoot,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(16.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.84)),
    ))
}

fn spawn_loading_ui(mut commands: Commands) {
    spawn_overlay_root(&mut commands, "LoadingUiRoot").with_children(|parent| {
        parent.spawn((
            Text::new("Loading .ron configurations..."),
            TextFont {
                font_size: FontSize::Px(26.0),
                ..default()
            },
            TextColor(Color::srgb(0.88, 0.92, 0.98)),
        ));
    });
}

fn spawn_game_over_ui(
    mut commands: Commands,
    outcome: Res<LastRunOutcome>,
    bank: Res<ScrapBank>,
) {
    let summary = match *outcome {
        LastRunOutcome::Died {
            scrap_lost,
            kills,
            elapsed_secs,
        } => format!(
            "Kills: {kills}   |   Survived: {elapsed_secs:.1}s\nLost Un-extracted Scrap: {scrap_lost}   |   Banked Scrap: {}",
            bank.banked
        ),
        _ => format!("Banked Scrap: {}", bank.banked),
    };

    spawn_overlay_root(&mut commands, "GameOverUiRoot").with_children(|parent| {
        parent.spawn((
            Text::new("YOU DIED"),
            TextFont {
                font_size: FontSize::Px(52.0),
                ..default()
            },
            TextColor(Color::srgb(0.95, 0.25, 0.22)),
        ));

        parent.spawn((
            Text::new(summary),
            TextFont {
                font_size: FontSize::Px(20.0),
                ..default()
            },
            TextColor(Color::srgb(0.90, 0.92, 0.96)),
        ));

        parent
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|row| {
                spawn_action_button(
                    row,
                    "Upgrade Shop",
                    BTN_SECONDARY_IDLE,
                    BTN_SECONDARY_HOVER,
                    OpenUpgradesRequestedMessage,
                );
                spawn_action_button(
                    row,
                    "Deploy Again (Enter)",
                    BTN_IDLE_COLOR,
                    BTN_HOVER_COLOR,
                    DeployRequestedMessage,
                );
            });
    });
}

fn spawn_upgrades_ui(
    mut commands: Commands,
    outcome: Res<LastRunOutcome>,
    bank: Res<ScrapBank>,
    levels: Res<UpgradeLevels>,
    modifiers: Res<UpgradeModifiers>,
    cfg: Res<UpgradesConfig>,
) {
    let (title, title_color, subtitle) = match *outcome {
        LastRunOutcome::Extracted {
            scrap_banked,
            kills,
            elapsed_secs,
        } => (
            "EXTRACTION SUCCESSFUL!",
            Color::srgb(0.28, 0.92, 0.56),
            format!("Banked +{scrap_banked} Scrap   |   Kills: {kills}   |   Time: {elapsed_secs:.1}s"),
        ),
        _ => (
            "UPGRADE WORKSHOP",
            Color::srgb(0.45, 0.80, 0.98),
            "Spend banked Scrap to power up before deploying.".to_string(),
        ),
    };

    spawn_overlay_root(&mut commands, "UpgradesUiRoot").with_children(|parent| {
        parent.spawn((
            Text::new(title),
            TextFont {
                font_size: FontSize::Px(42.0),
                ..default()
            },
            TextColor(title_color),
        ));

        parent.spawn((
            Text::new(subtitle),
            TextFont {
                font_size: FontSize::Px(18.0),
                ..default()
            },
            TextColor(Color::srgb(0.86, 0.90, 0.95)),
        ));

        parent.spawn((
            UpgradeBankText,
            Text::new(format!("Available Scrap: {}", bank.banked)),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.78, 0.25)),
        ));

        for kind in [
            UpgradeKind::WeaponDamage,
            UpgradeKind::MaxHp,
            UpgradeKind::MoveAndReloadSpeed,
        ] {
            parent
                .spawn((
                    Node {
                        width: Val::Px(580.0),
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.10, 0.13, 0.18, 0.92)),
                ))
                .with_children(|row| {
                    row.spawn((
                        UpgradeRowText(kind),
                        Text::new(format_upgrade_label(kind, &levels, &modifiers, &cfg)),
                        TextFont {
                            font_size: FontSize::Px(18.0),
                            ..default()
                        },
                        TextColor(Color::srgb(0.92, 0.95, 0.98)),
                    ));

                    spawn_action_button(
                        row,
                        "Buy",
                        BTN_SECONDARY_IDLE,
                        BTN_SECONDARY_HOVER,
                        PurchaseUpgradeMessage { kind },
                    );
                });
        }

        spawn_action_button(
            parent,
            "Deploy Again (Enter)",
            BTN_IDLE_COLOR,
            BTN_HOVER_COLOR,
            DeployRequestedMessage,
        );
    });
}

fn format_upgrade_label(
    kind: UpgradeKind,
    levels: &UpgradeLevels,
    modifiers: &UpgradeModifiers,
    cfg: &UpgradesConfig,
) -> String {
    let lvl = levels.level(kind);
    let max_lvl = levels.max_level(kind, cfg);
    let cost_str = if lvl >= max_lvl {
        "MAX".to_string()
    } else {
        format!("Cost: {} Scrap", levels.next_cost(kind, cfg))
    };

    match kind {
        UpgradeKind::WeaponDamage => {
            let bonus_pct = (modifiers.damage_multiplier - 1.0) * 100.0;
            format!(
                "+Weapon Damage   (Lv {lvl}/{max_lvl}, +{bonus_pct:.0}%)   |   {cost_str}"
            )
        }
        UpgradeKind::MaxHp => {
            format!(
                "+Max HP   (Lv {lvl}/{max_lvl}, +{:.0} HP)   |   {cost_str}",
                modifiers.bonus_max_hp
            )
        }
        UpgradeKind::MoveAndReloadSpeed => {
            let bonus_pct = (modifiers.move_speed_multiplier - 1.0) * 100.0;
            format!(
                "+Move & Reload Speed   (Lv {lvl}/{max_lvl}, +{bonus_pct:.0}%)   |   {cost_str}"
            )
        }
    }
}

fn refresh_upgrades_ui(
    mut changed: MessageReader<UpgradesChangedMessage>,
    bank: Res<ScrapBank>,
    levels: Res<UpgradeLevels>,
    modifiers: Res<UpgradeModifiers>,
    cfg: Res<UpgradesConfig>,
    mut bank_texts: Query<&mut Text, (With<UpgradeBankText>, Without<UpgradeRowText>)>,
    mut row_texts: Query<(&UpgradeRowText, &mut Text), Without<UpgradeBankText>>,
) {
    if changed.read().next().is_none() {
        return;
    }

    for mut txt in &mut bank_texts {
        txt.0 = format!("Available Scrap: {}", bank.banked);
    }
    for (row, mut txt) in &mut row_texts {
        txt.0 = format_upgrade_label(row.0, &levels, &modifiers, &cfg);
    }
}

fn despawn_menu_ui(mut commands: Commands, roots: Query<Entity, With<MenuRoot>>) {
    for entity in &roots {
        commands.entity(entity).despawn();
    }
}

fn handle_deploy_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut deploy_writer: MessageWriter<DeployRequestedMessage>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        deploy_writer.write(DeployRequestedMessage);
    }
}
