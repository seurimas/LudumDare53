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
    pub world_position: (usize, usize),
    pub nearest_neighbors: Vec<(usize, usize)>,
    pub followers: Vec<Follower>,
    pub agents: Vec<Agent>,
}

impl WorldArea {
    pub fn new(name: &str, x: usize, y: usize) -> Self {
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

    pub fn corrupt_followers(&mut self, agent_id: AgentId) -> (u32, u32) {
        if let Some(follower) = self.followers.iter_mut().find(|follower| {
            follower.power > 5 && !follower.corrupted && follower.affinity == Some(agent_id.player)
        }) {
            if follower.power > 7 {
                follower.corrupted = true;
                follower.power *= 10;
                (1, 0)
            } else {
                follower.power /= 2;
                (0, 1)
            }
        } else {
            (0, 0)
        }
    }

    pub fn prostelytize_followers(&mut self, agent_id: AgentId) -> (u32, u32) {
        let agent_power = self.get_agent_power(agent_id).unwrap_or(0);
        if let Some(follower) = self.followers.iter_mut().find(|follower| {
            follower.power < agent_power
                && !follower.corrupted
                && follower.affinity != Some(agent_id.player)
        }) {
            follower.affinity = Some(agent_id.player);
            self.get_agent_mut(agent_id).map(|agent| agent.exhaust(10));
            (1, 0)
        } else if let Some(follower) = self.followers.iter_mut().find(|follower| {
            follower.power > agent_power
                && !follower.corrupted
                && follower.affinity != Some(agent_id.player)
                && follower.affinity.is_some()
        }) {
            if follower.power >= 2 {
                follower.power -= 2;
            }
            self.get_agent_mut(agent_id).map(|agent| {
                agent.exhaust(10);
                if agent.power >= 2 {
                    agent.power -= 2;
                }
            });
            (0, 1)
        } else {
            (1, 0)
        }
    }

    pub fn get_player_agent_count(&self, player: PlayerId) -> u32 {
        self.agents.iter().filter(|a| a.id.player == player).count() as u32
    }

    pub fn get_nth_player_agent(&self, player: PlayerId, agent: usize) -> Option<&Agent> {
        self.agents
            .iter()
            .filter(|a| a.id.player == player)
            .nth(agent)
    }

    pub fn get_unassigned_player_agent(&self, player_turn: &PlayerTurn) -> Option<usize> {
        self.agents
            .iter()
            .filter(|a| a.id.player == player_turn.player_id)
            .enumerate()
            .filter(|(_, a)| player_turn.is_unassigned_player_agent(a.id))
            .map(|(idx, _)| idx)
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

    pub fn get_value(&self) -> u32 {
        self.followers.iter().map(|f| f.power).sum()
    }

    pub fn get_player_corrupted(&self, player: PlayerId) -> u32 {
        self.followers
            .iter()
            .filter(|f| f.affinity == Some(player) && f.corrupted)
            .count() as u32
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
                    let player_corrupted = area.get_player_corrupted(*player);
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
