use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Reflect, PartialEq, SystemSet, Hash, Eq)]
pub struct StatsUpdateSet;

macro_rules! make_stat_system {
    ($name:ident, $base:ty, $computed:ty, $effects:ty, $op:expr) => {
        pub fn $name(
            mut query: Query<(
                &$base,
                &mut $computed,
                Option<&$effects>,
                Option<&EfficiencyEffect>,
                Option<&BlockEfficiency>,
            )>,
        ) {
            for (base, mut computed, effects, efficiency_effect, block_efficiency) in &mut query {
                let eff = block_efficiency
                    .map(|e| e.value())
                    .or_else(|| efficiency_effect.map(|e| e.value()))
                    .unwrap_or(1.0);

                // A starved feature is disabled rather than running at a
                // negligible fraction of its base value.
                let mut val = if eff <= 0.01 {
                    0.0
                } else {
                    ($op)(base.value(), eff)
                };

                if let Some(effects) = effects {
                    for effect in &effects.0 {
                        val = effect.rule.apply(val, base.value(), effect.value);
                    }
                }
                computed.set(val);
            }
        }
    };
}

pub(crate) use make_stat_system;

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
