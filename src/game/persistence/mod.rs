use crate::prelude::*;

mod map_desc;
mod runes;
pub use map_desc::*;
pub use runes::*;

use super::turn_ui::{TurnReport, EVOKE_COLOR, TRANSPARENT_EVOKE_COLOR};

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(save_periodically.run_if(in_state(GameState::Playing)));

        #[cfg(target_arch = "wasm32")]
        app.add_system(add_save_button.in_schedule(OnEnter(GameState::Playing)));

        #[cfg(target_arch = "wasm32")]
        app.add_system(show_hide_save.run_if(in_state(GameState::Playing)));

        #[cfg(target_arch = "wasm32")]
        app.add_system(save_on_click.run_if(in_state(GameState::Playing)));

        #[cfg(target_arch = "wasm32")]
        app.add_system(wait_for_loads.in_schedule(OnEnter(GameState::MainMenu)));

        #[cfg(target_arch = "wasm32")]
        app.add_system(load_on_event.run_if(in_state(GameState::MainMenu)));
    }
}

#[derive(Component)]
struct SaveGameButton;

fn add_save_button(mut commands: Commands, assets: Res<MyAssets>) {
    // Evoke darkness.
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(ONE_UNIT),
                        right: Val::Px(ONE_UNIT),
                        ..default()
                    },
                    ..default()
                },
                background_color: Color::WHITE.into(),
                ..default()
            },
            SaveGameButton,
            RelativeCursorPosition::default(),
            SimpleTooltip::new("Export save file."),
            Name::new("Save Game"),
        ))
        .with_children(|parent| {
            parent.spawn((TextBundle {
                style: Style {
                    flex_shrink: 1.,
                    ..default()
                },
                text: Text::from_section(
                    "Save Game",
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 32.,
                        color: Color::BLACK,
                    },
                ),
                background_color: EVOKE_COLOR.into(),
                ..default()
            },));
        });
    // Evoke darkness - Inactive.
    commands
        .spawn((
            ButtonBundle {
                style: Style {
                    border: UiRect::all(Val::Px(ONE_UNIT)),
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Px(ONE_UNIT),
                        right: Val::Px(ONE_UNIT),
                        ..default()
                    },
                    ..default()
                },
                background_color: Color::BLACK.into(),
                ..default()
            },
            RelativeCursorPosition::default(),
            SimpleTooltip::new("You may save at the end of your turn."),
            Name::new("Save Game Inactive"),
        ))
        .with_children(|parent| {
            parent.spawn((TextBundle {
                style: Style {
                    flex_shrink: 1.,
                    ..default()
                },
                text: Text::from_section(
                    "Save Game",
                    TextStyle {
                        font: assets.font.clone(),
                        font_size: 32.,
                        color: Color::BLACK,
                    },
                ),
                background_color: TRANSPARENT_EVOKE_COLOR.into(),
                ..default()
            },));
        });
}

fn show_hide_save(
    player_turn: Res<PlayerTurn>,
    save_data: Option<Res<SaveData>>,
    mut button_query: Query<(&Name, &mut Visibility)>,
) {
    if save_data.is_none() {
        button_query.iter_mut().for_each(|(name, mut visibility)| {
            if name.eq_ignore_ascii_case("Save Game") {
                *visibility = Visibility::Hidden;
            } else if name.eq_ignore_ascii_case("Save Game Inactive") {
                *visibility = Visibility::Visible;
            }
        });
    } else {
        button_query.iter_mut().for_each(|(name, mut visibility)| {
            if name.eq_ignore_ascii_case("Save Game") {
                *visibility = Visibility::Visible;
            } else if name.eq_ignore_ascii_case("Save Game Inactive") {
                *visibility = Visibility::Hidden;
            }
        });
    }
}

#[cfg(target_arch = "wasm32")]
fn save_on_click(
    players: Res<GamePlayers>,
    player: Res<PlayerId>,
    save_data: Option<Res<SaveData>>,
    interactions: Query<&Interaction, (Changed<Interaction>, With<SaveGameButton>)>,
) {
    for interaction in interactions.iter() {
        if *interaction == Interaction::Clicked && save_data.is_some() {
            let data = save_data.as_ref().unwrap();
            let json = serde_json::to_string(data.as_ref()).unwrap();
            save_game_js(format!("{}.json", players.get_save_prefix(*player)), json);
        }
    }
}

#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

