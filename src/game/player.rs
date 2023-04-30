use crate::prelude::*;

#[derive(Resource, Deref, Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayers(pub HashMap<PlayerId, String>);

impl GamePlayers {
    pub fn new(players: Vec<String>) -> Self {
        let mut map = HashMap::new();
        for (i, name) in players.into_iter().enumerate() {
            map.insert(PlayerId(i as u32), name);
        }
        Self(map)
    }
}

#[derive(
    Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord,
)]
pub struct PlayerId(pub u32);

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
