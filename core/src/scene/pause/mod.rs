use bevy::prelude::*;

use crate::resource::SceneState;
use crate::save::world::{EditorReturnTarget, PendingExitAction, SaveWorldEvent, WorldContext};
use crate::ui::{Modal, ModalRoot, ModalStack};
use crate::scene::layout::{BORDER_WIDTH, BUTTON_HEIGHT, BUTTON_WIDTH, CORNER_RADIUS, FONT_TITLE};
use crate::scene::widget::{ButtonVisuals, spawn_button};
use crate::simulation::SimulationSet;

pub struct PauseScenePlugin;

impl Plugin for PauseScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                Update,
                (
                    handle_modal_input.in_set(SimulationSet::Input),
                    sync_pause_ui.in_set(SimulationSet::Present),
                ),
            );
    }
}

#[derive(Component)]
struct ResumeButton;

#[derive(Component)]
struct PauseSettingsButton;

#[derive(Component)]
struct PauseShipEditorButton;

#[derive(Component)]
struct HomeButton;

pub fn handle_modal_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut stack: ResMut<ModalStack>,
    scene: Res<State<SceneState>>,
) {
    if !keys.just_pressed(KeyCode::Escape) {
        return;
    }
    if stack.top().is_some() {
        stack.pop();
    } else if matches!(scene.get(), SceneState::Game | SceneState::ShipEditor) {
        stack.push(Modal::Pause);
    }
}

fn sync_pause_ui(
    mut commands: Commands,
    stack: Res<ModalStack>,
    roots: Query<(Entity, &ModalRoot)>,
    session: Res<WorldContext>,
) {
    let open = stack.contains(Modal::Pause);
    let existing = roots.iter().find(|(_, root)| root.0 == Modal::Pause).map(|(e, _)| e);

    match (open, existing) {
        (true, None) => setup_pause(&mut commands, &session),
        (false, Some(entity)) => commands.entity(entity).despawn(),
        _ => {}
    }
}

fn on_resume(
    click: On<PointerClick>,
    buttons: Query<&ResumeButton>,
    mut stack: ResMut<ModalStack>,
) {
    if buttons.contains(click.event().entity) {
        stack.remove(Modal::Pause);
    }
}

fn on_settings(
    click: On<PointerClick>,
    buttons: Query<&PauseSettingsButton>,
    mut stack: ResMut<ModalStack>,
) {
    if buttons.contains(click.event().entity) {
        stack.push(Modal::Settings);
    }
}

fn on_ship_editor(
    click: On<PointerClick>,
    buttons: Query<&PauseShipEditorButton>,
    session: Res<WorldContext>,
    mut target: ResMut<EditorReturnTarget>,
    mut stack: ResMut<ModalStack>,
    mut scene: ResMut<NextState<SceneState>>,
    mut saves: MessageWriter<SaveWorldEvent>,
) {
    if buttons.contains(click.event().entity) {
        if session.is_save_slot() {
            saves.write(SaveWorldEvent);
        }
        *target = EditorReturnTarget::Game;
        stack.clear();
        scene.set(SceneState::ShipEditor);
    }
}

fn on_home(
    click: On<PointerClick>,
    buttons: Query<&HomeButton>,
    scene_state: Res<State<SceneState>>,
    session: Res<WorldContext>,
    mut stack: ResMut<ModalStack>,
    mut scene: ResMut<NextState<SceneState>>,
    mut saves: MessageWriter<SaveWorldEvent>,
    mut pending_exit: ResMut<PendingExitAction>,
) {
    if !buttons.contains(click.event().entity) {
        return;
    }
    stack.clear();
    if *scene_state.get() == SceneState::Game
        && session.is_save_slot()
    {
        saves.write(SaveWorldEvent);
        *pending_exit = PendingExitAction::Home;
    } else {
        scene.set(SceneState::Home);
    }
}

fn setup_pause(commands: &mut Commands, session: &WorldContext) {
    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 100.0),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(FONT_TITLE),
                ..default()
            },
            ModalRoot(Modal::Pause),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Paused"),
                TextFont {
                    font_size: FontSize::Px(40.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            spawn_button(
                parent,
                button_node(),
                "Resume",
                20.0,
                Color::WHITE,
                ResumeButton,
                on_resume,
                ButtonVisuals::dark(),
            );
            spawn_button(
                parent,
                button_node(),
                "Settings",
                20.0,
                Color::WHITE,
                PauseSettingsButton,
                on_settings,
                ButtonVisuals::dark(),
            );
            if session.is_save_slot() {
                spawn_button(
                    parent,
                    button_node(),
                    "Ship Editor",
                    20.0,
                    Color::WHITE,
                    PauseShipEditorButton,
                    on_ship_editor,
                    ButtonVisuals::blue(),
                );
            }
            spawn_button(
                parent,
                button_node(),
                "Back to Home",
                20.0,
                Color::WHITE,
                HomeButton,
                on_home,
                ButtonVisuals::dark(),
            );
        });
}

fn button_node() -> Node {
    Node {
        width: Val::Px(BUTTON_WIDTH),
        height: Val::Px(BUTTON_HEIGHT),
        border: UiRect::all(Val::Px(BORDER_WIDTH)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        padding: UiRect::all(Val::Px(CORNER_RADIUS)),
        border_radius: BorderRadius::all(Val::Px(CORNER_RADIUS)),
        ..default()
    }
}
