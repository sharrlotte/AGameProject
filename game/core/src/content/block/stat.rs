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

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct EfficiencyEffect {
    pub value: f32,
}

/// Computed efficiency of the block.
/// Final value = Ship Power Efficiency * max(1.0, Efficiency Buffs)
#[derive(Component, Clone, Copy, Debug, Reflect)]
pub struct BlockEfficiency {
    pub value: f32,
    /// Base efficiency from ship power status (0.0-1.0)
    pub base: f32,
}

impl Default for BlockEfficiency {
    fn default() -> Self {
        Self {
            value: 1.0,
            base: 1.0,
        }
    }
}

/// Component that provides efficiency buff to nearby blocks.
#[derive(Component, Clone, Copy, Debug, Reflect, Default, PartialEq)]
pub struct EfficiencyProvider {
    pub radius: f32,
    pub value: f32,
}

// PowerUsage
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerUsageBase {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct PowerUsageComputed {
    pub value: f32,
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct PowerUsageEffects(pub Vec<StatEffect>);

// ThrustPower
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ThrustPowerBase {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct ThrustPowerComputed {
    pub value: f32,
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct ThrustPowerEffects(pub Vec<StatEffect>);

// RotateSpeed
#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct RotateSpeedBase {
    pub value: f32,
}

#[derive(Component, Clone, Copy, Debug, Reflect, Default)]
pub struct RotateSpeedComputed {
    pub value: f32,
}

#[derive(Component, Clone, Debug, Reflect, Default)]
pub struct RotateSpeedEffects(pub Vec<StatEffect>);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockStatUpdateSet;

pub fn update_power_usage_stats(
    mut query: Query<(
        &PowerUsageBase,
        &mut PowerUsageComputed,
        Option<&PowerUsageEffects>,
        Option<&EfficiencyEffect>,
    )>,
) {
    for (base, mut computed, effects, efficiency_effect) in &mut query {
        let eff = efficiency_effect.map(|e| e.value).unwrap_or(1.0);

        // Efficiency decreases power usage.
        let mut val = if eff > 0.0001 {
            base.value / eff
        } else {
            base.value
        };

        if let Some(effects) = effects {
            for effect in &effects.0 {
                val = effect.rule.apply(val, base.value, effect.value);
            }
        }
        computed.value = val;
    }
}

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                update_power_usage_stats,
                update_thrust_power_stats,
                update_rotate_speed_stats,
            )
                .chain()
                .in_set(BlockStatUpdateSet),
        );
    }
}

pub fn update_thrust_power_stats(
    mut query: Query<(
        &ThrustPowerBase,
        &mut ThrustPowerComputed,
        Option<&ThrustPowerEffects>,
        Option<&EfficiencyEffect>,
        Option<&BlockEfficiency>,
    )>,
) {
    for (base, mut computed, effects, efficiency_effect, block_efficiency) in &mut query {
        let eff = if let Some(be) = block_efficiency {
            be.value
        } else {
            efficiency_effect.map(|e| e.value).unwrap_or(1.0)
        };

        // Efficiency increases thrust.
        let mut val = base.value * eff;

        if let Some(effects) = effects {
            for effect in &effects.0 {
                val = effect.rule.apply(val, base.value, effect.value);
            }
        }
        computed.value = val;
    }
}

pub fn update_rotate_speed_stats(
    mut query: Query<(
        &RotateSpeedBase,
        &mut RotateSpeedComputed,
        Option<&RotateSpeedEffects>,
        Option<&EfficiencyEffect>,
        Option<&BlockEfficiency>,
    )>,
) {
    for (base, mut computed, effects, efficiency_effect, block_efficiency) in &mut query {
        let eff = if let Some(be) = block_efficiency {
            be.value
        } else {
            efficiency_effect.map(|e| e.value).unwrap_or(1.0)
        };

        // Efficiency increases rotate speed.
        let mut val = base.value * eff;

        if let Some(effects) = effects {
            for effect in &effects.0 {
                val = effect.rule.apply(val, base.value, effect.value);
            }
        }
        computed.value = val;
    }
}
