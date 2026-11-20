use bevy::prelude::*;

#[derive(Component, Clone, Copy, Debug, Default, Reflect, PartialEq)]
#[reflect(Component)]
pub struct Throttle {
    linear: f32,
    angular: f32,
}

impl Throttle {
    pub fn new(linear: f32, angular: f32) -> Self {
        let mut throttle = Self::default();
        throttle.set(linear, angular);
        throttle
    }

    pub fn set(&mut self, linear: f32, angular: f32) {
        self.linear = linear.clamp(0.0, 1.0);
        self.angular = angular.clamp(0.0, 1.0);
    }

    pub fn linear(&self) -> f32 {
        self.linear
    }

    pub fn angular(&self) -> f32 {
        self.angular
    }

    pub fn max_axis(&self) -> f32 {
        self.linear.max(self.angular)
    }
}
