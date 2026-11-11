use bevy::prelude::*;
use common::GameState;

#[test]
fn app_initializes_with_all_plugins() {
    let app = app::build_app();
    assert!(app.world().contains_resource::<State<GameState>>());
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::Loading
    );
}
