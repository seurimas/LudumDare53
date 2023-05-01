use bevy::ui::RelativeCursorPosition;

use crate::prelude::*;

use super::tooltip::Tooltip;

pub type TileLoc = i32;

#[derive(Component)]
pub struct MapTile {
    pub selected: bool,
    pub hovered: bool,
    pub focused: bool,
    pub travelling: bool,
    pub x: TileLoc,
    pub y: TileLoc,
    pub sprite_id: u32,
    pub offset_y: f32,
    pub target_y: f32,
    pub base_y: f32,
}

pub struct TilesPlugin;

impl Plugin for TilesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(load_map.in_schedule(OnEnter(GameState::Playing)))
            .add_system(raise_map.run_if(in_state(GameState::Playing)))
            .add_system(focus_tile.run_if(in_state(GameState::Playing)))
            .add_system(map_mouse_system.run_if(in_state(GameState::Playing)))
            .add_system(map_tooltip.run_if(in_state(GameState::Playing)));
    }
}

pub const TILE_SIZE: f32 = 128.;
pub const WATER_SPRITE_ID: u32 = 5;
pub const WATER_COUNT: i32 = 7;

fn load_map(mut commands: Commands, map: Res<MapDesc>, assets: Res<MyAssets>) {
    for y in 0..map.height {
        for x in 0..map.width {
            let (base_x, base_y) = get_world_pos_for_tile(x as TileLoc, y as TileLoc);
            let sprite_id = map.get_tile(x, y);
            let mut tile = commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: assets.map.clone(),
                    transform: Transform {
                        translation: Vec3::new(base_x, base_y, 100. + (y as i32 - x as i32) as f32),
                        ..Default::default()
                    },
                    sprite: TextureAtlasSprite::new(sprite_id as usize),
                    ..Default::default()
                },
                MapTile {
                    selected: false,
                    hovered: false,
                    focused: false,
                    travelling: false,
                    x: x as i32,
                    y: y as i32,
                    sprite_id,
                    offset_y: TILE_SIZE,
                    target_y: rand::random::<f32>() * 8.,
                    base_y,
                },
            ));
            if let Some(area) = map.get_area(x, y) {
                tile.insert(area.clone());
            }
        }
    }
    for y in -WATER_COUNT..(map.height as TileLoc + WATER_COUNT) {
        for x in -WATER_COUNT..(map.width as TileLoc + WATER_COUNT) {
            if x >= 0 && x < map.width as TileLoc && y >= 0 && y < map.height as TileLoc {
                continue;
            }
            let (base_x, base_y) = get_world_pos_for_tile(x, y);
            let sprite_id = WATER_SPRITE_ID;
            commands.spawn((
                SpriteSheetBundle {
                    texture_atlas: assets.map.clone(),
                    transform: Transform {
                        translation: Vec3::new(base_x, base_y, 100. + (y - x) as f32),
                        ..Default::default()
                    },
                    sprite: TextureAtlasSprite::new(sprite_id as usize),
                    ..Default::default()
                },
                MapTile {
                    selected: false,
                    hovered: false,
                    focused: false,
                    travelling: false,
                    x,
                    y,
                    sprite_id,
                    offset_y: TILE_SIZE,
                    target_y: rand::random::<f32>() * 8.,
                    base_y,
                },
            ));
        }
    }
}

fn map_tooltip(
    mut is_showing_map_tooltip: Local<bool>,
    map_query: Query<(&MapTile, &WorldArea)>,
    mut tooltip: ResMut<Tooltip>,
) {
    let mut tooltip_value = None;
    for (tile, area) in map_query.iter() {
        if tile.hovered {
            tooltip_value = Some(format!("{} ({})", area.name.clone(), area.get_value()));
        }
    }
    if *is_showing_map_tooltip && tooltip_value.is_none() {
        *is_showing_map_tooltip = false;
        tooltip.value = None;
    } else if let Some(tooltip_value) = tooltip_value {
        *is_showing_map_tooltip = true;
        tooltip.value = Some(tooltip_value);
    }
}

fn raise_map(time: Res<Time>, mut query: Query<(&mut Transform, &mut MapTile)>) {
    for (mut transform, mut tile) in query.iter_mut() {
        let translation = &mut transform.translation;
        if !tile.selected && !tile.hovered && !tile.travelling {
            tile.offset_y = tile.offset_y + time.delta_seconds() * TILE_SIZE * 2.;
            if tile.offset_y > tile.target_y {
                tile.offset_y = tile.target_y;
            }
        } else if tile.selected {
            tile.offset_y = -8.;
        } else if tile.travelling {
            tile.offset_y = 16.;
        } else if tile.hovered {
            tile.offset_y = 0.;
        }
        translation.y = tile.base_y + tile.offset_y;
    }
}

