use bevy::math::Vec2;
use rand::Rng;


pub fn random_vec2_step(min: Vec2, max: Vec2, step: f32) -> Vec2 {
    let mut rng = rand::rng();
    let x_steps = ((max.x - min.x) / step).floor() as i32;
    let y_steps = ((max.y - min.y) / step).floor() as i32;

    let x = min.x + rng.random_range(0..=x_steps) as f32 * step;
    let y = min.y + rng.random_range(0..=y_steps) as f32 * step;

    Vec2::new(x, y)
}
