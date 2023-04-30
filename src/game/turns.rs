use crate::prelude::*;

use super::{turn_ui::TurnReportEvent, WIN_SIGN_COUNT};

#[derive(Resource, Deref, DerefMut)]
pub struct Season(pub i32);

pub struct TurnResults {
    pub report: Vec<TurnReportEvent>,
    pub new_world_areas: HashMap<(u32, u32), WorldArea>,
}

impl TurnResults {
    pub fn get_new_world_area(&self, position: (u32, u32)) -> Option<WorldArea> {
        self.new_world_areas.get(&position).cloned()
    }
}

pub fn apply_turns(
    season: i32,
    reporting_player: PlayerId,
    mut turns: Vec<PlayerTurn>,
    seeds: Vec<u64>,
    world_areas: Vec<WorldArea>,
) -> TurnResults {
    let mut report = Vec::new();
    let mut new_world_areas = HashMap::new();
    let mut agents = HashMap::new();
    for area in world_areas {
        agents.extend(area.agents.iter().map(|a| (a.id, a.clone())));
        new_world_areas.insert(area.world_position, area);
    }
    turns.sort_by(|a, b| a.player_id.cmp(&b.player_id));
    let mut rngs = seeds
        .iter()
        .map(|seed| StdRng::seed_from_u64(*seed))
        .collect::<Vec<_>>();

    let moved_agents = move_agents(&turns, &mut new_world_areas);
    report.extend(moved_agents.iter().flat_map(|(x, y, agent_id)| {
        if reporting_player == agent_id.player {
            Some(TurnReportEvent::AgentAction {
                location: (*x, *y),
                location_name: new_world_areas[&(*x, *y)].name.clone(),
                agent_name: agents[&agent_id].name.clone(),
                action: AgentAction::Move(*x, *y, new_world_areas[&(*x, *y)].name.clone()),
                success_amount: 0,
                fail_amount: 0,
            })
        } else {
            None
        }
    }));

    let corruptions = corrupt_followers(&turns, &mut rngs, &mut new_world_areas);
    report.extend(
        corruptions
            .iter()
            .flat_map(|(x, y, agent_id, success_amount, signs_seen)| {
                if reporting_player == agent_id.player {
                    let mut events = vec![TurnReportEvent::AgentAction {
                        location: (*x, *y),
                        location_name: new_world_areas[&(*x, *y)].name.clone(),
                        agent_name: agents[&agent_id].name.clone(),
                        action: AgentAction::Corrupt,
                        success_amount: *success_amount,
                        fail_amount: *signs_seen,
                    }];
                    if *signs_seen > 0 {
                        events.push(TurnReportEvent::SignSeen {
                            location: (*x, *y),
                            location_name: new_world_areas[&(*x, *y)].name.clone(),
                            mine: true,
                        });
                    }
                    events
                } else if *signs_seen > 0 {
                    vec![TurnReportEvent::SignSeen {
                        location: (*x, *y),
                        location_name: new_world_areas[&(*x, *y)].name.clone(),
                        mine: false,
                    }]
                } else {
                    vec![]
                }
            }),
    );

    let (prostelyizes, converts) = prostelytize_followers(&turns, &mut rngs, &mut new_world_areas);
    report.extend(
        prostelyizes
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

    let sacrifices = single_action(
        &turns,
        &mut rngs,
        &mut new_world_areas,
        &|action| {
            if let AgentAction::Sacrifice = action {
                true
            } else {
                false
            }
        },
        &|world_area: &mut WorldArea, agent_id, rng| world_area.sacrifice_followers(agent_id, rng),
    );

    report.extend(
        sacrifices
            .iter()
            .flat_map(|(x, y, agent_id, success_amount, fail_amount)| {
                if reporting_player == agent_id.player {
                    Some(TurnReportEvent::AgentAction {
                        location: (*x, *y),
                        location_name: new_world_areas[&(*x, *y)].name.clone(),
                        agent_name: agents[&agent_id].name.clone(),
                        action: AgentAction::Sacrifice,
                        success_amount: *success_amount,
                        fail_amount: *fail_amount,
                    })
                } else if *fail_amount > 0 {
                    Some(TurnReportEvent::Sacrificed {
                        location: (*x, *y),
                        location_name: new_world_areas[&(*x, *y)].name.clone(),
                        follower: false,
                    })
                } else {
                    None
                }
            }),
    );

    report.extend(sacrifices.iter().flat_map(|(x, y, agent_id, signs, _)| {
        if *signs > 0 {
            Some(TurnReportEvent::SignSeen {
                location: (*x, *y),
                location_name: new_world_areas[&(*x, *y)].name.clone(),
                mine: reporting_player == agent_id.player,
            })
        } else {
            None
        }
    }));

    let brutalities = single_action(
        &turns,
        &mut rngs,
        &mut new_world_areas,
        &|action| {
            if let AgentAction::Brutalize = action {
                true
            } else {
                false
            }
        },
        &|world_area: &mut WorldArea, agent_id, rng| world_area.brutalize_locals(agent_id, rng),
    );

    report.extend(
        brutalities
            .iter()
            .flat_map(|(x, y, agent_id, success_amount, fail_amount)| {
                if reporting_player == agent_id.player {
                    Some(TurnReportEvent::AgentAction {
                        location: (*x, *y),
                        location_name: new_world_areas[&(*x, *y)].name.clone(),
                        agent_name: agents[&agent_id].name.clone(),
                        action: AgentAction::Brutalize,
                        success_amount: *success_amount,
                        fail_amount: *fail_amount,
                    })
                } else if *fail_amount > 0
                    && new_world_areas[&(*x, *y)].get_player_power(reporting_player) > 0
                {
                    Some(TurnReportEvent::Brutalized {
                        location: (*x, *y),
                        location_name: new_world_areas[&(*x, *y)].name.clone(),
                        dead: *success_amount,
                        fleeing: *fail_amount,
                    })
                } else {
                    None
                }
            }),
    );

    report.extend(converts.iter().flat_map(|(player, x, y)| {
        if reporting_player == *player {
            Some(TurnReportEvent::FollowersLost {
                location: (*x, *y),
                location_name: new_world_areas[&(*x, *y)].name.clone(),
            })
        } else {
            None
        }
    }));

    if let Some(winner) = check_winners(&new_world_areas) {
        report.push(TurnReportEvent::GameOver {
            winner,
            scores: get_scores(&new_world_areas),
        });
    } else {
        report.push(TurnReportEvent::NewTurn { turn: season });
    }

    // Reset stamina.
    for area in new_world_areas.values_mut() {
        for agent in &mut area.agents {
            agent.stamina = 100 + agent.power;
        }
        // Kill all the dead followers.
        area.followers.retain(|follower| follower.power > 0);
    }
    let locations: Vec<(u32, u32)> = new_world_areas.keys().cloned().collect();
    for location in locations {
        let fleeing = new_world_areas.get_mut(&location).unwrap().flee();
        for follower in fleeing {
            new_world_areas
                .get_mut(&location)
                .unwrap()
                .add_follower(follower);
        }
    }

    TurnResults {
        report,
        new_world_areas,
    }
}

fn get_agent_location(
    world_areas: &HashMap<(u32, u32), WorldArea>,
    agent_id: AgentId,
) -> Option<(u32, u32)> {
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
    world_areas: &mut HashMap<(u32, u32), WorldArea>,
) -> Vec<(u32, u32, AgentId)> {
    let mut results = Vec::new();
    for turn in turns {
        let mut movement_actions = turn
            .actions
            .iter()
            .filter_map(|(agent_id, action)| {
                if let AgentAction::Move(x, y, _) = action {
                    Some((agent_id, x, y))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        movement_actions.sort_by(|a, b| {
            let a = *a.0;
            let b = *b.0;
            a.cmp(&b)
        });
        for (agent_id, x, y) in movement_actions {
            if let Some(source) = get_agent_location(&world_areas, *agent_id) {
                if !world_areas.contains_key(&(*x, *y)) {
                    continue;
                }
                let agent = world_areas
                    .get_mut(&source)
                    .unwrap()
                    .remove_agent(*agent_id);
                world_areas.get_mut(&(*x, *y)).unwrap().add_agent(agent);
                results.push((*x, *y, *agent_id));
            }
        }
    }
    results
}

fn corrupt_followers(
    turns: &Vec<PlayerTurn>,
    rngs: &mut Vec<StdRng>,
    world_areas: &mut HashMap<(u32, u32), WorldArea>,
) -> Vec<(u32, u32, AgentId, u32, u32)> {
    let mut results = Vec::new();
    for (turn, rng) in turns.iter().zip(rngs.iter_mut()) {
        let mut corruption_actions = turn
            .actions
            .iter()
            .filter_map(|(agent_id, action)| {
                if let AgentAction::Corrupt = action {
                    Some(agent_id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        corruption_actions.sort_by(|a, b| a.player.cmp(&b.player));
        for agent_id in corruption_actions {
            if let Some(source) = get_agent_location(&world_areas, *agent_id) {
                let (success_amount, signs_seen) = world_areas
                    .get_mut(&source)
                    .unwrap()
                    .corrupt_followers(*agent_id, rng);
                results.push((source.0, source.1, *agent_id, success_amount, signs_seen));
            }
        }
    }
    results
}

fn single_action(
    turns: &Vec<PlayerTurn>,
    rngs: &mut Vec<StdRng>,
    world_areas: &mut HashMap<(u32, u32), WorldArea>,
    action_predicate: &dyn Fn(&AgentAction) -> bool,
    action_fn: &dyn Fn(&mut WorldArea, AgentId, &mut StdRng) -> (u32, u32),
) -> Vec<(u32, u32, AgentId, u32, u32)> {
    let mut results = Vec::new();
    for (turn, rng) in turns.iter().zip(rngs.iter_mut()) {
        let acting_agents = turn.actions.iter().filter_map(|(agent_id, action)| {
            if action_predicate(action) {
                Some(agent_id)
            } else {
                None
            }
        });
        for agent_id in acting_agents {
            if let Some(source) = get_agent_location(&world_areas, *agent_id) {
                let (success_amount, signs_seen) =
                    action_fn(world_areas.get_mut(&source).unwrap(), *agent_id, rng);
                results.push((source.0, source.1, *agent_id, success_amount, signs_seen));
            }
        }
    }
    results
}

fn prostelytize_followers(
    turns: &Vec<PlayerTurn>,
    rngs: &mut Vec<StdRng>,
    world_areas: &mut HashMap<(u32, u32), WorldArea>,
) -> (
    Vec<(u32, u32, AgentId, u32, u32)>,
    HashSet<(PlayerId, u32, u32)>,
) {
    let mut results = Vec::new();
    let mut converts: HashSet<(PlayerId, u32, u32)> = HashSet::new();
    for (turn, rng) in turns.iter().zip(rngs.iter_mut()) {
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
                        let (success_amount, fail_amount, converted) =
                            area.prostelytize_followers(*agent_id, rng);
                        successes += success_amount;
                        failures += fail_amount;
                        if let Some(converted) = converted {
                            converts.insert((converted, source.0, source.1));
                        }
                        if success_amount == 0 && fail_amount == 0 {
                            break;
                        }
                    }
                    results.push((source.0, source.1, *agent_id, successes, failures));
                }
            }
        }
    }
    (results, converts)
}

fn check_winners(world_areas: &HashMap<(u32, u32), WorldArea>) -> Option<PlayerId> {
    let scores = get_scores(world_areas);
    let max_score = scores.values().max().unwrap_or(&0);
    if *max_score < WIN_SIGN_COUNT {
        return None;
    }
    let winners: Vec<PlayerId> = scores
        .iter()
        .filter(|(_, score)| *score == max_score)
        .map(|(player, _)| *player)
        .collect();
    if winners.len() == 1 {
        winners.get(0).copied()
    } else {
        None
    }
}

fn get_scores(world_areas: &HashMap<(u32, u32), WorldArea>) -> HashMap<PlayerId, u32> {
    let mut scores = HashMap::new();
    for area in world_areas.values() {
        for agent in &area.agents {
            let score = scores.entry(agent.id.player).or_insert(0);
            *score += agent.signs;
        }
    }
    scores
}
