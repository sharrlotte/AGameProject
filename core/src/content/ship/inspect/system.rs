use bevy::prelude::*;

use crate::content::ship::Ship;
use crate::content::team::Team;
use crate::scene::widget::ButtonVisualState;

use super::*;

pub fn ignore_hud_picking(
    mut commands: Commands,
    nodes: Query<(Entity, &Node), (Without<ButtonVisualState>, Without<Pickable>)>,
) {
    for (entity, _) in &nodes {
        commands.entity(entity).insert(Pickable::IGNORE);
    }
}

pub fn on_ship_click(
    click: On<PointerClick>,
    keyboard: Res<ButtonInput<KeyCode>>,
    targets: Query<(&Team, &Visibility)>,
    mut events: MessageWriter<ShipSelected>,
) {
    if click.button != PointerButton::Primary || keyboard.pressed(KeyCode::ControlLeft) {
        return;
    }

    let entity = click.event().event_target();
    let Ok((team, visibility)) = targets.get(entity) else {
        return;
    };
    if *visibility == Visibility::Hidden || !can_inspect_team(team) {
        return;
    }

    events.write(ShipSelected::toggle(entity));
}

pub fn clear_selection_on_escape(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    inspected: Res<InspectedShip>,
    mut events: MessageWriter<ShipSelectionCleared>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && inspected.0.is_some() {
        events.write(ShipSelectionCleared);
        keyboard.clear_just_pressed(KeyCode::Escape);
    }
}

pub fn apply_selection(
    mut inspected: ResMut<InspectedShip>,
    ships: Query<(&Team, &Visibility)>,
    mut selects: MessageReader<ShipSelected>,
    mut clears: MessageReader<ShipSelectionCleared>,
) {
    for event in selects.read() {
        if let Ok((team, visibility)) = ships.get(event.entity)
            && (*visibility == Visibility::Hidden || !can_inspect_team(team))
        {
            continue;
        }

        if event.toggle && inspected.0 == Some(event.entity) {
            inspected.0 = None;
        } else {
            inspected.0 = Some(event.entity);
        }
    }

    if clears.read().next().is_some() {
        inspected.0 = None;
    }
}

pub fn remove_despawned_ships(
    mut inspected: ResMut<InspectedShip>,
    mut removed: RemovedComponents<Ship>,
) {
    let Some(current) = inspected.0 else {
        removed.read().for_each(|_| {});
        return;
    };

    for entity in removed.read() {
        if entity == current {
            inspected.0 = None;
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::selectable::EntitySelectEvent;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<InspectedShip>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<ShipSelected>()
            .add_message::<ShipSelectionCleared>()
            .add_message::<EntitySelectEvent>()
            .add_systems(
                Update,
                (clear_selection_on_escape, apply_selection, remove_despawned_ships).chain(),
            );
        app
    }

    fn spawn_ship(app: &mut App, team: Team) -> Entity {
        app.world_mut()
            .spawn((Ship::default(), team, Visibility::Visible))
            .id()
    }

    fn toggle(app: &mut App, entity: Entity) {
        app.world_mut()
            .resource_mut::<Messages<ShipSelected>>()
            .write(ShipSelected::toggle(entity));
        app.update();
    }

    #[test]
    fn click_toggles_player_ship() {
        let mut app = test_app();
        let entity = spawn_ship(&mut app, Team::new(0));

        toggle(&mut app, entity);
        assert_eq!(app.world().resource::<InspectedShip>().0, Some(entity));

        toggle(&mut app, entity);
        assert_eq!(app.world().resource::<InspectedShip>().0, None);
    }

    #[test]
    fn selecting_second_ship_replaces_first() {
        let mut app = test_app();
        let first = spawn_ship(&mut app, Team::new(0));
        let second = spawn_ship(&mut app, Team::new(0));

        toggle(&mut app, first);
        assert_eq!(app.world().resource::<InspectedShip>().0, Some(first));

        toggle(&mut app, second);
        assert_eq!(app.world().resource::<InspectedShip>().0, Some(second));
    }

    #[test]
    fn non_toggle_select_keeps_ship_inspected() {
        let mut app = test_app();
        let entity = spawn_ship(&mut app, Team::new(0));

        for _ in 0..2 {
            app.world_mut()
                .resource_mut::<Messages<ShipSelected>>()
                .write(ShipSelected::select(entity));
            app.update();
        }

        assert_eq!(app.world().resource::<InspectedShip>().0, Some(entity));
    }

    #[test]
    fn enemy_selection_respects_dev_feature() {
        let mut app = test_app();
        let enemy = spawn_ship(&mut app, Team::new(1));

        toggle(&mut app, enemy);

        #[cfg(feature = "dev")]
        assert_eq!(app.world().resource::<InspectedShip>().0, Some(enemy));

        #[cfg(not(feature = "dev"))]
        assert_eq!(app.world().resource::<InspectedShip>().0, None);
    }

    #[test]
    fn hidden_enemy_cannot_be_inspected() {
        let mut app = test_app();
        let enemy = spawn_ship(&mut app, Team::new(1));
        *app.world_mut()
            .get_mut::<Visibility>(enemy)
            .expect("enemy has visibility") = Visibility::Hidden;

        toggle(&mut app, enemy);

        assert_eq!(app.world().resource::<InspectedShip>().0, None);
    }

    #[test]
    fn escape_clears_selection() {
        let mut app = test_app();
        let entity = spawn_ship(&mut app, Team::new(0));
        toggle(&mut app, entity);

        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Escape);
        app.update();

        assert_eq!(app.world().resource::<InspectedShip>().0, None);
    }

    #[test]
    fn despawn_clears_inspected_ship() {
        let mut app = test_app();
        let entity = spawn_ship(&mut app, Team::new(0));
        toggle(&mut app, entity);

        app.world_mut().entity_mut(entity).despawn();
        app.update();

        assert_eq!(app.world().resource::<InspectedShip>().0, None);
    }

    #[test]
    fn selection_does_not_emit_gameplay_select_event() {
        let mut app = test_app();
        let entity = spawn_ship(&mut app, Team::new(0));

        toggle(&mut app, entity);

        assert!(app.world().resource::<Messages<EntitySelectEvent>>().is_empty());
    }
}
