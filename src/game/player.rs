use crate::prelude::*;

#[derive(Resource, Deref, Debug, Clone, Serialize, Deserialize)]
pub struct GamePlayers(pub HashMap<PlayerId, String>);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PlayerId(pub u32);

#[derive(Resource, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerTurn {
    pub player_id: PlayerId,
    pub actions: HashMap<AgentId, AgentAction>,
}

impl PlayerTurn {
    pub fn new(player_id: PlayerId, agent_count: u32) -> Self {
        let mut actions = HashMap::default();
        for i in 0..agent_count {
            actions.insert(
                AgentId {
                    player: player_id,
                    agent: i,
                },
                AgentAction::None,
            );
        }
        Self { player_id, actions }
    }

    pub fn set_action(&mut self, agent_id: AgentId, action: AgentAction) {
        self.actions.insert(agent_id, action);
    }

    pub fn get_action(&self, agent_id: AgentId) -> Option<AgentAction> {
        self.actions.get(&agent_id).copied()
    }

    pub fn is_unassigned_player_agent(&self, agent_id: AgentId) -> bool {
        if agent_id.player != self.player_id {
            return false;
        }
        if let Some(action) = self.actions.get(&agent_id) {
            if let AgentAction::None = action {
                return true;
            }
        }
        false
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
