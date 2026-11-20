use bevy::log::warn;
use uuid::Uuid;

pub fn sanitize_health(id: Uuid, health: f32, max_health: f32) -> (f32, f32) {
    let clean_max = if max_health.is_finite() && max_health > 0.0 {
        max_health
    } else {
        warn!("block {id} has invalid max health {max_health}; defaulting to 1");
        1.0
    };

    let clean_hp = if health.is_finite() && health >= 0.0 {
        health.min(clean_max)
    } else {
        warn!("block {id} has invalid health {health}; defaulting to max");
        clean_max
    };

    (clean_hp, clean_max)
}

pub fn sanitize_charge(id: Uuid, label: &str, charge: Option<f32>) -> Option<f32> {
    charge.and_then(|val| {
        if val.is_finite() && val >= 0.0 {
            Some(val)
        } else {
            warn!("block {id} has invalid {label} charge {val}; clamping to 0");
            Some(0.0)
        }
    })
}

pub fn sanitize_remaining_secs(label: &str, secs: f32) -> f32 {
    if secs.is_finite() && secs >= 0.0 {
        secs
    } else {
        warn!("{label} has invalid remaining secs {secs}; clamping to 0");
        0.0
    }
}
