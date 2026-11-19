use bevy::prelude::*;

use super::*;
use crate::content::block::{
    BlockOwner, Disabled,
    feature::power::{Boost, Consumer, Demand, STARVATION_THRESHOLD, Supply},
    stat::make_stat_system,
};
use crate::content::ship::Throttle;


pub fn update_thruster_demand(
    ships: Query<&Throttle>,
    mut thrusters: Query<
        (&BlockOwner, &Consumer, &mut Demand, Option<&Boost>),
        (With<ThrustPowerBase>, Without<Disabled>),
    >,
) {
    for (owner, consumer, mut demand, boost) in &mut thrusters {
        let throttle = ships.get(owner.body).map_or(0.0, Throttle::max_axis);
        let boost = boost.map_or(1.0, Boost::value);
        demand.value = consumer.rate() * throttle * boost;
    }
}

make_stat_system!(
    update_thrust_power_stats,
    ThrustPowerBase,
    ThrustPowerComputed,
    ThrustPowerEffects,
    |base, eff| base * eff
);
make_stat_system!(
    update_rotate_speed_stats,
    RotateSpeedBase,
    RotateSpeedComputed,
    RotateSpeedEffects,
    |base, eff| base * eff
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_thruster_draws_nothing_and_scales_with_throttle_and_boost() {
        let mut app = App::new();
        app.add_systems(Update, update_thruster_demand);

        let ship = app.world_mut().spawn(Throttle::default()).id();
        
        let thruster = app
            .world_mut()
            .spawn((
                BlockOwner { body: ship },
                ThrustPowerBase::new(100.0),
                Consumer { rate: 3.0 },
                Demand::default(),
                Boost { value: 1.0 },
            ))
            .id();

        app.update();
        assert_eq!(app.world().get::<Demand>(thruster).unwrap().value, 0.0);

        app.world_mut().get_mut::<Throttle>(ship).unwrap().set(1.0, 0.0);
        app.update();
        assert_eq!(app.world().get::<Demand>(thruster).unwrap().value, 3.0);

        app.world_mut().get_mut::<Throttle>(ship).unwrap().set(0.0, 0.5);
        app.update();
        assert_eq!(app.world().get::<Demand>(thruster).unwrap().value, 1.5);

        app.world_mut().get_mut::<Throttle>(ship).unwrap().set(1.0, 0.0);
        app.world_mut().get_mut::<Boost>(thruster).unwrap().value = 1.5;
        app.update();
        assert_eq!(app.world().get::<Demand>(thruster).unwrap().value, 4.5);
    }

    #[test]
    fn thruster_output_scales_with_supply_and_boost_and_disables_below_threshold() {
        let mut app = App::new();
        app.add_systems(Update, update_thrust_power_stats);

        let powered = app
            .world_mut()
            .spawn((
                ThrustPowerBase::new(200.0),
                ThrustPowerComputed::default(),
                Supply { value: 0.5 },
                Boost { value: 1.5 },
            ))
            .id();
        let unpowered = app
            .world_mut()
            .spawn((
                ThrustPowerBase::new(200.0),
                ThrustPowerComputed::default(),
                Boost { value: 1.5 },
            ))
            .id();
        let starved = app
            .world_mut()
            .spawn((
                ThrustPowerBase::new(200.0),
                ThrustPowerComputed::default(),
                Supply { value: 0.005 },
                Boost { value: 1.5 },
            ))
            .id();

        app.update();
        assert_eq!(
            app.world().get::<ThrustPowerComputed>(powered).unwrap().value(),
            150.0
        );
        assert_eq!(
            app.world()
                .get::<ThrustPowerComputed>(unpowered)
                .unwrap()
                .value(),
            300.0
        );
        assert_eq!(
            app.world().get::<ThrustPowerComputed>(starved).unwrap().value(),
            0.0
        );
    }
}
