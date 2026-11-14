use std::f32::consts::TAU;

/// Upper bound on locomotion anchors. Keeps weights in a fixed-size array.
pub const MAX_ANCHORS: usize = 8;

const ANGLE_EPS: f32 = 1e-4;

/// Wraps an angle into `[0, TAU)`.
pub fn wrap_turn(angle: f32) -> f32 {
    if !angle.is_finite() {
        return 0.0;
    }
    let wrapped = angle.rem_euclid(TAU);
    if wrapped >= TAU { 0.0 } else { wrapped }
}

/// Shortest signed delta between two angles, in `[-pi, pi]`.
pub fn wrap_delta(delta: f32) -> f32 {
    if !delta.is_finite() {
        return 0.0;
    }
    wrap_turn(delta + std::f32::consts::PI) - std::f32::consts::PI
}

/// Fixed-capacity locomotion blend weights, indexed like the sorted anchor list.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnchorWeights {
    count: usize,
    weights: [f32; MAX_ANCHORS],
}

impl Default for AnchorWeights {
    fn default() -> Self {
        Self {
            count: 0,
            weights: [0.0; MAX_ANCHORS],
        }
    }
}

impl AnchorWeights {
    pub fn new(count: usize) -> Self {
        Self {
            count: count.min(MAX_ANCHORS),
            weights: [0.0; MAX_ANCHORS],
        }
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    pub fn get(&self, index: usize) -> f32 {
        self.weights.get(index).copied().unwrap_or(0.0)
    }

    pub fn set(&mut self, index: usize, weight: f32) {
        if index < self.count {
            self.weights[index] = weight;
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.weights[..self.count].iter().copied()
    }

    pub fn sum(&self) -> f32 {
        self.iter().sum()
    }

    pub fn scale(&mut self, factor: f32) {
        for weight in &mut self.weights[..self.count] {
            *weight *= factor;
        }
    }

    pub fn normalize(&mut self, extra: f32) {
        let sum = self.sum();
        if sum <= 1e-5 {
            return;
        }
        let target = (1.0 - extra).max(0.0);
        let scale = target / sum;
        for weight in &mut self.weights[..self.count] {
            *weight *= scale;
        }
    }

    /// Highest-weight anchor, or `None` when every weight is negligible.
    pub fn dominant(&self) -> Option<usize> {
        let mut best: Option<(usize, f32)> = None;
        for (index, weight) in self.iter().enumerate() {
            if best.is_none_or(|(_, value)| weight > value) {
                best = Some((index, weight));
            }
        }
        best.filter(|(_, weight)| *weight > 1e-5)
            .map(|(index, _)| index)
    }

    /// Brackets `angle` with the two nearest anchors in a sorted `[0, TAU)` list.
    ///
    /// Exact anchor angles and the `+/-pi` seam produce a single or 50/50 blend
    /// without an allocation.
    pub fn bracket(angles: &[f32], angle: f32) -> Self {
        let count = angles.len().min(MAX_ANCHORS);
        let mut out = Self::new(count);
        if count == 0 {
            return out;
        }
        if count == 1 {
            out.set(0, 1.0);
            return out;
        }

        let target = wrap_turn(angle);
        let high = angles[..count]
            .iter()
            .position(|value| wrap_turn(*value) >= target - ANGLE_EPS)
            .unwrap_or(0);
        let high_angle = wrap_turn(angles[high]);
        if wrap_delta(high_angle - target).abs() <= ANGLE_EPS {
            out.set(high, 1.0);
            return out;
        }

        let low = (high + count - 1) % count;
        let low_angle = wrap_turn(angles[low]);
        let span = wrap_turn(high_angle - low_angle);
        if span <= ANGLE_EPS {
            out.set(low, 1.0);
            return out;
        }

        let t = (wrap_turn(target - low_angle) / span).clamp(0.0, 1.0);
        out.set(low, 1.0 - t);
        out.set(high, t);
        out
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
    let next = current + (target - current) * alpha.clamp(0.0, 1.0);
    if (target - next).abs() < 1e-4 {
        target
    } else {
        next
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Targets {
    pub weights: AnchorWeights,
    pub idle: f32,
}

pub fn target_weights(angle: f32, moving: bool, anchors: &[f32]) -> Targets {
    if !moving || anchors.is_empty() {
        return Targets {
            weights: AnchorWeights::new(anchors.len()),
            idle: 1.0,
        };
    }
    Targets {
        weights: AnchorWeights::bracket(anchors, angle),
        idle: 0.0,
    }
}

pub fn resize_weights(weights: &mut AnchorWeights, count: usize) {
    if weights.len() != count.min(MAX_ANCHORS) {
        let old = *weights;
        let mut replacement = AnchorWeights::new(count);
        for index in 0..replacement.len() {
            replacement.set(index, old.get(index));
        }
        *weights = replacement;
    }
}

pub fn seek_secs(phase: f32, duration: f32) -> f32 {
    if !duration.is_finite() || duration <= 0.0 {
        return 0.0;
    }
    phase.clamp(0.0, 1.0) * duration
}

pub fn advance_phase(phase: f32, dt: f32, playback: f32, duration: f32) -> f32 {
    if !duration.is_finite() || duration <= 0.0 || !dt.is_finite() || dt <= 0.0 {
        return phase;
    }
    let next = phase + dt * playback.max(0.0) / duration;
    next.rem_euclid(1.0)
}

#[cfg(test)]
mod tests {
use super::*;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    fn cardinals() -> Vec<f32> {
        vec![0.0, FRAC_PI_2, PI, 3.0 * FRAC_PI_2]
    }

    fn eight() -> Vec<f32> {
        (0..8).map(|index| index as f32 * FRAC_PI_4).collect()
    }

    fn assert_close(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "expected {b}, got {a}");
    }

    #[test]
    fn exact_anchor_angles_get_full_weight() {
        for (index, angle) in cardinals().into_iter().enumerate() {
            let weights = AnchorWeights::bracket(&cardinals(), angle);
            assert_close(weights.get(index), 1.0);
            assert_close(weights.sum(), 1.0);
            assert_eq!(weights.dominant(), Some(index));
        }
    }

    #[test]
    fn cardinal_diagonals_split_fifty_fifty() {
        let angles = cardinals();
        let right_forward = AnchorWeights::bracket(&angles, FRAC_PI_4);
        assert_close(right_forward.get(0), 0.5);
        assert_close(right_forward.get(1), 0.5);

        let back_right = AnchorWeights::bracket(&angles, 3.0 * FRAC_PI_4);
        assert_close(back_right.get(1), 0.5);
        assert_close(back_right.get(2), 0.5);
    }

    #[test]
    fn eight_anchor_exact_diagonals_are_single_poses() {
        let angles = eight();
        let diagonal = AnchorWeights::bracket(&angles, FRAC_PI_4);
        assert_close(diagonal.get(1), 1.0);
        assert_close(diagonal.sum(), 1.0);

        let between = AnchorWeights::bracket(&angles, FRAC_PI_4 + FRAC_PI_4 * 0.5);
        assert_close(between.get(1), 0.5);
        assert_close(between.get(2), 0.5);
    }

    #[test]
    fn seam_is_continuous_across_plus_minus_pi() {
        let angles = cardinals();
        let below = AnchorWeights::bracket(&angles, -FRAC_PI_4);
        assert_close(below.get(3), 0.5);
        assert_close(below.get(0), 0.5);

        let above = AnchorWeights::bracket(&angles, TAU - FRAC_PI_4);
        assert_close(above.get(3), 0.5);
        assert_close(above.get(0), 0.5);

        let almost = AnchorWeights::bracket(&angles, TAU - 0.01);
        assert!(almost.get(0) > 0.99);
        assert_close(almost.sum(), 1.0);
    }

    #[test]
    fn zero_and_one_anchor_fallbacks() {
        let empty = AnchorWeights::bracket(&[], 0.3);
        assert!(empty.is_empty());
        assert_eq!(empty.dominant(), None);

        let single = AnchorWeights::bracket(&[1.2], -2.0);
        assert_close(single.get(0), 1.0);
    }

    #[test]
    fn dominant_picks_the_largest_weight() {
        let mut weights = AnchorWeights::new(3);
        weights.set(0, 0.2);
        weights.set(1, 0.7);
        weights.set(2, 0.1);
        assert_eq!(weights.dominant(), Some(1));

        let zero = AnchorWeights::new(3);
        assert_eq!(zero.dominant(), None);
    }

    #[test]
    fn wrap_turn_and_delta_handle_full_turns() {
        assert_close(wrap_turn(TAU), 0.0);
        assert_close(wrap_turn(-FRAC_PI_2), 3.0 * FRAC_PI_2);
        assert_close(wrap_delta(0.1), 0.1);
        assert_close(wrap_delta(PI - 0.1), PI - 0.1);
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

    #[test]
    fn stationary_targets_are_idle_weighted() {
        let targets = target_weights(0.4, false, &[0.0, FRAC_PI_2]);
        assert_eq!(targets.idle, 1.0);
        assert_eq!(targets.weights.sum(), 0.0);
    }

    #[test]
    fn moving_targets_split_on_diagonals() {
        let targets = target_weights(FRAC_PI_4, true, &[0.0, FRAC_PI_2, PI]);
        assert_eq!(targets.idle, 0.0);
        assert!((targets.weights.get(0) - 0.5).abs() < 1e-4);
        assert!((targets.weights.get(1) - 0.5).abs() < 1e-4);
        assert!((targets.weights.sum() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn phase_seek_maps_normalized_time_into_duration() {
        assert_eq!(seek_secs(0.0, 2.0), 0.0);
        assert_eq!(seek_secs(0.5, 2.0), 1.0);
        assert_eq!(seek_secs(1.0, 2.0), 2.0);
        assert_eq!(seek_secs(0.5, 0.0), 0.0);
    }

    #[test]
    fn phase_advances_and_wraps() {
        let half = advance_phase(0.0, 0.5, 1.0, 2.0);
        assert!((half - 0.25).abs() < 1e-4);
        let wrapped = advance_phase(0.9, 0.5, 1.0, 2.0);
        assert!((wrapped - 0.15).abs() < 1e-4);
        assert_eq!(advance_phase(0.3, 1.0, 1.0, 0.0), 0.3);
    }
}
