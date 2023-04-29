use crate::prelude::*;

#[derive(Component)]
pub struct MapTile {
    pub x: i32,
    pub y: i32,
    pub offset_y: f32,
    pub target_y: f32,
    pub base_y: f32,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(debug_map.in_schedule(OnEnter(GameState::Playing)))
            .add_system(raise_map.run_if(in_state(GameState::Playing)))
            .add_system(lower_map.run_if(in_state(GameState::Playing)));
    }
}

pub const TILE_SIZE: f32 = 128.;

fn debug_map(mut commands: Commands, assets: Res<MyAssets>) {
    println!("Spawning debug map");
    for x in 0..10 {
        for y in 0..10 {
            let base_x = (x as f32 + y as f32) * (TILE_SIZE / 2.);
            let base_y = (x as f32 - y as f32) * (TILE_SIZE / 4.);
            println!("Spawning tile at {}, {}", base_x, base_y);
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: assets.map.clone(),
                    transform: Transform {
                        translation: Vec3::new(base_x, base_y, 100. + (y - x) as f32),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                MapTile {
                    x,
                    y,
                    offset_y: TILE_SIZE,
                    target_y: rand::random::<f32>() * 8.,
                    base_y,
                },
            ));
        }
    }
}

fn raise_map(time: Res<Time>, mut query: Query<(&mut Transform, &mut MapTile)>) {
    for (mut transform, mut tile) in query.iter_mut() {
        let translation = &mut transform.translation;
        tile.offset_y = tile.offset_y + time.delta_seconds() * TILE_SIZE * 2.;
        if tile.offset_y > tile.target_y {
            tile.offset_y = tile.target_y;
        }
        translation.y = tile.base_y + tile.offset_y;
    }
}

fn lower_map(
    mut mouse_location: Local<Vec2>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut query: Query<&mut MapTile>,
    mut cursor_moved: EventReader<CursorMoved>,
) {
    for event in cursor_moved.iter() {
        *mouse_location = event.position;
    }
    if let Some((x, y)) = get_tile_at_screen_pos(*mouse_location, camera) {
        println!("Mouse is at {}, {} ({:?})", x, y, mouse_location);
        for mut map_tile in query.iter_mut() {
            if map_tile.x == x && map_tile.y == y {
                map_tile.offset_y = -8.;
            }
        }
    }
}

pub fn get_tile_at_screen_pos(
    location: Vec2,
    camera: Query<(&Camera, &GlobalTransform)>,
) -> Option<(i32, i32)> {
    let (camera, camera_transform) = camera.single();
    if let Some(mouse_world_location) = camera.viewport_to_world(camera_transform, location) {
        let (sx, sy) = (mouse_world_location.origin.x, mouse_world_location.origin.y);
        println!("Mouse world location: {:?}", mouse_world_location);
        let x = sx / TILE_SIZE + sy / (TILE_SIZE / 2.);
        let y = sx / TILE_SIZE - sy / (TILE_SIZE / 2.);
        Some((x.round() as i32, y.round() as i32))
    } else {
        None
    }
}
