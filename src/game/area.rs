use std::convert;

use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Follower {
    pub sign_holder: bool,
    pub corrupted: bool,
    pub affinity: Option<PlayerId>,
    pub power: u32,
}

impl Follower {
    pub fn new(power: u32) -> Self {
        Self {
            sign_holder: false,
            corrupted: false,
            affinity: None,
            power,
        }
    }
}

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

    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.push(agent);
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

    pub fn sacrifice_followers(&mut self, agent_id: AgentId, rng: &mut StdRng) -> (u32, u32) {
        let agent = self.agents.iter_mut().find(|a| a.id == agent_id);
        if agent.is_none() {
            return (0, 0);
        }
        let agent = agent.unwrap();
        let player_followers = self
            .followers
            .iter_mut()
            .filter(can_be_sign_holder(agent_id))
            .count();
        let my_followers = self
            .followers
            .iter_mut()
            .filter(can_be_sign_holder(agent_id));

        let sacrificed_follower = choose_mut_iter(rng, my_followers, player_followers);
        if sacrificed_follower.sign_holder {
            agent.power += sacrificed_follower.power;
            agent.signs += 1;
            sacrificed_follower.power = 0;
            (1, 0)
        } else {
            agent.power += sacrificed_follower.power / 2;
            sacrificed_follower.power = 0;
            (0, 1)
        }
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
        let corrupted_follower = choose_mut_iter(rng, my_followers, player_followers);
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
        agent.exhaust(10);
        let follower = choose_mut(rng, self.followers.as_mut_slice());
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
                agent.power += (follower.power / 2).clamp(3, 5);
                let converted_player = follower.affinity;
                follower.affinity = Some(agent_id.player);
                return (1, 0, converted_player);
            } else {
                return (0, 1, None);
            }
        }
    }

    pub fn get_player_agent_count(&self, player: PlayerId) -> u32 {
        self.agents.iter().filter(|a| a.id.player == player).count() as u32
    }

    pub fn get_nth_player_agent(&self, player: PlayerId, agent: u32) -> Option<&Agent> {
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
        self.followers
            .iter()
            .filter(|f| f.affinity == Some(player))
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

fn can_be_sign_holder(agent_id: AgentId) -> impl Fn(&&mut Follower) -> bool {
    move |follower| follower.affinity == Some(agent_id.player) && follower.power > 10
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
                } else if name.eq_ignore_ascii_case("Area Name") {
                    text.unwrap().sections[0].value = area.name.clone();
                } else if name.eq_ignore_ascii_case("Area Population") {
                    let all_followers = area.followers.len();
                    text.unwrap().sections[0].value = all_followers.to_string();
                } else if name.eq_ignore_ascii_case("Area Value") {
                    let all_power = area.get_value();
                    text.unwrap().sections[0].value = all_power.to_string();
                } else if name.eq_ignore_ascii_case("Area Followers") {
                    let player_followers = area.get_player_followers(*player);
                    text.unwrap().sections[0].value = player_followers.to_string();
                } else if name.eq_ignore_ascii_case("Area Power") {
                    let player_power = area.get_player_power(*player);
                    text.unwrap().sections[0].value = player_power.to_string();
                } else if name.eq_ignore_ascii_case("Area Corrupted") {
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
