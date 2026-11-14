use bevy::prelude::*;
use crate::world::Survivor;

use crate::presentation::anim::PlayerVisualRoot;

pub fn attach_player_visuals(
    mut commands: Commands,
    survivors: Query<Entity, Added<Survivor>>,
) {
    for entity in &survivors {
        commands
            .entity(entity)
            .insert(Visibility::default())
            .with_children(|parent| {
                parent.spawn((
                    Name::new("PlayerVisualRoot"),
                    PlayerVisualRoot,
                    Transform::default(),
                    Visibility::default(),
                ));
            });
    }
}

