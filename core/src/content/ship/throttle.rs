use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default, Reflect, PartialEq)]
#[reflect(Component)]
pub struct Throttle {
    pub linear: f32,
    pub angular: f32,
}

impl Throttle {
    pub fn set(&mut self, linear: f32, angular: f32) {
        self.linear = linear.clamp(0.0, 1.0);
        self.angular = angular.clamp(0.0, 1.0);
    }

    pub fn max_axis(&self) -> f32 {
        self.linear.max(self.angular)
    }
}