fn focus_tile(
    time: Res<Time>,
    mut query: Query<&mut MapTile>,
    mut camera: Query<(&mut Transform, &Camera)>,
) {
    if let Some(mut focused) = query.iter_mut().find(|tile| tile.focused) {
        let mut camera = camera.single_mut();
        let translation = &mut camera.0.translation;
        let (target_x, target_y) = get_world_pos_for_tile(focused.x, focused.y);
        let delta = Vec2::new(target_x, target_y) - Vec2::new(translation.x, translation.y);
        let direction = delta.normalize();
        let length = delta.length();
        if length < time.delta_seconds() * 1000. {
            translation.x = target_x;
            translation.y = target_y;
            focused.focused = false;
        } else {
            translation.x += direction.x * time.delta_seconds() * 1000.;
            translation.y += direction.y * time.delta_seconds() * 1000.;
        }
    }
}

const MOUSE_SOUND_SCALE: f32 = 10.;

fn map_mouse_system(
    mut mouse_location_timer: Local<(Vec2, f32)>,
    time: Res<Time>,
    camera: Query<(&Camera, &GlobalTransform)>,
    mut tile_query: Query<(&mut MapTile, Option<&WorldArea>)>,
    mut agent_action_query: Query<&mut AgentAction>,
    ui_query: Query<(&ComputedVisibility, &RelativeCursorPosition)>,
    mut cursor_moved: EventReader<CursorMoved>,
    mouse_button_input: Res<Input<MouseButton>>,
    assets: Res<MyAssets>,
    audio: Res<Audio>,
) {
    for event in cursor_moved.iter() {
        mouse_location_timer.0 = event.position;
    }
    mouse_location_timer.1 += time.delta_seconds() * MOUSE_SOUND_SCALE;
    if ui_query
        .iter()
        .any(|(visible, rcp)| visible.is_visible() && rcp.mouse_over())
    {
        for (mut map_tile, _) in tile_query.iter_mut() {
            map_tile.hovered = false;
        }
        return;
    }
    if let Some((x, y)) = get_tile_at_screen_pos(mouse_location_timer.0, camera) {
        for (mut map_tile, m_area) in tile_query.iter_mut() {
            if map_tile.x == x && map_tile.y == y {
                if m_area.is_some()
                    && (mouse_button_input.just_pressed(MouseButton::Left) || map_tile.selected)
                {
                    if !map_tile.selected {
                        audio.play(assets.tile_click.clone());
                    }
                    map_tile.selected = true;
                    map_tile.target_y = rand::random::<f32>() * 8.;
                } else if m_area.is_some() && mouse_button_input.just_pressed(MouseButton::Right) {
                    audio.play(assets.tile_click.clone());
                    agent_action_query.iter_mut().for_each(|mut action| {
                        if matches!(*action, AgentAction::Move(_, _, _)) && x >= 0 && y >= 0 {
                            *action =
                                AgentAction::Move(x as u32, y as u32, m_area.unwrap().name.clone());
                        }
                    });
                    map_tile.travelling = true;
                    map_tile.selected = false;
                } else if !map_tile.selected && !map_tile.travelling {
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
                    map_tile.target_y = rand::random::<f32>() * 8.;
                }
            } else if mouse_button_input.just_pressed(MouseButton::Left) {
                map_tile.selected = false;
                map_tile.hovered = false;
            } else if mouse_button_input.just_pressed(MouseButton::Right) {
                map_tile.travelling = false;
            } else {
                map_tile.hovered = false;
            }
        }
    }
}

pub fn get_tile_at_screen_pos(
    location: Vec2,
    camera: Query<(&Camera, &GlobalTransform)>,
) -> Option<(TileLoc, TileLoc)> {
    let (camera, camera_transform) = camera.single();
    if let Some(mouse_world_location) = camera.viewport_to_world(camera_transform, location) {
        let (sx, sy) = (mouse_world_location.origin.x, mouse_world_location.origin.y);
        let x = sx / TILE_SIZE + sy / (TILE_SIZE / 2.);
        let y = sx / TILE_SIZE - sy / (TILE_SIZE / 2.);
        Some((x.round() as TileLoc, y.round() as TileLoc))
    } else {
        None
    }
}

pub fn get_world_pos_for_tile(x: TileLoc, y: TileLoc) -> (f32, f32) {
    let base_x = (x as f32 + y as f32) * (TILE_SIZE / 2.);
    let base_y = (x as f32 - y as f32) * (TILE_SIZE / 4.);
    (base_x, base_y)
}
