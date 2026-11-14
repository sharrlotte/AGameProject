use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrainState {
    #[default]
    Idle,
    Alert,
    Engage,
    Stagger,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct EnemyBrain {
    pub state: BrainState,
    pub target: Option<Entity>,
    pub last_known: Option<Vec3>,
    pub alert_timer: f32,
    pub stagger_timer: f32,
    pub unseen_secs: f32,
}

impl Default for EnemyBrain {
    fn default() -> Self {
        Self {
            state: BrainState::Idle,
            target: None,
            last_known: None,
            alert_timer: 0.0,
            stagger_timer: 0.0,
            unseen_secs: 0.0,
        }
    }
}

impl EnemyBrain {
    pub fn engage(&mut self, target: Entity, at: Vec3) {
        self.target = Some(target);
        self.last_known = Some(at);
        self.unseen_secs = 0.0;
        if self.state != BrainState::Stagger {
            self.state = BrainState::Engage;
        }
    }

    pub fn stagger(&mut self, secs: f32) {
        self.state = BrainState::Stagger;
        self.stagger_timer = secs.max(0.0);
    }
}
