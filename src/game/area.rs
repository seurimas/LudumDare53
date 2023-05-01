use std::convert;

use crate::prelude::*;

use super::player;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Follower {
    pub sign_holder: bool,
    pub corrupted: bool,
    #[serde(skip)]
    pub fleeing: Option<(u32, u32)>,
    pub affinity: Option<PlayerId>,
    pub power: u32,
}

impl Follower {
    pub fn new(power: u32) -> Self {
        Self {
            sign_holder: false,
            corrupted: false,
            fleeing: None,
            affinity: None,
            power,
        }
    }
}

pub const SIGN_HOLDER_MINIMUM: u32 = 10;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldArea {
    pub name: String,
    pub world_position: (u32, u32),
    pub nearest_neighbors: Vec<(u32, u32)>,
    pub followers: Vec<Follower>,
    pub agents: Vec<Agent>,
}

impl WorldArea {
    pub fn new(name: &str, x: u32, y: u32) -> Self {
        WorldArea {
            name: name.to_string(),
            world_position: (x, y),
            nearest_neighbors: Vec::new(),
            followers: Vec::new(),
            agents: Vec::new(),
        }
    }

    pub fn add_follower(&mut self, follower: Follower) {
        self.followers.push(follower);
    }

    pub fn flee(&mut self) -> Vec<Follower> {
        let fleeing = self
            .followers
            .iter()
            .filter(|f| f.fleeing.is_some())
            .cloned()
            .collect();
        self.followers.retain(|f| f.fleeing.is_none());
        fleeing
    }

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.push(agent);
    }

    pub fn get_agent_powers(&self) -> HashMap<PlayerId, u32> {
        self.agents.iter().map(|a| (a.id.player, a.power)).collect()
    }

    pub fn get_agent_mut(&mut self, agent_id: AgentId) -> Option<&mut Agent> {
        self.agents.iter_mut().find(|a| a.id == agent_id)
    }

    pub fn get_agent_power(&self, agent_id: AgentId) -> Option<u32> {
        self.agents
            .iter()
            .find(|a| a.id == agent_id)
            .map(|a| a.power)
    }

    pub fn get_possible_sign_holder_count(&self, agent_id: AgentId) -> usize {
        self.followers
            .iter()
            .filter(can_be_sign_holder(agent_id))
            .count()
    }

    pub fn get_agent_stamina(&self, agent_id: AgentId) -> Option<u32> {
        self.agents
            .iter()
            .find(|a| a.id == agent_id)
            .map(|a| a.stamina)
    }

    pub fn remove_agent(&mut self, agent_id: AgentId) -> Agent {
        let agent = self
            .agents
            .iter()
            .find(|a| a.id == agent_id)
            .unwrap()
            .clone();
        self.agents.retain(|a| a.id != agent_id);
        agent
    }

    pub fn promote_follower(
        &mut self,
        rng: &mut StdRng,
        player: PlayerId,
        agent_count: usize,
    ) -> Option<(AgentId, String)> {
        if let Some(power) = {
            let mut promotable_followers_count = self
                .followers
                .iter_mut()
                .filter(|f| {
                    f.power > SIGN_HOLDER_MINIMUM
                        && f.affinity == Some(player)
                        && !f.corrupted
                        && !f.sign_holder
                })
                .count();
            let mut promotable_followers = self.followers.iter_mut().filter(|f| {
                f.power > SIGN_HOLDER_MINIMUM
                    && f.affinity == Some(player)
                    && !f.corrupted
                    && !f.sign_holder
            });
            let promoted = choose_mut_iter(rng, promotable_followers, promotable_followers_count)?;
            let new_power = promoted.power;
            promoted.power = 0;
            Some(new_power)
        } {
            let agent_name = generate_agent_name(rng);
            self.add_agent(Agent::new(
                agent_name.clone(),
                AgentId {
                    player,
                    agent: agent_count as u32,
                },
                (0, 0),
                power,
            ));
            Some((
                AgentId {
                    player,
                    agent: agent_count as u32,
                },
                agent_name,
            ))
        } else {
            None
        }
    }

    pub fn brutalize_locals(&mut self, agent_id: AgentId, rng: &mut StdRng) -> (u32, u32) {
        let agent_powers = self.get_agent_powers();
        let agent = self.agents.iter_mut().find(|a| a.id == agent_id);
        if agent.is_none() {
            return (0, 0);
        }
        let agent = agent.unwrap();
        let mut player_power = self
            .followers
            .iter()
            .map(|follower| {
                if follower.affinity == Some(agent.id.player) {
                    follower.power
                } else {
                    0
                }
            })
            .sum::<u32>()
            + agent.power;
        let mut attacks_left = agent.stamina / 10;
        let mut swayed = 0;
        let mut flee = 0;
        while attacks_left > 0 && player_power > 0 {
            attacks_left -= 1;
            let non_player_local_count = self
                .followers
                .iter()
                .filter(|follower| follower.affinity != Some(agent.id.player))
                .count();
            if non_player_local_count == 0 {
                break;
            }
            let local = choose_mut_iter(
                rng,
                self.followers
                    .iter_mut()
                    .filter(|follower| follower.affinity != Some(agent.id.player)),
                non_player_local_count,
            )
            .unwrap();
            player_power = player_power.saturating_sub(local.power);
            if let Some(other_player) = local.affinity {
                if agent_powers.contains_key(&other_player) {
                    player_power = player_power.saturating_sub(agent_powers[&other_player]);
                } else {
                    local.fleeing = walking_choose(rng, self.nearest_neighbors.as_slice());
                    flee += 1;
                }
            } else if rng.gen_bool((player_power as f64 / 100.).clamp(0.1, 0.9)) {
                local.power /= 2;
                local.affinity = Some(agent.id.player);
                swayed += 1;
            } else if !local.sign_holder {
                local.fleeing = walking_choose(rng, self.nearest_neighbors.as_slice());
                local.power = local.power.saturating_sub(player_power / 2).clamp(4, 8);
                flee += 1;
            } else {
                // Continue to attack harder, despite the sign holder.
                player_power += local.power;
            }
        }
        for _ in 0..swayed {
            let follower_count = self
                .followers
                .iter()
                .filter(|follower| follower.affinity == Some(agent.id.player))
                .count();
            if follower_count == 0 {
                break;
            }
            let follower = choose_mut_iter(
                rng,
                self.followers
                    .iter_mut()
                    .filter(|follower| follower.affinity == Some(agent.id.player)),
                follower_count,
            );
            follower.unwrap().power += rng.gen_range(2..=5);
            agent.power += rng.gen_range(1..=3);
        }
        (swayed, flee)
    }

    pub fn sacrifice_followers(&mut self, agent_id: AgentId, rng: &mut StdRng) -> (u32, u32) {
        let agent = self.agents.iter_mut().find(|a| a.id == agent_id);
        if agent.is_none() {
            return (0, 0);
        }
        let agent = agent.unwrap();
        let player_followers = self
            .followers
            .iter_mut()
            .filter(can_be_sign_holder_mut(agent_id))
            .count();
        let my_followers = self
            .followers
            .iter_mut()
            .filter(can_be_sign_holder_mut(agent_id));
        if player_followers == 0 {
            return (0, 0);
        }
        let signs = {
            let sacrificed_follower = choose_mut_iter(rng, my_followers, player_followers).unwrap();
            if sacrificed_follower.sign_holder {
                agent.power += sacrificed_follower.power;
                agent.signs += 1;
                sacrificed_follower.power = 0;
                1
            } else {
                agent.power += sacrificed_follower.power / 2;
                sacrificed_follower.power = 0;
                0
            }
        };
        let mut fleeing = 0;
        // Send other sign holders fleeing.
        for follower in self.followers.iter_mut() {
            if follower.affinity != Some(agent_id.player) && follower.sign_holder {
                if rng.gen_bool(0.33 + signs as f64 * 0.33) {
                    follower.fleeing = walking_choose(rng, self.nearest_neighbors.as_slice());
                    if follower.fleeing.is_some() {
                        fleeing += 1;
                    }
                }
            }
        }
        (signs, fleeing)
    }

    pub fn corrupt_followers(&mut self, agent_id: AgentId, rng: &mut StdRng) -> (u32, u32) {
        let player_followers = self.get_player_followers(agent_id.player) as usize;
        let agent = self.agents.iter_mut().find(|a| a.id == agent_id);
        if agent.is_none() {
            return (0, 0);
        }
        let agent = agent.unwrap();
        let my_followers = self
            .followers
            .iter_mut()
            .filter(|follower| follower.affinity == Some(agent_id.player));
        if player_followers == 0 {
            return (0, 0);
        }
        let corrupted_follower = choose_mut_iter(rng, my_followers, player_followers).unwrap();
        if corrupted_follower.sign_holder {
            corrupted_follower.corrupted = true;
            corrupted_follower.power *= 10;
            agent.signs += 1;
            corrupted_follower.sign_holder = false;
            (1, 1)
        } else if rng.gen_bool(0.333) {
            corrupted_follower.power /= 10;
            (0, 0)
        } else {
            corrupted_follower.corrupted = true;
            corrupted_follower.power *= 10;
            (1, 0)
        }
    }

    pub fn corrupt_agent(&mut self, agent_id: AgentId, rng: &mut StdRng) -> (u32, u32) {
        let agent = self.agents.iter_mut().find(|a| a.id == agent_id);
        if agent.is_none() {
            return (0, 0);
        }
        let agent = agent.unwrap();
        let corrupted_follower = self
            .followers
            .iter_mut()
            .find(|follower| follower.affinity == Some(agent_id.player) && follower.corrupted);
        if let Some(corrupted_follower) = corrupted_follower {
            corrupted_follower.power = 0;
            if rng.gen_bool(0.333) {
                agent.power *= 10;
                agent.corrupted = true;
                agent.name = format!("Dark {}", agent.name);
                (1, 0)
            } else {
                (0, 0)
            }
        } else {
            (0, 0)
        }
    }

    pub fn can_prostelytize(&self, agent: &Agent) -> bool {
        self.followers.iter().any(|follower| {
            follower.power < agent.power
                && !follower.corrupted
                && follower.affinity != Some(agent.id.player)
        })
    }

    pub fn prostelytize_followers(
        &mut self,
        agent_id: AgentId,
        rng: &mut StdRng,
    ) -> (u32, u32, Option<PlayerId>) {
        let agent = self.agents.iter_mut().find(|a| a.id == agent_id);
        if agent.is_none() {
            return (0, 0, None);
        }
        let mut agent = agent.unwrap();
        agent.exhaust(agent.power);
        if self.followers.len() == 0 {
            return (0, 0, None);
        }
        let follower = choose_mut(rng, self.followers.as_mut_slice()).unwrap();
        if follower.affinity.is_none() {
            if follower.power < agent.power {
                follower.affinity = Some(agent_id.player);
                agent.power += (follower.power / 2).clamp(3, 10);
                return (1, 0, None);
            } else {
                agent.power += 3;
                return (0, 1, None);
            }
        } else if follower.affinity == Some(agent_id.player) {
            agent.power += (follower.power / 2).clamp(3, 10);
            follower.power += (agent.power / 5).clamp(1, 3);
            return (0, 0, None);
        } else {
            if rng.gen_range(0..=agent.power) > follower.power {
                // Double loss for stealing!
                agent.exhaust(follower.power);
                agent.power += (follower.power / 2).clamp(3, 5);
                let converted_player = follower.affinity;
                follower.affinity = Some(agent_id.player);
                return (1, 0, converted_player);
            } else {
                // Double loss for stealing, but don't end your turn!
                agent.exhaust(follower.power);
                return (0, 0, None);
            }
        }
    }

    pub fn get_player_agent_count(&self, player: PlayerId) -> u32 {
        self.agents.iter().filter(|a| a.id.player == player).count() as u32
    }

    pub fn get_nth_player_agent(&self, player: PlayerId, mut agent: u32) -> Option<&Agent> {
        let count = self.get_player_agent_count(player);
        while agent >= count && count != 0 {
            agent -= count;
        }
        self.agents
            .iter()
            .filter(|a| a.id.player == player)
            .nth(agent as usize)
    }

    pub fn get_unassigned_player_agent(&self, player_turn: &PlayerTurn) -> Option<u32> {
        self.agents
            .iter()
            .filter(|a| a.id.player == player_turn.player_id)
            .enumerate()
            .filter(|(_, a)| player_turn.is_unassigned_player_agent(a.id))
            .map(|(idx, _)| idx as u32)
            .next()
    }

    pub fn get_player_followers(&self, player: PlayerId) -> u32 {
        self.followers
            .iter()
            .filter(|f| f.affinity == Some(player))
            .count() as u32
    }

    pub fn get_player_power(&self, player: PlayerId) -> u32 {
        let follower_power: u32 = self
            .followers
            .iter()
            .filter(|f| f.affinity == Some(player))
            .map(|f| f.power)
            .sum();
        let agent_power: u32 = self
            .agents
            .iter()
            .filter(|a| a.id.player == player)
            .map(|f| f.power)
            .sum();
        follower_power + agent_power
    }

    pub fn get_non_player_follower_power(&self, player: PlayerId) -> u32 {
        self.followers
            .iter()
            .filter(|f| f.affinity.is_some() && f.affinity != Some(player))
            .map(|f| f.power)
            .sum()
    }

    pub fn get_non_player_agent_power(&self, player: PlayerId) -> u32 {
        self.agents
            .iter()
            .filter(|a| a.id.player != player)
            .map(|f| f.power)
            .sum()
    }

    pub fn corrupted_followers(&self, player: PlayerId) -> u32 {
        self.followers
            .iter()
            .filter(|f| f.affinity == Some(player) && f.corrupted)
            .count() as u32
    }

    pub fn corrupted_agents(&self, player: PlayerId) -> u32 {
        self.agents
            .iter()
            .filter(|a| a.id.player == player && a.corrupted)
            .count() as u32
    }

    pub fn corrupted_count(&self, player: PlayerId) -> u32 {
        self.corrupted_followers(player) + self.corrupted_agents(player)
    }

    pub fn get_value(&self) -> u32 {
        self.followers.iter().map(|f| f.power).sum()
    }

    pub fn player_agents(&self, player: PlayerId) -> impl Iterator<Item = &Agent> {
        self.agents.iter().filter(move |a| a.id.player == player)
    }

    pub fn sign_count(&self, player: PlayerId) -> u32 {
        self.agents
            .iter()
            .filter_map(|a| {
                if a.id.player == player {
                    Some(a.signs)
                } else {
                    None
                }
            })
            .sum()
    }
}

