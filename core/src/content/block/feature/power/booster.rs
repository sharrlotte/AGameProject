use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier2d::{parry::shape::Ball, plugin::ReadRapierContext, prelude::QueryFilter};

use super::{
    component::{Boost, Booster, BoosterState},
    grid::{BoosterInput, GridScratch, allocate_boosters},
};
use crate::content::{
    block::{BlockOwner, Disabled},
    ship::Ship,
    team::Team,
};

pub fn power_boosters(
    time: Res<Time>,
    mut scratch: ResMut<GridScratch>,
    ships: Query<&Ship>,
    mut boosters: Query<
        (Entity, &BlockOwner, &Transform, &Booster, &mut BoosterState),
        Without<Disabled>,
    >,
) {
    let dt = time.delta_secs();
    let mut by_ship: HashMap<Entity, Vec<BoosterInput>> = HashMap::new();

    for (entity, owner, transform, booster, _) in &boosters {
        let core_pos = ships
            .get(owner.body)
            .map_or(Vec2::ZERO, |ship| ship.core_position);

        let distance_sq = transform.translation.truncate().distance_squared(core_pos);

        by_ship.entry(owner.body).or_default().push(BoosterInput {
            entity,
            distance_sq,
            rate: booster.rate,
        });
    }

    for list in by_ship.values_mut() {
        list.sort_by(|a, b| {
            a.distance_sq
                .total_cmp(&b.distance_sq)
                .then_with(|| a.entity.cmp(&b.entity))
        });
    }

    for (ship_entity, entry) in &mut scratch.ships {
        let available_power = if dt > 0.0 {
            entry.generation + entry.stored / dt
        } else {
            entry.generation
        };

        let Some(list) = by_ship.get(ship_entity) else {
            entry.booster_draw = 0.0;
            entry.remaining_available = available_power;
            continue;
        };

        let (powered_flags, draw, remaining) = allocate_boosters(available_power, list);
        entry.booster_draw = draw;
        entry.remaining_available = remaining;

        for (input, powered) in list.iter().zip(powered_flags) {
            if let Ok((_, _, _, _, mut state)) = boosters.get_mut(input.entity) {
                state.powered = powered;
            }
        }
    }
}

pub fn apply_boosts(
    mut blocks: Query<(&mut Boost, &Team)>,
    boosters: Query<(&GlobalTransform, &Booster, &BoosterState, &Team), Without<Disabled>>,
    rapier_context: ReadRapierContext,
) {
    for (mut boost, _) in &mut blocks {
        boost.value = 1.0;
    }

    let Ok(rapier) = rapier_context.single() else {
        return;
    };
    let filter = QueryFilter::default();

    for (transform, booster, state, booster_team) in &boosters {
        if !state.powered || booster.radius <= 0.0 || booster.boost <= 1.0 {
            continue;
        }
        let shape = Ball::new(booster.radius);
        let origin = transform.translation().truncate();

        rapier.intersect_shape(origin, 0.0, &shape, filter, |entity, _collider| {
            if let Ok((mut boost, team)) = blocks.get_mut(entity)
                && team.id == booster_team.id
            {
                boost.value = boost.value.max(booster.boost);
            }
            true
        });
    }
}

pub fn reset_disabled_booster(
    trigger: On<Add<Disabled>>,
    mut boosters: Query<&mut BoosterState>,
) {
    if let Ok(mut booster) = boosters.get_mut(trigger.event().entity) {
        booster.powered = false;
    }
}
