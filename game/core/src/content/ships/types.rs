use bevy::prelude::*;

#[derive(Component)]
pub struct PlayerShipController;

#[derive(Component)]
pub struct AIShipController {
    pub behavior: AIShipBehavior,
    pub initial_behavior: AIShipBehavior,
}

#[derive(Debug, Clone, Copy)]
pub enum AIShipBehavior {
    Idle,
    Wander,
    Attack { target_pos: Vec2 },
    Patrol { around_pos: Vec2 },
    Chase { target: Entity },
    Flee { from: Entity },
    Guard { target: Entity },
    Follow { target: Entity },
}