fn can_be_sign_holder_mut(agent_id: AgentId) -> impl Fn(&&mut Follower) -> bool {
    move |follower| {
        follower.affinity == Some(agent_id.player)
            && follower.power > SIGN_HOLDER_MINIMUM
            && !follower.corrupted
    }
}

fn can_be_sign_holder(agent_id: AgentId) -> impl Fn(&&Follower) -> bool {
    move |follower| {
        follower.affinity == Some(agent_id.player)
            && follower.power > SIGN_HOLDER_MINIMUM
            && !follower.corrupted
    }
}

pub struct AreaPlugin;

impl Plugin for AreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(render_area_ui.run_if(in_state(GameState::Playing)));
    }
}

fn render_area_ui(
    mut text_query: Query<(&Name, Option<&mut Text>, &mut Visibility)>,
    player: Res<PlayerId>,
    map_query: Query<(&MapTile, &WorldArea)>,
) {
    let mut area_selected = false;
    for (tile, area) in map_query.iter() {
        if tile.selected {
            area_selected = true;
            for (name, text, mut visibility) in text_query.iter_mut() {
                if name.eq_ignore_ascii_case("area_ui") {
                    *visibility = Visibility::Visible;
                } else if name.eq_ignore_ascii_case("area_name") {
                    text.unwrap().sections[0].value = area.name.clone();
                } else if name.eq_ignore_ascii_case("area_population") {
                    let all_followers = area.followers.len();
                    text.unwrap().sections[0].value = all_followers.to_string();
                } else if name.eq_ignore_ascii_case("area_total_power") {
                    let all_power = area.get_value();
                    text.unwrap().sections[0].value = all_power.to_string();
                } else if name.eq_ignore_ascii_case("area_followers") {
                    let player_followers = area.get_player_followers(*player);
                    text.unwrap().sections[0].value = player_followers.to_string();
                } else if name.eq_ignore_ascii_case("area_your_power") {
                    let player_power = area.get_player_power(*player);
                    text.unwrap().sections[0].value = player_power.to_string();
                } else if name.eq_ignore_ascii_case("area_corrupted") {
                    let player_corrupted = area.corrupted_followers(*player);
                    text.unwrap().sections[0].value = player_corrupted.to_string();
                }
            }
        }
    }
    if !area_selected {
        for (name, _, mut visibility) in text_query.iter_mut() {
            if name.eq_ignore_ascii_case("area_ui") {
                *visibility = Visibility::Hidden;
            }
        }
    }
}
