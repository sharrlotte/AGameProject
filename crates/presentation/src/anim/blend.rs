use bevy::prelude::*;

/// Cardinal locomotion weights in character-local space.
///
/// `x` is right, `y` is forward. Weights always sum to `1.0` after
/// normalization, or are all zero when there is no movement input.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Weights {
    pub forward: f32,
    pub back: f32,
    pub left: f32,
    pub right: f32,
}

impl Weights {
    /// Splits a local-space movement direction into four cardinal weights.
    ///
    /// A diagonal produces two weights of `0.707`; zero input produces all
    /// zero weights so the caller can select idle.
    pub fn from_local(dir: Vec2) -> Self {
        let len = dir.length();
        if len <= 1e-4 || !len.is_finite() {
            return Self::default();
        }
        let dir = dir / len;
        let mut weights = Self {
            forward: dir.y.max(0.0),
            back: (-dir.y).max(0.0),
            left: (-dir.x).max(0.0),
            right: dir.x.max(0.0),
        };
        // Projections are normalized so the blend contract always sums to 1.0.
        // Bevy's blend node normalizes its children again, so this is stable.
        weights.normalize();
        weights
    }

    pub fn normalize(&mut self) {
        let total = self.total();
        if total <= 1e-5 {
            *self = Self::default();
            return;
        }
        self.forward /= total;
        self.back /= total;
        self.left /= total;
        self.right /= total;
    }

    pub fn normalized(mut self) -> Self {
        self.normalize();
        self
    }

    pub fn total(&self) -> f32 {
        self.forward + self.back + self.left + self.right
    }

    pub fn scaled(self, factor: f32) -> Self {
        Self {
            forward: self.forward * factor,
            back: self.back * factor,
            left: self.left * factor,
            right: self.right * factor,
        }
    }

    /// Moves backward weight to forward when the back clip is unavailable.
    pub fn fallback_back_to_forward(mut self) -> Self {
        self.forward += self.back;
        self.back = 0.0;
        self
    }

    /// Moves diagonal side weight to forward when a strafe clip is missing.
    pub fn fallback_sides_to_forward(mut self) -> Self {
        self.forward += self.left + self.right;
        self.left = 0.0;
        self.right = 0.0;
        self
    }

    pub fn is_idle(&self) -> bool {
        self.total() <= 1e-5
    }
}

/// Playback rate that keeps walk and strafe clips roughly in sync with movement.
pub fn speed_scale(speed: f32, reference: f32, min: f32, max: f32) -> f32 {
    if !speed.is_finite() || reference <= 0.0 {
        return 1.0;
    }
    let lo = min.max(0.01);
    let hi = max.max(lo);
    (speed / reference).clamp(lo, hi)
}

/// Exponential approach that is stable for any frame delta.
pub fn approach(current: f32, target: f32, delta_secs: f32, blend_secs: f32) -> f32 {
    if blend_secs <= 0.0 || delta_secs <= 0.0 {
        return target;
    }
    let alpha = 1.0 - (-delta_secs / blend_secs).exp();
    current + (target - current) * alpha.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_close(a: f32, b: f32) {
        assert!((a - b).abs() < 0.001, "expected {b}, got {a}");
    }

    #[test]
    fn eight_directions_split_into_cardinal_weights() {
        let fwd = Weights::from_local(Vec2::new(0.0, 1.0));
        assert_close(fwd.forward, 1.0);

        let back = Weights::from_local(Vec2::new(0.0, -1.0));
        assert_close(back.back, 1.0);

        let left = Weights::from_local(Vec2::new(-1.0, 0.0));
        assert_close(left.left, 1.0);

        let right = Weights::from_local(Vec2::new(1.0, 0.0));
        assert_close(right.right, 1.0);

        let diagonal = Weights::from_local(Vec2::new(1.0, 1.0));
        assert_close(diagonal.forward, 0.5);
        assert_close(diagonal.right, 0.5);
        assert_close(diagonal.total(), 1.0);
    }

    #[test]
    fn zero_input_is_idle() {
        let weights = Weights::from_local(Vec2::ZERO);
        assert!(weights.is_idle());
        assert_close(weights.total(), 0.0);
    }

    #[test]
    fn missing_cardinals_fall_back_to_forward() {
        let back = Weights::from_local(Vec2::new(0.0, -1.0)).fallback_back_to_forward();
        assert_close(back.forward, 1.0);
        assert_close(back.back, 0.0);

        let sides = Weights::from_local(Vec2::new(1.0, 0.0)).fallback_sides_to_forward();
        assert_close(sides.forward, 1.0);
        assert_close(sides.right, 0.0);
    }

    #[test]
    fn speed_scale_clamps_and_guards_zero_reference() {
        assert_close(speed_scale(7.5, 7.5, 0.6, 1.6), 1.0);
        assert_close(speed_scale(100.0, 7.5, 0.6, 1.6), 1.6);
        assert_close(speed_scale(0.0, 7.5, 0.6, 1.6), 0.6);
        assert_close(speed_scale(7.5, 0.0, 0.6, 1.6), 1.0);
    }

    #[test]
    fn approach_is_stable_and_reaches_target() {
        assert_close(approach(0.0, 1.0, 1.0, 0.0), 1.0);
        let halfway = approach(0.0, 1.0, 0.12, 0.12);
        assert!(halfway > 0.5 && halfway < 1.0);
    }
}
