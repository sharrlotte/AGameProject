use bevy::prelude::*;

/// Timer shared by all AI behavior systems. Each behavior system ticks it;
/// a ship carries exactly one behavior marker, so only one system acts per tick.
#[derive(Component, Debug, Clone)]
pub struct BehaviorTimer(pub Timer);

impl BehaviorTimer {
    pub fn new(think_rate_hz: f32) -> Self {
        Self(Timer::from_seconds(
            1.0 / think_rate_hz.max(f32::EPSILON),
            TimerMode::Repeating,
        ))
    }
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorIdle;

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorWander;

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorKiting;

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorRegroup;

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorChase {
    pub target: Entity,
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorAttack {
    pub target_position: Vec2,
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorPatrol {
    pub around_position: Vec2,
    pub radius: f32,
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorFlee {
    pub from: Entity,
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorGuard {
    pub target: Entity,
    pub radius: f32,
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorFollow {
    pub target: Entity,
    pub radius: f32,
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct BehaviorKamikaze {
    pub target: Entity,
}

/// Remove every behavior marker and insert the new one in one deferred command.
macro_rules! set_behavior {
    ($commands:expr, $entity:expr, $new:expr) => {{
        let mut ec = $commands.entity($entity);
        ec.remove::<(
            BehaviorIdle,
            BehaviorWander,
            BehaviorKiting,
            BehaviorRegroup,
            BehaviorChase,
            BehaviorAttack,
            BehaviorPatrol,
            BehaviorFlee,
            BehaviorGuard,
            BehaviorFollow,
            BehaviorKamikaze,
        )>();
        ec.insert($new);
    }};
}

pub(crate) use set_behavior;

/// Human-readable label for the single behavior marker on a ship (HUD/debug).
pub fn behavior_label(
    idle: Option<&BehaviorIdle>,
    wander: Option<&BehaviorWander>,
    kiting: Option<&BehaviorKiting>,
    regroup: Option<&BehaviorRegroup>,
    chase: Option<&BehaviorChase>,
    attack: Option<&BehaviorAttack>,
    patrol: Option<&BehaviorPatrol>,
    flee: Option<&BehaviorFlee>,
    guard: Option<&BehaviorGuard>,
    follow: Option<&BehaviorFollow>,
    kamikaze: Option<&BehaviorKamikaze>,
) -> &'static str {
    if idle.is_some() {
        "Idle"
    } else if wander.is_some() {
        "Wander"
    } else if kiting.is_some() {
        "Kiting"
    } else if regroup.is_some() {
        "Regroup"
    } else if chase.is_some() {
        "Chase"
    } else if attack.is_some() {
        "Attack"
    } else if patrol.is_some() {
        "Patrol"
    } else if flee.is_some() {
        "Flee"
    } else if guard.is_some() {
        "Guard"
    } else if follow.is_some() {
        "Follow"
    } else if kamikaze.is_some() {
        "Kamikaze"
    } else {
        "None"
    }
}
