mod assets;
mod game;
mod menu;
mod prelude;
mod state;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_mod_picking::prelude::*;
use game::GamePlugins;
use menu::MenuPlugin;

#[macro_use]
extern crate lazy_static;
use crate::prelude::*;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Signs of Corruption".to_string(),
                resolution: (948., 533.).into(),
                fit_canvas_to_parent: false,
                prevent_default_event_handling: true,
                canvas: Some("#bevy".to_owned()),
                ..Default::default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::EntityCountDiagnosticsPlugin::default())
        .add_plugin(MenuPlugin)
        .add_plugin(assets::GameAssetsPlugin)
        .add_system(despawn_camera.in_schedule(OnEnter(GameState::Playing)))
        .add_system(spawn_menu_camera.in_schedule(OnEnter(GameState::Loading)))
        .add_system(spawn_menu_camera.in_schedule(OnEnter(GameState::MainMenu)))
        .add_system(despawn_menu_camera.in_schedule(OnEnter(GameState::MainMenu)))
        .run();
}

fn spawn_menu_camera(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), RaycastPickCamera::default()));
}

fn despawn_camera(mut commands: Commands, camera: Query<Entity, With<Camera>>) {
    for entity in camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn despawn_menu_camera(mut commands: Commands, camera: Query<Entity, With<Camera>>) {
    for entity in camera.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
