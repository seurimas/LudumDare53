use bevy::utils::HashMap;

use crate::prelude::*;

pub struct TurnPlugin;

impl Plugin for TurnPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerTurn::new(PlayerId(0), 5))
            .insert_resource(PlayerId(0));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentAction {
    None,
    Move(i32, i32),
    Prostelytize,
    Brutalize,
    Corrupt,
    Sacrifice,
}

#[derive(Resource, Debug, Clone, PartialEq, Eq)]
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
