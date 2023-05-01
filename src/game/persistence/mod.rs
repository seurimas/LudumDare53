use crate::prelude::*;

mod map_desc;
mod runes;
pub use map_desc::*;
pub use runes::*;

use super::{darkness::Evokation, turn_ui::TurnReport};

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(save_periodically.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Serialize, Deserialize)]
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
    mut last_season: Local<i32>,
    season: Res<Season>,
    ai_seeds: Res<AiSeeds>,
    players: Res<GamePlayers>,
    player: Res<PlayerId>,
    evokation: Res<EvokingState>,
    turn_report: Res<TurnReport>,
    tile_query: Query<(&MapTile, Option<&WorldArea>)>,
) {
    if *last_season != season.0 {
        *last_season = season.0;
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
        save(
            format!("{}.json", players.get_save_prefix(*player)),
            save_data,
        );
    }
}

fn save<T: Serialize>(name: impl ToString, data: T) {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(format!("{}.json", name.to_string())).unwrap();
    let json = serde_json::to_string(&data).unwrap();
    file.write_all(json.as_bytes()).unwrap();
}
