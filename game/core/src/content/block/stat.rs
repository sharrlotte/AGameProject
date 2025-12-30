use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Reflect, PartialEq)]
pub enum StackRule {
    Additive,
    Multiply,
    Max,
    Min,
    Replace,
}

impl StackRule {
    pub fn apply(&self, current: f32, _base: f32, value: f32) -> f32 {
        match self {
            StackRule::Additive => current + value,
            StackRule::Multiply => current * value,
            StackRule::Max => current.max(value),
            StackRule::Min => current.min(value),
            StackRule::Replace => value,
        }
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub struct StatEffect {
    pub value: f32,
    pub rule: StackRule,
}
