use super::*;

use crate::content::ship::ai::{preset, validate_profile};
use crate::scene::widget::ButtonVisualState;

#[derive(Component)]
pub struct AiPresetButton(pub &'static str);

pub fn on_ai_preset(
    click: On<PointerClick>,
    buttons: Query<&AiPresetButton>,
    mut editor: ResMut<EditorState>,
) {
    let Ok(button) = buttons.get(click.event().entity) else { return; };
    let profile = preset(button.0);
    if validate_profile(&profile).is_empty() {
        editor.ai_profile = profile;
    }
}

pub fn sync_ai_preset(
    editor: Res<EditorState>,
    mut buttons: Query<(&AiPresetButton, &mut ButtonVisualState)>,
) {
    for (button, mut state) in &mut buttons {
        *state = if preset(button.0) == editor.ai_profile {
            ButtonVisualState::Selected
        } else {
            ButtonVisualState::Normal
        };
    }
}

pub fn update_ai_preview(
    editor: Res<EditorState>,
    mut texts: Query<&mut Text, With<AiPreviewText>>,
) {
    if editor.active_tab != EditorTab::Ai {
        return;
    }
    let Ok(mut text) = texts.single_mut() else {
        return;
    };
    let Ok(ron) = to_string_pretty(&editor.ai_profile, PrettyConfig::default()) else {
         return;
    };
    text.0 = ron;
}
