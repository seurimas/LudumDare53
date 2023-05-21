use std::f32::consts::PI;

use bevy::ui::RelativeCursorPosition;

use crate::prelude::*;

pub type TileLoc = i32;

#[derive(Resource, Default)]
pub struct TileInputState {
    pub selected: Option<(Entity, TileLoc, TileLoc)>,
    pub hovered: Option<(Entity, TileLoc, TileLoc)>,
    pub traveling: Option<(Entity, TileLoc, TileLoc)>,
    pub noise_timer: f32,
}

impl TileInputState {
    pub fn clear(&mut self) {
        self.selected = None;
        self.hovered = None;
        self.traveling = None;
    }

    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected.map(|(e, _, _)| e == entity).unwrap_or(false)
    }

    pub fn is_hovered(&self, entity: Entity) -> bool {
        self.hovered.map(|(e, _, _)| e == entity).unwrap_or(false)
    }

    pub fn is_traveling(&self, entity: Entity) -> bool {
        self.traveling.map(|(e, _, _)| e == entity).unwrap_or(false)
    }

    pub fn is_touched(&self, entity: Entity) -> bool {
        self.is_selected(entity) || self.is_hovered(entity) || self.is_traveling(entity)
    }
}

#[derive(Component)]
pub struct MapTile {
    pub x: TileLoc,
    pub y: TileLoc,
    pub sprite_id: u32,
    pub offset: f32,
    pub target: f32,
}

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileInputState>()
            .add_system(load_map.in_schedule(OnEnter(GameState::Playing)))
            // .add_startup_system(spawn_camera_3d)
            .add_system(spawn_camera_3d.in_schedule(OnEnter(GameState::Playing)))
            .add_system(raise_map.run_if(in_state(GameState::Playing)))
            // .add_system(focus_tile.run_if(in_state(GameState::Playing)))
            .add_system(map_timer_system.run_if(in_state(GameState::Playing)))
            .add_system(map_tooltip.run_if(in_state(GameState::Playing)))
            .add_system(move_camera.run_if(in_state(GameState::Playing)));
    }
}

pub const MAP_WAVINESS: f32 = 0.25;
pub const WATER_SPRITE_ID: u32 = 5;
pub const WATER_COUNT: i32 = 7;

fn spawn_camera_3d(mut commands: Commands) {
    let mut projection = PerspectiveProjection::default();
    projection.far = projection.far.max(4000.0);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(5., 5., 5.)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            projection: projection.into(),
            ..default()
        },
        RaycastPickCamera::default(),
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 70000.0,
            color: Color::rgb(0.8, 0.8, 0.7),
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        ..default()
    });
}

fn load_map(
    mut commands: Commands,
    map: Res<MapDesc>,
    assets: Res<MyAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut camera: Query<&mut Transform, With<Camera>>,
) {
    for mut transform in camera.iter_mut() {
        let offset_x = map.width as f32;
        let offset_y = map.height as f32;
        *transform = Transform::from_xyz(offset_x + 5., 5., offset_y + 5.)
            .looking_at(Vec3::new(offset_x, 0.0, offset_y), Vec3::Y);
    }
    let pick_mesh = meshes.add(Mesh::from(shape::Box::new(2., 1., 2.)));
    let pick_material = materials.add(Color::WHITE.into());
    for y in 0..map.height {
        for x in 0..map.width {
            spawn_tile_3d(
                x,
                y,
                &map,
                &mut commands,
                &assets,
                pick_mesh.clone(),
                pick_material.clone(),
            );
        }
    }
    for y in -WATER_COUNT..(map.height as TileLoc + WATER_COUNT) {
        for x in -WATER_COUNT..(map.width as TileLoc + WATER_COUNT) {
            if x >= 0 && x < map.width as TileLoc && y >= 0 && y < map.height as TileLoc {
                continue;
            }
            // spawn_water_2d(x, y, &mut commands, &assets);
        }
    }
}

