use crate::prelude::*;

use super::turn_ui::TurnReportEvent;

#[derive(Resource, Deref, DerefMut)]
pub struct Season(pub i32);

pub struct TurnResults {
    pub report: Vec<TurnReportEvent>,
    pub new_world_areas: HashMap<(i32, i32), WorldArea>,
}

impl TurnResults {
    pub fn get_new_world_area(&self, position: (i32, i32)) -> Option<WorldArea> {
        self.new_world_areas.get(&position).cloned()
    }
}

pub fn apply_turns(
    season: i32,
    reporting_player: PlayerId,
    turns: Vec<PlayerTurn>,
    world_areas: Vec<WorldArea>,
) -> TurnResults {
    let mut report = Vec::new();
    let mut new_world_areas = HashMap::new();
    let mut agents = HashMap::new();
    for area in world_areas {
        agents.extend(area.agents.iter().map(|a| (a.id, a.clone())));
        new_world_areas.insert(area.world_position, area);
    }

    let moved_agents = move_agents(&turns, &mut new_world_areas);
    report.extend(moved_agents.iter().flat_map(|(x, y, agent_id)| {
        if reporting_player == agent_id.player {
            Some(TurnReportEvent::AgentAction {
                location: (*x, *y),
                location_name: new_world_areas[&(*x, *y)].name.clone(),
                agent_name: agents[&agent_id].name.clone(),
                action: AgentAction::Move(0),
                success_amount: 0,
                fail_amount: 0,
            })
        } else {
            None
        }
    }));

    let corruptions = corrupt_followers(&turns, &mut new_world_areas);
    report.extend(
        corruptions
            .iter()
            .flat_map(|(x, y, agent_id, success_amount, fail_amount)| {
                if reporting_player == agent_id.player {
                    Some(TurnReportEvent::AgentAction {
                        location: (*x, *y),
                        location_name: new_world_areas[&(*x, *y)].name.clone(),
                        agent_name: agents[&agent_id].name.clone(),
                        action: AgentAction::Corrupt,
                        success_amount: *success_amount,
                        fail_amount: *fail_amount,
                    })
                } else {
                    None
                }
            }),
    );

    let converts = prostelytize_followers(&turns, &mut new_world_areas);
    report.extend(
        converts
            .iter()
            .flat_map(|(x, y, agent_id, success_amount, fail_amount)| {
                if reporting_player == agent_id.player {
                    Some(TurnReportEvent::AgentAction {
                        location: (*x, *y),
                        location_name: new_world_areas[&(*x, *y)].name.clone(),
                        agent_name: agents[&agent_id].name.clone(),
                        action: AgentAction::Prostelytize,
                        success_amount: *success_amount,
                        fail_amount: *fail_amount,
                    })
                } else {
                    None
                }
            }),
    );

    if let Some(winner) = check_winners(&new_world_areas) {
        report.push(TurnReportEvent::GameOver {
            winner,
            scores: get_scores(&new_world_areas),
        });
    } else {
        report.push(TurnReportEvent::NewTurn { turn: season });
    }

    TurnResults {
        report,
        new_world_areas,
    }
}

fn get_agent_location(
    world_areas: &HashMap<(i32, i32), WorldArea>,
    agent_id: AgentId,
) -> Option<(i32, i32)> {
    for area in world_areas.values() {
        for agent in &area.agents {
            if agent.id == agent_id {
                return Some(area.world_position);
            }
        }
    }
    None
}

fn move_agents(
    turns: &Vec<PlayerTurn>,
    world_areas: &mut HashMap<(i32, i32), WorldArea>,
) -> Vec<(i32, i32, AgentId)> {
    let mut results = Vec::new();
    for turn in turns {
        let movement_actions = turn.actions.iter().filter_map(|(agent_id, action)| {
            if let AgentAction::Move(idx) = action {
                Some((agent_id, idx))
            } else {
                None
            }
        });
        for (agent_id, neighbor_idx) in movement_actions {
            if let Some(source) = get_agent_location(&world_areas, *agent_id) {
                let agent = world_areas
                    .get_mut(&source)
                    .unwrap()
                    .remove_agent(*agent_id);
                let target = world_areas[&source].nearest_neighbors[*neighbor_idx];
                world_areas.get_mut(&target).unwrap().add_agent(agent);
                results.push((source.0, source.1, *agent_id));
            }
        }
    }
    results
}

fn corrupt_followers(
    turns: &Vec<PlayerTurn>,
    world_areas: &mut HashMap<(i32, i32), WorldArea>,
) -> Vec<(i32, i32, AgentId, u32, u32)> {
    let mut results = Vec::new();
    for turn in turns {
        let corruption_actions = turn.actions.iter().filter_map(|(agent_id, action)| {
            if let AgentAction::Corrupt = action {
                Some(agent_id)
            } else {
                None
            }
        });
        for agent_id in corruption_actions {
            if let Some(source) = get_agent_location(&world_areas, *agent_id) {
                let (success_amount, fail_amount) = world_areas
                    .get_mut(&source)
                    .unwrap()
                    .corrupt_followers(*agent_id);
                results.push((source.0, source.1, *agent_id, success_amount, fail_amount));
            }
        }
    }
    results
}

fn prostelytize_followers(
    turns: &Vec<PlayerTurn>,
    world_areas: &mut HashMap<(i32, i32), WorldArea>,
) -> Vec<(i32, i32, AgentId, u32, u32)> {
    let mut results = Vec::new();
    for turn in turns {
        let corruption_actions = turn.actions.iter().filter_map(|(agent_id, action)| {
            if let AgentAction::Prostelytize = action {
                Some(agent_id)
            } else {
                None
            }
        });
        for agent_id in corruption_actions {
            if let Some(source) = get_agent_location(&world_areas, *agent_id) {
                if let Some(area) = world_areas.get_mut(&source) {
                    let mut successes = 0;
                    let mut failures = 0;
                    while area.get_agent_stamina(*agent_id).unwrap_or_default() > 0 && failures == 0
                    {
                        let (success_amount, fail_amount) = area.prostelytize_followers(*agent_id);
                        successes += success_amount;
                        failures += fail_amount;
                        if success_amount == 0 && fail_amount == 0 {
                            break;
                        }
                    }
                    results.push((source.0, source.1, *agent_id, successes, failures));
                }
            }
        }
    }
    results
}

fn check_winners(world_areas: &HashMap<(i32, i32), WorldArea>) -> Option<PlayerId> {
    let scores = get_scores(world_areas);
    let max_score = scores.values().max().unwrap_or(&0);
    let winners: Vec<PlayerId> = scores
        .iter()
        .filter(|(_, score)| *score == max_score)
        .map(|(player, _)| *player)
        .collect();
    if winners.len() == 1 {
        None
    } else {
        winners.get(0).copied()
    }
}

fn get_scores(world_areas: &HashMap<(i32, i32), WorldArea>) -> HashMap<PlayerId, u32> {
    let mut scores = HashMap::new();
    for area in world_areas.values() {
        for agent in &area.agents {
            let score = scores.entry(agent.id.player).or_insert(0);
            *score += agent.signs;
        }
    }
    scores
}
