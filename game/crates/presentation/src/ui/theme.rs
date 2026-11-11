use bevy::prelude::*;

pub const BTN_IDLE_COLOR: Color = Color::srgb(0.18, 0.45, 0.78);
pub const BTN_HOVER_COLOR: Color = Color::srgb(0.25, 0.56, 0.90);
pub const BTN_SECONDARY_IDLE: Color = Color::srgb(0.22, 0.30, 0.40);
pub const BTN_SECONDARY_HOVER: Color = Color::srgb(0.30, 0.40, 0.52);

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct MenuButtonColors {
    pub idle: Color,
    pub hover: Color,
}

pub fn spawn_progress_bar(
    parent: &mut ChildSpawnerCommands,
    width_px: f32,
    height_px: f32,
    bg_color: Color,
    fill_color: Color,
    fill_marker: impl Bundle,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(width_px),
                height: Val::Px(height_px),
                ..default()
            },
            BackgroundColor(bg_color),
        ))
        .with_children(|bar| {
            bar.spawn((
                fill_marker,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(fill_color),
            ));
        });
}

pub fn spawn_action_button<E: Message + Clone>(
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
        .observe(
            move |_click: On<PointerClick>, mut writer: MessageWriter<E>| {
                writer.write(message.clone());
            },
        )
        .observe(
            |over: On<PointerOver>,
             mut buttons: Query<(&MenuButtonColors, &mut BackgroundColor)>| {
                if let Ok((colors, mut bg)) = buttons.get_mut(over.entity) {
                    bg.0 = colors.hover;
                }
            },
        )
        .observe(
            |out: On<PointerOut>, mut buttons: Query<(&MenuButtonColors, &mut BackgroundColor)>| {
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

pub fn spawn_overlay_root<'a>(
    commands: &'a mut Commands,
    name: &'static str,
) -> EntityCommands<'a> {
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
