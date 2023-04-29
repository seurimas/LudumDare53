mod assets;
mod game;
mod prelude;
mod state;
use game::GamePlugins;

use crate::prelude::*;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Hello Bevy!".to_string(),
                resolution: (948., 533.).into(),
                fit_canvas_to_parent: false,
                prevent_default_event_handling: true,
                ..Default::default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugins)
        .add_plugin(assets::GameAssetsPlugin)
        .add_system(spawn_camera.in_schedule(OnEnter(GameState::Playing)))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
