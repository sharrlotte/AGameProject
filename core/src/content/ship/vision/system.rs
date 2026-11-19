use bevy::prelude::*;

use crate::content::ship::ai::TacticalBoard;
use crate::content::team::Team;

use super::lib::visible_to_player;

pub fn sync_enemy_visibility(
    board: Res<TacticalBoard>,
    mut ships: Query<(Entity, &Team, &mut Visibility)>,
) {
    for (entity, team, mut visibility) in &mut ships {
        let target = if visible_to_player(&board, team, entity) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };

        if *visibility != target {
            *visibility = target;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::content::ship::ai::{LastSeen, SharedContact};
    use crate::content::team::DEFAULT_TEAM_ID;

    fn test_app() -> App {
        let mut app = App::new();
        app.init_resource::<TacticalBoard>()
            .add_systems(Update, sync_enemy_visibility);
        app
    }

    fn spawn_ship(app: &mut App, team: u8) -> Entity {
        app.world_mut()
            .spawn((Team::new(team), Visibility::Visible))
            .id()
    }

    fn add_contact(app: &mut App, entity: Entity) {
        let mut board = app.world_mut().resource_mut::<TacticalBoard>();
        let plan = board.teams.entry(DEFAULT_TEAM_ID).or_default();
        plan.intel.contacts.insert(
            entity,
            SharedContact {
                entity,
                position: Vec2::ZERO,
                velocity: Vec2::ZERO,
                estimated_threat: 1.0,
                has_fixed_gun: false,
                heading: Vec2::Y,
            },
        );
    }

    fn add_last_known(app: &mut App, entity: Entity) {
        let mut board = app.world_mut().resource_mut::<TacticalBoard>();
        let plan = board.teams.entry(DEFAULT_TEAM_ID).or_default();
        plan.intel.last_known.insert(
            entity,
            LastSeen {
                position: Vec2::ZERO,
                velocity: Vec2::ZERO,
                ttl: 5.0,
            },
        );
    }

    fn visibility(app: &App, entity: Entity) -> Visibility {
        *app.world()
            .get::<Visibility>(entity)
            .expect("ship has visibility")
    }

    #[test]
    fn undetected_enemy_is_hidden() {
        let mut app = test_app();
        let enemy = spawn_ship(&mut app, 1);

        app.update();

        assert_eq!(visibility(&app, enemy), Visibility::Hidden);
    }

    #[test]
    fn enemy_in_player_contacts_is_visible() {
        let mut app = test_app();
        let enemy = spawn_ship(&mut app, 1);
        add_contact(&mut app, enemy);

        app.update();

        assert_eq!(visibility(&app, enemy), Visibility::Visible);
    }

    #[test]
    fn contact_from_any_ally_reveals_enemy() {
        let mut app = test_app();
        let ally = spawn_ship(&mut app, 0);
        let enemy = spawn_ship(&mut app, 2);
        add_contact(&mut app, enemy);

        app.update();

        assert_eq!(visibility(&app, ally), Visibility::Visible);
        assert_eq!(visibility(&app, enemy), Visibility::Visible);
    }

    #[test]
    fn player_ships_always_stay_visible() {
        let mut app = test_app();
        let player = spawn_ship(&mut app, 0);

        app.update();

        assert_eq!(visibility(&app, player), Visibility::Visible);
    }

    #[test]
    fn lost_contact_hides_enemy_even_with_last_known_memory() {
        let mut app = test_app();
        let enemy = spawn_ship(&mut app, 1);
        add_last_known(&mut app, enemy);

        app.update();

        assert_eq!(visibility(&app, enemy), Visibility::Hidden);
    }
}
