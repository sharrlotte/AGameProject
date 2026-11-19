use super::*;

use crate::content::ship::ai::Doctrine;
use crate::scene::widget::ButtonVisualState;

#[derive(Component)]
pub struct AiPresetButton(pub Doctrine);

pub fn on_doctrine(
    click: On<PointerClick>,
    buttons: Query<&AiPresetButton>,
    mut editor: ResMut<EditorState>,
    mut save_events: MessageWriter<ShipSaveChanged>,
) {
    let Ok(button) = buttons.get(click.event().entity) else {
        return;
    };
    editor.doctrine = button.0;
    save_events.write(ShipSaveChanged);
}

pub fn sync_doctrine(
    editor: Res<EditorState>,
    mut buttons: Query<(&AiPresetButton, &mut ButtonVisualState)>,
) {
    for (button, mut state) in &mut buttons {
        *state = if button.0 == editor.doctrine {
            ButtonVisualState::Selected
        } else {
            ButtonVisualState::Normal
        };
    }
}
