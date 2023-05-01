use crate::prelude::*;

#[derive(Resource, Deref, Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayers(pub Vec<String>);

impl GamePlayers {
    pub fn new(mut players: Vec<String>, ai: u32) -> Self {
        players.extend((0..ai).map(|i| format!("AI Player {}", i)));
        Self(players)
    }

    pub fn get_save_prefix(&self, player_id: PlayerId) -> String {
        format!(
            "{} ({})",
            self.0
                .iter()
                .filter(|name| !name.starts_with("AI Player"))
                .cloned()
                .collect::<Vec<_>>()
                .join(" vs "),
            player_id.0
        )
    }

    pub fn get_name(&self, player_id: PlayerId) -> Option<&String> {
        self.0.get(player_id.0 as usize)
    }

    pub fn get_ids(&self) -> Vec<PlayerId> {
        (0..self.0.len()).map(|i| PlayerId(i as u32)).collect()
    }

    pub fn get_ai_seed_index(&self, player_id: PlayerId) -> Option<usize> {
        let mut ai_seed = 0;
        for (i, name) in self.0.iter().enumerate() {
            if name.starts_with("AI Player") {
                if PlayerId(i as u32) == player_id {
                    return Some(ai_seed);
                }
                ai_seed += 1;
            }
        }
        None
    }

    pub fn is_ai(&self, player_id: PlayerId) -> bool {
        self.get_name(player_id)
            .map(|name| name.starts_with("AI Player"))
            .unwrap_or(false)
    }
}

#[derive(
    Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct PlayerId(pub u32);

impl From<usize> for PlayerId {
    fn from(id: usize) -> Self {
        Self(id as u32)
    }
}

#[derive(Resource, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTurn {
    pub player_id: PlayerId,
    pub actions: HashMap<AgentId, AgentAction>,
}

impl PlayerTurn {
    pub fn new(player_id: PlayerId) -> Self {
        let mut actions = HashMap::default();
        Self { player_id, actions }
    }

    pub fn initialize_agent(&mut self, agent_id: AgentId) {
        if !self.actions.contains_key(&agent_id) && agent_id.player == self.player_id {
            self.actions.insert(agent_id, AgentAction::None);
        }
    }

    pub fn reset(&mut self) {
        self.actions.clear();
    }

    pub fn set_action(&mut self, agent_id: AgentId, action: AgentAction) {
        self.actions.insert(agent_id, action);
    }

    pub fn get_action(&self, agent_id: AgentId) -> Option<AgentAction> {
        self.actions.get(&agent_id).cloned()
    }

    pub fn is_unassigned_player_agent(&self, agent_id: AgentId) -> bool {
        if agent_id.player != self.player_id {
            return false;
        }
        if let Some(action) = self.actions.get(&agent_id) {
            if let AgentAction::None = action {
                return true;
            } else {
                return false;
            }
        }
        true
    }

    pub fn get_unassigned_agents(&self) -> i32 {
        let mut unassigned_agents = 0;
        for (_, action) in self.actions.iter() {
            if let AgentAction::None = action {
                unassigned_agents += 1;
            }
        }
        unassigned_agents
    }

    pub fn get_unassigned_agent(&self) -> AgentId {
        for (agent_id, action) in self.actions.iter() {
            if let AgentAction::None = action {
                return *agent_id;
            }
        }
        panic!("No unassigned agents");
    }
}