#[cfg(target_arch = "wasm32")]
lazy_static! {
    static ref LOAD_STRING: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
}

#[cfg(target_arch = "wasm32")]
fn wait_for_loads() {
    use wasm_bindgen::prelude::*;
    let closure = Closure::new(move |s: String| {
        *LOAD_STRING.lock().unwrap() = Some(s.to_string());
        ()
    });
    set_loader(&closure);
    closure.forget();
}

#[cfg(target_arch = "wasm32")]
fn load_on_event(mut commands: Commands, mut next_state: ResMut<NextState<GameState>>) {
    if let Some(save) = LOAD_STRING.lock().unwrap().take() {
        let save: SaveData = serde_json::from_str(&save).unwrap();
        commands.insert_resource(save.players.clone());
        commands.insert_resource(save.player_id.clone());
        commands.insert_resource(save.ai_seeds.clone());
        commands.insert_resource(save.map_desc);
        commands.insert_resource(save.turn_report);
        if let Some(evokation) = save
            .last_evokation
            .and_then(|evokation| read_from_runes::<Evokation>(&evokation, false))
        {
            if evokation.season == *save.season {
                commands.insert_resource(EvokingState::resume(evokation.clone(), &save.players));
                commands.insert_resource(evokation.player_turn);
            } else {
                commands.insert_resource(EvokingState::None {
                    last_evokation: Some(evokation),
                });
                commands.insert_resource(PlayerTurn::new(save.player_id));
            }
        } else {
            commands.insert_resource(PlayerTurn::new(save.player_id));
        }
        commands.insert_resource(save.season);
        next_state.set(GameState::Playing);
        hide_load();
    }
}

#[derive(Resource, Serialize, Deserialize)]
pub struct SaveData {
    pub season: Season,
    pub ai_seeds: AiSeeds,
    pub players: GamePlayers,
    pub player_id: PlayerId,
    pub map_desc: MapDesc,
    pub last_evokation: Option<String>,
    pub turn_report: TurnReport,
}

fn describe_map(tile_query: &Query<(&MapTile, Option<&WorldArea>)>) -> MapDesc {
    let width = tile_query.iter().map(|(tile, _)| tile.x).max().unwrap_or(0) as u32 + 1;
    let height = tile_query.iter().map(|(tile, _)| tile.y).max().unwrap_or(0) as u32 + 1;
    let mut tiles = vec![0; (width * height) as usize];
    let mut areas = Vec::new();
    for (tile, area) in tile_query.iter() {
        if tile.x < 0 || tile.y < 0 {
            continue;
        } else if tile.x >= width as i32 || tile.y >= height as i32 {
            continue;
        }
        tiles[(tile.y as u32 * width + tile.x as u32) as usize] = tile.sprite_id;
        if let Some(area) = area {
            areas.push(area.clone());
        }
    }
    MapDesc {
        width,
        height,
        tiles,
        areas,
    }
}

fn save_periodically(
    mut last_season: Local<(i32, bool)>,
    player_turn: Res<PlayerTurn>,
    season: Res<Season>,
    ai_seeds: Res<AiSeeds>,
    players: Res<GamePlayers>,
    player: Res<PlayerId>,
    evokation: Res<EvokingState>,
    turn_report: Res<TurnReport>,
    tile_query: Query<(&MapTile, Option<&WorldArea>)>,
    #[cfg(target_arch = "wasm32")] mut commands: Commands,
) {
    if last_season.0 != season.0 || last_season.1 != evokation.is_evoking() {
        last_season.0 = season.0;
        last_season.1 = evokation.is_evoking();
        let map_desc = describe_map(&tile_query);
        let save_data = SaveData {
            season: *season,
            ai_seeds: ai_seeds.clone(),
            players: players.clone(),
            player_id: *player,
            map_desc,
            last_evokation: evokation.get_evokation(&*player).map(|e| e.to_runes(false)),
            turn_report: turn_report.clone(),
        };
        let default = "Unknown".to_string();
        #[cfg(not(target_arch = "wasm32"))]
        save(
            format!("{}.json", players.get_save_prefix(*player)),
            save_data,
        );
        #[cfg(target_arch = "wasm32")]
        commands.insert_resource(save_data);
    }
}

fn save<T: Serialize>(name: impl ToString, data: T) {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(format!("{}.json", name.to_string())).unwrap();
    let json = serde_json::to_string(&data).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
