use super::*;

pub fn handle_ai_preset_click(
    mut interaction: Query<(&PickingInteraction, &AiPresetButton), Changed<PickingInteraction>>,
    mut editor: ResMut<EditorState>,
) {
    use crate::content::ship::ai::{preset, validate_profile};
    for (interaction, button) in &mut interaction {
        if *interaction == PickingInteraction::Pressed {
            let profile = preset(button.0);
            let errors = validate_profile(&profile);
            if errors.is_empty() {
                editor.ai_profile = profile;
            } else {
                warn!("AI preset {:?} is invalid: {:?}", button.0, errors);
            }
        }
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
