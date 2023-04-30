use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::prelude::*;

pub struct AiPlugin;

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(ai_turn.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Resource, Serialize, Deserialize)]
pub struct AiSeeds {
    pub seeds: Vec<u64>,
}

fn get_ai_rng(game_players: &GamePlayers, player: PlayerId, seeds: &AiSeeds) -> StdRng {
    let seed_id = game_players.get_ai_seed_index(player).unwrap();
    let seed = seeds.seeds[seed_id];
    StdRng::seed_from_u64(seed)
}

fn get_action(rng: &mut StdRng, agent: &Agent, area: &WorldArea) -> AgentAction {
    if AgentAction::Sacrifice
        .invalid_reasons(agent, area)
        .is_none()
    {
        AgentAction::Sacrifice
    } else if AgentAction::Corrupt.invalid_reasons(agent, area).is_none() {
        AgentAction::Corrupt
    } else if AgentAction::Brutalize
        .invalid_reasons(agent, area)
        .is_none()
    {
        AgentAction::Brutalize
    } else if AgentAction::Prostelytize
        .invalid_reasons(agent, area)
        .is_none()
    {
        AgentAction::Prostelytize
    } else if let Some(target_location) = walking_choose(rng, &area.nearest_neighbors) {
        AgentAction::Move(target_location.0, target_location.1, "???".to_string())
    } else {
        // Unlikely players, AI can just... do nothing.
        AgentAction::None
    }
}

pub fn generate_seeds(player_names: Vec<String>, ai_players: u32) -> AiSeeds {
    let mut hasher = DefaultHasher::new();
    player_names.iter().for_each(|name| name.hash(&mut hasher));
    let seeds_seed = hasher.finish();
    let mut rng = StdRng::seed_from_u64(seeds_seed);
    let mut seeds = Vec::new();
    for _ in 0..ai_players {
        seeds.push(rng.gen());
    }
    AiSeeds { seeds }
}

fn take_turn(player: PlayerId, rng: &mut StdRng, world_areas: &Query<&WorldArea>) -> PlayerTurn {
    let mut turn = PlayerTurn::new(player);
    for area in world_areas.iter() {
        for agent in area.player_agents(player) {
            let action = get_action(rng, agent, area);
            turn.set_action(agent.id, action);
        }
    }
    turn
}

fn ai_turn(
    mut cooldown: Local<f32>,
    time: Res<Time>,
    game_players: Res<GamePlayers>,
    mut ai_seeds: ResMut<AiSeeds>,
    mut evoking: ResMut<EvokingState>,
    world_areas: Query<&WorldArea>,
) {
    if let Some((season, seed, turn)) = match evoking.as_mut() {
        EvokingState::Evoking { season, evoked, .. } => {
            if *cooldown <= 0.0 {
                *cooldown = 0.5;
                let mut evokation = None;
                for player in game_players.get_ids() {
                    if game_players.is_ai(player) && !evoked.contains_key(&player) {
                        let mut rng = get_ai_rng(&game_players, player, &ai_seeds);
                        let ai_turn = take_turn(player, &mut rng, &world_areas);
                        evokation = Some((*season, rng.gen(), ai_turn))
                    }
                }
                evokation
            } else {
                *cooldown -= time.delta_seconds();
                None
            }
        }
        _ => None,
    } {
        evoking.push(season, seed, turn);
    }
}
