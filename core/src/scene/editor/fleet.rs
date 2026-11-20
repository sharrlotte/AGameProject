use super::*;

use crate::save::design::remove;
use crate::scene::widget::{ButtonVisuals, spawn_button};


#[derive(Component)]
pub struct FleetListPanel;


#[derive(Component)]
pub struct FleetListContent;


#[derive(Component)]
pub struct RemoveFleetButton(pub String);


pub fn on_remove_fleet(
    click: On<PointerClick>,
    buttons: Query<&RemoveFleetButton>,
    mut deploy: ResMut<DeployPayload>,
) {
    let Ok(button) = buttons.get(click.event().entity) else {
        return;
    };
    remove(&mut deploy.fleet, &button.0);
}


pub fn refresh_fleet_list(
    mut commands: Commands,
    deploy: Res<DeployPayload>,
    fleet_list: Single<Entity, With<FleetListContent>>,
) {
    commands.entity(fleet_list.entity()).despawn_children();

    commands.entity(fleet_list.entity()).with_children(|parent| {
        for design in &deploy.fleet {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_shrink: 0.0,
                    ..default()
                })
                .with_children(|row| {
                    spawn_button(
                        row,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Px(ROW_HEIGHT),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(4.0)),
                            ..default()
                        },
                        design.name.clone(),
                        14.0,
                        Color::WHITE,
                        RemoveFleetButton(design.name.clone()),
                        on_remove_fleet,
                        ButtonVisuals::dark(),
                    );
                });
        }
    });
}
