use bevy::ui::RelativeCursorPosition;

use crate::prelude::*;

#[derive(Component)]
pub struct MapTile {
    pub selected: bool,
    pub hovered: bool,
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
            .add_system(map_mouse_system.run_if(in_state(GameState::Playing)));
    }
}

pub const TILE_SIZE: f32 = 128.;

fn debug_map(mut commands: Commands, assets: Res<MyAssets>) {
    println!("Spawning debug map");
    for x in 0..10 {
        for y in 0..10 {
            let base_x = (x as f32 + y as f32) * (TILE_SIZE / 2.);
            let base_y = (x as f32 - y as f32) * (TILE_SIZE / 4.);
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: assets.map.clone(),
                    transform: Transform {
                        translation: Vec3::new(base_x, base_y, 100. + (y - x) as f32),
                        ..Default::default()
                    },
                    sprite: TextureAtlasSprite::new((rand::random::<f32>() * 5.) as usize),
                    ..Default::default()
                },
                MapTile {
                    selected: false,
                    hovered: false,
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
        if !tile.selected && !tile.hovered {
            tile.offset_y = tile.offset_y + time.delta_seconds() * TILE_SIZE * 2.;
            if tile.offset_y > tile.target_y {
                tile.offset_y = tile.target_y;
            }
        }
        translation.y = tile.base_y + tile.offset_y;
    }
}

const MOUSE_SOUND_SCALE: f32 = 10.;

fn map_mouse_system(
    mut mouse_location_timer: Local<(Vec2, f32)>,
    time: Res<Time>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut tile_query: Query<&mut MapTile>,
    ui_query: Query<&RelativeCursorPosition>,
    mut cursor_moved: EventReader<CursorMoved>,
    mouse_button_input: Res<Input<MouseButton>>,
    assets: Res<MyAssets>,
    audio: Res<Audio>,
) {
    for event in cursor_moved.iter() {
        mouse_location_timer.0 = event.position;
    }
    mouse_location_timer.1 += time.delta_seconds() * MOUSE_SOUND_SCALE;
    if !ui_query.iter().all(|rcp| !rcp.mouse_over()) {
        return;
    }
    if let Some((x, y)) = get_tile_at_screen_pos(mouse_location_timer.0, camera) {
        for mut map_tile in tile_query.iter_mut() {
            if map_tile.x == x && map_tile.y == y {
                if mouse_button_input.just_pressed(MouseButton::Left) || map_tile.selected {
                    if !map_tile.selected {
                        audio.play(assets.tile_click.clone());
                    }
                    map_tile.selected = true;
                    map_tile.offset_y = -8.;
                    map_tile.target_y = rand::random::<f32>() * 8.;
                } else if !map_tile.selected {
                    if !map_tile.hovered {
                        let volume =
                            (mouse_location_timer.1 * mouse_location_timer.1).clamp(0., 1.);
                        audio.play_with_settings(
                            assets.tile_hover.clone(),
                            PlaybackSettings {
                                repeat: false,
                                volume,
                                speed: rand::random::<f32>() * 0.5 + 0.5,
                            },
                        );
                        mouse_location_timer.1 = 0.;
                    }
                    map_tile.hovered = true;
                    map_tile.offset_y = 0.;
                    map_tile.target_y = rand::random::<f32>() * 8.;
                }
            } else if mouse_button_input.just_pressed(MouseButton::Left) {
                map_tile.selected = false;
                map_tile.hovered = false;
            } else {
                map_tile.hovered = false;
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
        let x = sx / TILE_SIZE + sy / (TILE_SIZE / 2.);
        let y = sx / TILE_SIZE - sy / (TILE_SIZE / 2.);
        Some((x.round() as i32, y.round() as i32))
    } else {
        None
    }
}
