mod slots;

use bevy::prelude::*;

use crate::resource::{SceneState, SettingState, SlotPickerState};
use crate::save::world::{EditorReturnTarget, StartQuickBattleEvent};
use crate::scene::home::slots::{
    SlotPickerUi, handle_name_input, refresh_slot_list, setup_picker, slot_picker_input,
};
use crate::scene::layout::{BORDER_WIDTH, BUTTON_HEIGHT, BUTTON_WIDTH, CORNER_RADIUS, FONT_TITLE};
use crate::scene::widget::{ButtonVisuals, spawn_button};
use crate::simulation::SimulationSet;

pub struct HomeScenePlugin;

impl Plugin for HomeScenePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<SlotPickerState>()
            .init_resource::<SlotPickerUi>()
            .add_systems(OnEnter(SceneState::Home), setup_home)
            .add_systems(
                OnEnter(SlotPickerState::Open),
                (refresh_slot_list, setup_picker).chain(),
            )
            .add_systems(
                Update,
                (
                    slot_picker_input.in_set(SimulationSet::Input),
                    handle_name_input
                        .run_if(in_state(SlotPickerState::Open))
                        .in_set(SimulationSet::Input),
                ),
            );
    }
}

#[derive(Component)]
struct PlayButton;

#[derive(Component)]
struct QuickBattleButton;

#[derive(Component)]
struct ShipEditorButton;

#[derive(Component)]
struct HomeSettingsButton;

#[derive(Component)]
struct ExitButton;

fn on_play(
    click: On<PointerClick>,
    buttons: Query<&PlayButton>,
    mut picker: ResMut<NextState<SlotPickerState>>,
) {
    if buttons.contains(click.event().entity) {
        picker.set(SlotPickerState::Open);
    }
}

fn on_quick_battle(
    click: On<PointerClick>,
    buttons: Query<&QuickBattleButton>,
    mut starts: MessageWriter<StartQuickBattleEvent>,
) {
    if buttons.contains(click.event().entity) {
        starts.write(StartQuickBattleEvent);
    }
}

fn on_ship_editor(
    click: On<PointerClick>,
    buttons: Query<&ShipEditorButton>,
    mut target: ResMut<EditorReturnTarget>,
    mut scene: ResMut<NextState<SceneState>>,
) {
    if buttons.contains(click.event().entity) {
        *target = EditorReturnTarget::Home;
        scene.set(SceneState::ShipEditor);
    }
}

fn on_settings(
    click: On<PointerClick>,
    buttons: Query<&HomeSettingsButton>,
    mut setting: ResMut<NextState<SettingState>>,
) {
    if buttons.contains(click.event().entity) {
        setting.set(SettingState::Open);
    }
}

fn on_exit(
    click: On<PointerClick>,
    buttons: Query<&ExitButton>,
    mut exit: MessageWriter<AppExit>,
) {
    if buttons.contains(click.event().entity) {
        exit.write(AppExit::Success);
    }
}

fn setup_home(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(FONT_TITLE),
                ..default()
            },
            DespawnOnExit(SceneState::Home),
        ))
        .with_children(|parent| {
            spawn_button(
                parent,
                button_node(),
                "Play",
                20.0,
                Color::WHITE,
                PlayButton,
                on_play,
                ButtonVisuals::dark(),
            );
            spawn_button(
                parent,
                button_node(),
                "Quick Battle",
                20.0,
                Color::WHITE,
                QuickBattleButton,
                on_quick_battle,
                ButtonVisuals::blue(),
            );
            spawn_button(
                parent,
                button_node(),
                "Ship Editor",
                20.0,
                Color::WHITE,
                ShipEditorButton,
                on_ship_editor,
                ButtonVisuals::dark(),
            );
            spawn_button(
                parent,
                button_node(),
                "Settings",
                20.0,
                Color::WHITE,
                HomeSettingsButton,
                on_settings,
                ButtonVisuals::dark(),
            );
            spawn_button(
                parent,
                button_node(),
                "Exit",
                20.0,
                Color::WHITE,
                ExitButton,
                on_exit,
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