fn on_mouse_over_tile(
    In(event): In<ListenedEvent<Over>>,
    mut tiles: Query<&mut MapTile>,
    mut state: ResMut<TileInputState>,
    assets: Res<MyAssets>,
    audio: Res<Audio>,
) -> Bubble {
    if let Ok(mut tile) = tiles.get_mut(event.listener) {
        let volume = (state.noise_timer * state.noise_timer).clamp(0., 1.);
        audio.play_with_settings(
            assets.tile_hover.clone(),
            PlaybackSettings {
                repeat: false,
                volume,
                speed: rand::random::<f32>() * 0.5 + 0.5,
            },
        );
        state.noise_timer = 0.;
        tile.target = rand::random::<f32>() * MAP_WAVINESS;
        state.hovered = Some((event.listener, tile.x, tile.y));
    }
    println!("Over");
    Bubble::Up
}

fn on_mouse_out_tile(
    In(event): In<ListenedEvent<Out>>,
    tiles: Query<&MapTile>,
    mut state: ResMut<TileInputState>,
) -> Bubble {
    if let Ok(tile) = tiles.get(event.listener) {
        if state.hovered == Some((event.listener, tile.x, tile.y)) {
            state.hovered = None;
        }
    }
    println!("Out");
    Bubble::Up
}

fn on_mouse_click_tile(
    In(event): In<ListenedEvent<Click>>,
    mut tiles: Query<(&mut MapTile, Option<&WorldArea>)>,
    mut state: ResMut<TileInputState>,
    assets: Res<MyAssets>,
    audio: Res<Audio>,
    mut agent_action_query: Query<&mut AgentAction>,
) -> Bubble {
    if let Ok((mut tile, m_area)) = tiles.get_mut(event.listener) {
        if m_area.is_some() && event.button == PointerButton::Secondary {
            if state.selected == Some((event.listener, tile.x, tile.y)) {
                state.selected = None;
            }
            state.traveling = Some((event.listener, tile.x, tile.y));
            audio.play(assets.tile_click.clone());
            agent_action_query.iter_mut().for_each(|mut action| {
                if matches!(*action, AgentAction::Move(_, _, _)) && tile.x >= 0 && tile.y >= 0 {
                    *action = AgentAction::Move(
                        tile.x as u32,
                        tile.y as u32,
                        m_area.unwrap().name.clone(),
                    );
                }
            });
        } else if event.button == PointerButton::Primary {
            if state.traveling == Some((event.listener, tile.x, tile.y)) {
                state.traveling = None;
            }
            if state.selected != Some((event.listener, tile.x, tile.y)) {
                state.selected = Some((event.listener, tile.x, tile.y));
                audio.play(assets.tile_click.clone());
            }
            tile.target = rand::random::<f32>() * MAP_WAVINESS;
        }
    }
    Bubble::Up
}

fn spawn_tile_3d(
    x: u32,
    y: u32,
    map: &Res<MapDesc>,
    commands: &mut Commands,
    assets: &Res<MyAssets>,
    pick_mesh: Handle<Mesh>,
    pick_material: Handle<StandardMaterial>,
) {
    let sprite_id = map.get_tile(x, y);
    let mut tile = commands.spawn((
        // SceneBundle {
        //     // scene: assets.tiles[sprite_id as usize].clone(),
        //     transform: Transform {
        //         translation: Vec3::new(x as f32 * 2., 0., y as f32 * 2.),
        //         // rotation: Quat::from_rotation_y(PI / 2. * (rand::random::<f32>() * 4.).round()),
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // },
        MaterialMeshBundle {
            transform: Transform {
                translation: Vec3::new(x as f32 * 2., 0., y as f32 * 2.),
                // rotation: Quat::from_rotation_y(PI / 2. * (rand::random::<f32>() * 4.).round()),
                ..Default::default()
            },
            mesh: pick_mesh,
            material: pick_material,
            ..Default::default()
        },
        MapTile {
            x: x as i32,
            y: y as i32,
            sprite_id,
            offset: -rand::random::<f32>() * MAP_WAVINESS,
            target: rand::random::<f32>() * MAP_WAVINESS,
        },
        // pick_mesh,
        // pick_material,
        PickableBundle::default(),
        RaycastPickTarget::default(),
        OnPointer::<Over>::run_callback(on_mouse_over_tile),
        OnPointer::<Out>::run_callback(on_mouse_out_tile),
        OnPointer::<Click>::run_callback(on_mouse_click_tile),
    ));
    if let Some(area) = map.get_area(x, y) {
        tile.insert(area.clone());
    }
}

