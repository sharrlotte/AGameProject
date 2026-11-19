use bevy::ecs::observer::IntoEntityObserver;
use bevy::prelude::*;

use crate::simulation::SimulationSet;

pub mod scroll;

use scroll::ScrollPlugin;

#[derive(Component, Clone, Copy)]
pub struct ButtonVisuals {
    pub base: Color,
    pub hover: Color,
    pub press: Color,
    pub selected: Color,
    pub disabled: Color,
}

impl ButtonVisuals {
    pub fn dark() -> Self {
        Self {
            base: Color::srgb(0.15, 0.15, 0.15),
            hover: Color::srgb(0.25, 0.25, 0.25),
            press: Color::srgb(0.35, 0.75, 0.35),
            selected: Color::srgb(0.30, 0.30, 0.30),
            disabled: Color::srgb(0.10, 0.10, 0.10),
        }
    }

    pub fn blue() -> Self {
        Self {
            base: Color::srgb(0.20, 0.35, 0.55),
            hover: Color::srgb(0.30, 0.50, 0.75),
            press: Color::srgb(0.35, 0.75, 0.95),
            selected: Color::srgb(0.35, 0.55, 0.80),
            disabled: Color::srgb(0.12, 0.20, 0.32),
        }
    }

    pub fn green() -> Self {
        Self {
            base: Color::srgb(0.25, 0.50, 0.25),
            hover: Color::srgb(0.35, 0.65, 0.35),
            press: Color::srgb(0.35, 0.85, 0.35),
            selected: Color::srgb(0.40, 0.70, 0.40),
            disabled: Color::srgb(0.12, 0.25, 0.12),
        }
    }

    pub fn red() -> Self {
        Self {
            base: Color::srgb(0.50, 0.20, 0.20),
            hover: Color::srgb(0.65, 0.28, 0.28),
            press: Color::srgb(0.85, 0.35, 0.35),
            selected: Color::srgb(0.70, 0.30, 0.30),
            disabled: Color::srgb(0.25, 0.10, 0.10),
        }
    }
}

#[derive(Component, Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVisualState {
    #[default]
    Normal,
    Selected,
    Disabled,
}

#[derive(Component, Default, Clone, Copy)]
struct ButtonInteraction {
    hovered: bool,
    pressed: bool,
}

pub struct WidgetPlugin;

impl Plugin for WidgetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScrollPlugin)
            .add_systems(Update, sync_button_colors.in_set(SimulationSet::Present));
    }
}

pub fn spawn_button<A: Component, M>(
    parent: &mut ChildSpawnerCommands,
    node: Node,
    label: impl Into<String>,
    font_size: f32,
    text_color: Color,
    action: A,
    on_click: impl IntoEntityObserver<M>,
    visuals: ButtonVisuals,
) -> Entity {
    let mut entity = parent.spawn(button_bundle(node, action, visuals));
    attach_observers(&mut entity, on_click);
    entity.with_children(|parent| {
        parent.spawn((
            Text::new(label),
            TextFont {
                font_size: FontSize::Px(font_size),
                ..default()
            },
            TextColor(text_color),
        ));
    });
    entity.id()
}

pub fn spawn_widget<A: Component, M>(
    parent: &mut ChildSpawnerCommands,
    node: Node,
    action: A,
    on_click: impl IntoEntityObserver<M>,
    visuals: ButtonVisuals,
) -> Entity {
    let mut entity = parent.spawn(button_bundle(node, action, visuals));
    attach_observers(&mut entity, on_click);
    entity.id()
}

fn button_bundle<A: Component>(
    node: Node,
    action: A,
    visuals: ButtonVisuals,
) -> impl Bundle {
    (
        node,
        BackgroundColor(visuals.base),
        action,
        visuals,
        ButtonVisualState::default(),
        ButtonInteraction::default(),
    )
}

fn attach_observers<M>(
    entity: &mut EntityCommands,
    on_click: impl IntoEntityObserver<M>,
) {
    entity
        .observe(on_click)
        .observe(handle_button_over)
        .observe(handle_button_out)
        .observe(handle_button_press)
        .observe(handle_button_release);
}

fn handle_button_over(over: On<PointerOver>, mut interactions: Query<&mut ButtonInteraction>) {
    if let Ok(mut interaction) = interactions.get_mut(over.event().entity) {
        interaction.hovered = true;
    }
}

fn handle_button_out(out: On<PointerOut>, mut interactions: Query<&mut ButtonInteraction>) {
    if let Ok(mut interaction) = interactions.get_mut(out.event().entity) {
        interaction.hovered = false;
        interaction.pressed = false;
    }
}

fn handle_button_press(press: On<PointerPress>, mut interactions: Query<&mut ButtonInteraction>) {
    if let Ok(mut interaction) = interactions.get_mut(press.event().entity) {
        interaction.pressed = true;
    }
}

fn handle_button_release(
    release: On<PointerRelease>,
    mut interactions: Query<&mut ButtonInteraction>,
) {
    if let Ok(mut interaction) = interactions.get_mut(release.event().entity) {
        interaction.pressed = false;
    }
}

fn sync_button_colors(
    mut buttons: Query<
        (
            &ButtonVisuals,
            &ButtonVisualState,
            &ButtonInteraction,
            &mut BackgroundColor,
        ),
        Or<(Changed<ButtonVisualState>, Changed<ButtonInteraction>)>,
    >,
) {
    for (visuals, state, interaction, mut background) in &mut buttons {
        let color = match state {
            ButtonVisualState::Disabled => visuals.disabled,
            ButtonVisualState::Selected if interaction.pressed => visuals.press,
            ButtonVisualState::Selected => visuals.selected,
            ButtonVisualState::Normal if interaction.pressed => visuals.press,
            ButtonVisualState::Normal if interaction.hovered => visuals.hover,
            ButtonVisualState::Normal => visuals.base,
        };
        if background.0 != color {
            *background = BackgroundColor(color);
        }
    }
}
