use bevy::prelude::*;

use crate::ui::state::SceneState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Modal {
    Pause,
    Settings,
    SlotPicker,
}

#[derive(Resource, Default, Debug)]
pub struct ModalStack(pub Vec<Modal>);

impl ModalStack {
    pub fn top(&self) -> Option<Modal> {
        self.0.last().copied()
    }

    pub fn contains(&self, modal: Modal) -> bool {
        self.0.contains(&modal)
    }

    pub fn push(&mut self, modal: Modal) {
        if self.top() != Some(modal) {
            self.0.push(modal);
        }
    }

    pub fn pop(&mut self) -> Option<Modal> {
        self.0.pop()
    }

    pub fn remove(&mut self, modal: Modal) {
        self.0.retain(|current| *current != modal);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

/// Marker for a UI root owned by a modal, used for despawn-on-close.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModalRoot(pub Modal);

pub fn modal_is(modal: Modal) -> impl Fn(Res<ModalStack>) -> bool + Clone {
    move |stack: Res<ModalStack>| stack.top() == Some(modal)
}

pub fn clear_modals_on_scene_exit(
    mut stack: ResMut<ModalStack>,
    mut time: ResMut<Time<Virtual>>,
) {
    stack.clear();
    time.unpause();
}

pub fn sync_modal_time(
    stack: Res<ModalStack>,
    scene: Res<State<SceneState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    let gameplay = matches!(scene.get(), SceneState::Game | SceneState::ShipEditor);
    if gameplay && stack.top() == Some(Modal::Pause) {
        time.pause();
    } else {
        time.unpause();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_pops_settings_before_pause() {
        let mut stack = ModalStack::default();
        stack.push(Modal::Pause);
        stack.push(Modal::Settings);
        assert_eq!(stack.pop(), Some(Modal::Settings));
        assert_eq!(stack.top(), Some(Modal::Pause));
        assert_eq!(stack.pop(), Some(Modal::Pause));
        assert_eq!(stack.top(), None);
    }

    #[test]
    fn pushing_same_modal_is_idempotent() {
        let mut stack = ModalStack::default();
        stack.push(Modal::SlotPicker);
        stack.push(Modal::SlotPicker);
        assert_eq!(stack.0.len(), 1);
    }

    #[test]
    fn clear_removes_all_modals() {
        let mut stack = ModalStack::default();
        stack.push(Modal::Pause);
        stack.push(Modal::Settings);
        stack.clear();
        assert_eq!(stack.top(), None);
    }
}
