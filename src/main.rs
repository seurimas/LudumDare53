mod assets;
mod game;
mod menu;
mod prelude;
mod state;
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
                ..Default::default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugins)
        .add_plugin(MenuPlugin)
        .add_plugin(assets::GameAssetsPlugin)
        .add_system(spawn_camera.in_schedule(OnEnter(GameState::Playing)))
        .add_system(despawn_camera.in_schedule(OnEnter(GameState::Playing)))
        .add_system(spawn_menu_camera.in_schedule(OnEnter(GameState::MainMenu)))
        .add_system(despawn_menu_camera.in_schedule(OnEnter(GameState::MainMenu)))
        .add_system(move_camera.run_if(in_state(GameState::Playing)))
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn spawn_menu_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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

fn move_camera(
    time: Res<Time>,
    mut camera: Query<(&mut Transform, &Camera)>,
    input: Res<Input<KeyCode>>,
) {
    let (mut transform, _camera) = camera.single_mut();
    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::W) {
        direction += Vec3::Y;
    }
    if input.pressed(KeyCode::S) {
        direction -= Vec3::Y;
    }
    if input.pressed(KeyCode::A) {
        direction -= Vec3::X;
    }
    if input.pressed(KeyCode::D) {
        direction += Vec3::X;
    }
    if direction != Vec3::ZERO {
        transform.translation += direction.normalize() * 500. * time.delta_seconds();
    }
}
