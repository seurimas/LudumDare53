use crate::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Agent {
    pub name: String,
    pub id: AgentId,
    pub world_position: (i32, i32),
    pub power: u32,
    pub corrupted: bool,
    pub stamina: u32,
    pub turn_action: AgentAction,
}

impl Agent {
    pub fn new(name: String, id: AgentId, world_position: (i32, i32)) -> Self {
        Self {
            name,
            id,
            world_position,
            power: 1,
            corrupted: false,
            stamina: 1,
            turn_action: AgentAction::None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId {
    pub player: PlayerId,
    pub agent: u32,
}

impl AgentId {
    pub fn new(player: u32, agent: u32) -> Self {
        Self {
            player: PlayerId(player),
            agent,
        }
    }
}

#[derive(Resource, Debug, Clone, Default)]
pub struct AgentLocations {
    pub locations: HashMap<AgentId, (i32, i32)>,
}

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AgentLocations>()
            .add_system(update_agent_locations.run_if(in_state(GameState::Playing)))
            .add_system(update_agent_label.run_if(in_state(GameState::Playing)))
            .add_system(agent_tooltip.run_if(in_state(GameState::Playing)));
    }
}

fn update_agent_locations(
    player: Res<PlayerId>,
    mut agent_locations: ResMut<AgentLocations>,
    areas: Query<&WorldArea>,
) {
    let mut locations = HashMap::new();
    for area in areas.iter() {
        for agent in area.agents.iter() {
            if agent.id.player == *player {
                locations.insert(agent.id, area.world_position);
            }
        }
    }
    agent_locations.locations = locations;
}

fn update_agent_label(player_turn: Res<PlayerTurn>, mut text_query: Query<(&Name, &mut Text)>) {
    let unassigned_agents = player_turn.get_unassigned_agents();
    for (name, mut text) in text_query.iter_mut() {
        if name.eq_ignore_ascii_case("Agents-value") {
            text.sections[0].value = unassigned_agents.to_string();
        }
    }
}

fn agent_tooltip(
    mut tooltip_control: Local<bool>,
    player_turn: Res<PlayerTurn>,
    player_agents: Res<AgentLocations>,
    mut tooltip: ResMut<Tooltip>,
    agent_label: Query<(&RelativeCursorPosition, &Name)>,
    mut tile_query: Query<(&mut MapTile)>,
    input: Res<Input<MouseButton>>,
) {
    for (cursor, name) in agent_label.iter() {
        if name.eq_ignore_ascii_case("Agents") {
            if cursor.mouse_over() {
                let unassigned_agents = player_turn.get_unassigned_agents();
                *tooltip_control = true;
                if unassigned_agents > 0 {
                    let unassigned_agent = player_turn.get_unassigned_agent();
                    let (agent_x, agent_y) = player_agents.locations[&unassigned_agent];
                    tooltip.value = Some(format!(
                        "{} unassigned agents. Click to focus one.",
                        unassigned_agents
                    ));
                    if input.just_pressed(MouseButton::Left) {
                        tile_query.iter_mut().for_each(|mut tile| {
                            if tile.x == agent_x && tile.y == agent_y {
                                tile.focused = true;
                                tile.selected = true;
                            } else {
                                tile.focused = false;
                                tile.selected = false;
                            }
                        });
                    }
                }
            } else if *tooltip_control {
                tooltip.value = None;
                *tooltip_control = false;
            }
        }
    }
}
