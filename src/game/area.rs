use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Follower {
    pub sign_holder: bool,
    pub corrupted: bool,
    pub affinity: Option<PlayerId>,
    pub power: u32,
}

#[derive(Component)]
pub struct WorldArea {
    pub name: String,
    pub world_position: (i32, i32),
    pub nearest_neighbors: Vec<(i32, i32)>,
    pub followers: Vec<Follower>,
    pub agents: Vec<Agent>,
}

impl WorldArea {
    pub fn new(name: &str, x: i32, y: i32) -> Self {
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

    pub fn get_player_agent_count(&self, player: PlayerId) -> u32 {
        self.agents.iter().filter(|a| a.id.player == player).count() as u32
    }

    pub fn get_nth_player_agent(&self, player: PlayerId, agent: usize) -> Option<&Agent> {
        self.agents
            .iter()
            .filter(|a| a.id.player == player)
            .skip(agent)
            .next()
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