fn map_tooltip(
    mut is_showing_map_tooltip: Local<bool>,
    area_query: Query<&WorldArea>,
    mut tooltip: ResMut<Tooltip>,
    input_state: Res<TileInputState>,
) {
    let mut tooltip_value = None;
    if let Some(area) = input_state
        .hovered
        .and_then(|(entity, x, y)| area_query.get(entity).ok())
    {
        tooltip_value = Some(format!("{} ({})", area.name.clone(), area.get_value()));
    }
    if *is_showing_map_tooltip && tooltip_value.is_none() {
        *is_showing_map_tooltip = false;
        tooltip.value = None;
    } else if let Some(tooltip_value) = tooltip_value {
        *is_showing_map_tooltip = true;
        tooltip.value = Some(tooltip_value);
    }
}

fn raise_map(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MapTile)>,
    input_state: Res<TileInputState>,
) {
    for (entity, mut transform, mut tile) in query.iter_mut() {
        let translation = &mut transform.translation;
        if !input_state.is_touched(entity) {
            tile.offset = tile.offset + time.delta_seconds();
            if tile.offset > tile.target {
                tile.offset = tile.target;
            }
        } else if input_state.is_selected(entity) {
            tile.offset = -MAP_WAVINESS;
        } else if input_state.is_traveling(entity) {
            tile.offset = MAP_WAVINESS * 2.;
        } else if input_state.is_hovered(entity) {
            tile.offset = 0.;
        }
        translation.y = tile.offset;
    }
}

// fn focus_tile(
//     time: Res<Time>,
//     mut query: Query<&mut MapTile>,
//     mut camera: Query<(&mut Transform, &Camera)>,
// ) {
//     if let Some(mut focused) = query.iter_mut().find(|tile| tile.focused) {
//         let mut camera = camera.single_mut();
//         let translation = &mut camera.0.translation;
//         let (target_x, target_y) = get_world_pos_for_tile(focused.x, focused.y);
//         let delta = Vec2::new(target_x, target_y) - Vec2::new(translation.x, translation.y);
//         let direction = delta.normalize();
//         let length = delta.length();
//         if length < time.delta_seconds() * 1000. {
//             translation.x = target_x;
//             translation.y = target_y;
//             focused.focused = false;
//         } else {
//             translation.x += direction.x * time.delta_seconds() * 1000.;
//             translation.y += direction.y * time.delta_seconds() * 1000.;
//         }
//     }
// }

const MOUSE_SOUND_SCALE: f32 = 10.;

fn map_timer_system(time: Res<Time>, mut input_state: ResMut<TileInputState>) {
    input_state.noise_timer += time.delta_seconds() * MOUSE_SOUND_SCALE;
}

fn move_camera(
    time: Res<Time>,
    mut camera: Query<(&mut Transform, &Camera)>,
    input: Res<Input<KeyCode>>,
) {
    let (mut transform, _camera) = camera.single_mut();
    let mut direction = Vec3::ZERO;
    if input.pressed(KeyCode::W) {
        direction -= Vec3::new(0.707, 0., 0.707);
    }
    if input.pressed(KeyCode::S) {
        direction += Vec3::new(0.707, 0., 0.707);
    }
    if input.pressed(KeyCode::A) {
        direction -= Vec3::new(0.707, 0., -0.707);
    }
    if input.pressed(KeyCode::D) {
        direction += Vec3::new(0.707, 0., -0.707);
    }
    if direction != Vec3::ZERO {
        transform.translation += direction.normalize() * 10. * time.delta_seconds();
    }
}
