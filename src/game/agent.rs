use crate::prelude::*;

use super::{player, ui::ActiveInactiveImages};

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentAction {
    None,
    Move(usize),
    Prostelytize,
    Brutalize,
    Corrupt,
    Sacrifice,
}

impl AgentAction {
    pub fn describe(&self) -> String {
        match self {
            AgentAction::None => "This agent is unassigned.".to_string(),
            AgentAction::Move(idx) => match idx {
                0 => format!("Move to the nearest area."),
                1 => format!("Move to the {}nd nearest area.", idx + 1),
                2 => format!("Move to the {}rd nearest area.", idx + 1),
                _ => format!("Move to the {}th nearest area.", idx + 1),
            },
            AgentAction::Prostelytize => {
                "Prostelytize to the locals, hoping to gain new followers".to_string()
            }
            AgentAction::Brutalize => {
                "Brutalize the locals, scaring away the weak and attracting the strong".to_string()
            }
            AgentAction::Corrupt => {
                "Attempt to corrupt a follower, gaining access to greater power".to_string()
            }
            AgentAction::Sacrifice => {
                "Sacrifice a local, hoping to unlock a Sign of Corruption".to_string()
            }
        }
    }

    pub fn check_sum(&self) -> u32 {
        match self {
            AgentAction::None => 0,
            AgentAction::Move(idx) => 1 + *idx as u32,
            AgentAction::Prostelytize => 8,
            AgentAction::Brutalize => 9,
            AgentAction::Corrupt => 10,
            AgentAction::Sacrifice => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub id: AgentId,
    pub world_position: (usize, usize),
    pub power: u32,
    pub corrupted: bool,
    pub stamina: u32,
    pub signs: u32,
}

impl Agent {
    pub fn new(name: String, id: AgentId, world_position: (usize, usize), power: u32) -> Self {
        Self {
            name,
            id,
            world_position,
            power,
            corrupted: false,
            stamina: 100,
            signs: 0,
        }
    }

    pub fn exhaust(&mut self, amount: u32) {
        self.stamina = self.stamina.saturating_sub(amount);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub locations: HashMap<AgentId, (usize, usize)>,
}

pub struct AgentPlugin;

impl Plugin for AgentPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AgentLocations>()
            .add_system(render_agent_ui.run_if(in_state(GameState::Playing)))
            .add_system(update_agent_locations.run_if(in_state(GameState::Playing)))
            .add_system(prepare_my_turn.run_if(in_state(GameState::Playing)))
            .add_system(update_agent_label.run_if(in_state(GameState::Playing)))
            .add_system(agent_tooltip.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Default)]
struct AgentUiState {
    x: usize,
    y: usize,
    agent_idx: usize,
    tooltip_control: bool,
}

fn render_agent_ui(
    mut local: Local<AgentUiState>,
    mut tooltip: ResMut<Tooltip>,
    player_id: Res<PlayerId>,
    mut player_turn: ResMut<PlayerTurn>,
    map_query: Query<(&MapTile, &WorldArea)>,
    mut ui_query: Query<
        (
            Entity,
            Option<&Name>,
            Option<&mut Text>,
            Option<&RelativeCursorPosition>,
            &mut Visibility,
        ),
        Or<(With<Name>, With<AgentAction>)>,
    >,
    action_query: Query<&AgentAction>,
    mut image_query: Query<(Entity, &ActiveInactiveImages, &mut UiImage)>,
    mut interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<Button>)>,
) {
    let AgentUiState {
        x,
        y,
        agent_idx,
        tooltip_control,
    } = *local;
    let mut tooltip_value = None;
    if let Some((tile, world_area)) = map_query.iter().find(|(tile, _)| tile.selected) {
        if x != tile.x || y != tile.y {
            local.x = tile.x;
            local.y = tile.y;
            local.agent_idx = world_area
                .get_unassigned_player_agent(&player_turn)
                .unwrap_or(0);
        }
        if let Some(active_agent) = world_area.get_nth_player_agent(*player_id, agent_idx) {
            for (entity, name, mut text, rcp, mut visibility) in ui_query.iter_mut() {
                if let Some(name) = name {
                    if name.eq_ignore_ascii_case("Agent") {
                        text.unwrap().sections[0].value = active_agent.name.clone();
                    } else if name.eq_ignore_ascii_case("Agent Power") {
                        text.unwrap().sections[0].value = active_agent.power.to_string();
                    } else if name.eq_ignore_ascii_case("Agent Stamina") {
                        text.unwrap().sections[0].value = active_agent.stamina.to_string();
                    } else if name.eq_ignore_ascii_case("Corrupted?") {
                        if active_agent.corrupted {
                            text.unwrap().sections[0].value = "Yes".to_string();
                        } else {
                            text.unwrap().sections[0].value = "No".to_string();
                        }
                    }
                } else if let Ok(action) = action_query.get(entity) {
                    if rcp.map(|rcp| rcp.mouse_over()).unwrap_or_default() {
                        tooltip_value = Some(action.describe());
                    }
                }
            }
            for (entity, interaction) in interaction_query.iter_mut() {
                match interaction {
                    Interaction::Clicked => {
                        if let Ok(action) = action_query.get(entity) {
                            player_turn.set_action(active_agent.id, *action);
                        }
                    }
                    _ => {}
                }
            }
            for (entity, active_inactive, mut image) in image_query.iter_mut() {
                if let Some(action) = action_query.get(entity).ok() {
                    if player_turn.get_action(active_agent.id) == Some(*action) {
                        image.texture = active_inactive.active.clone();
                    } else {
                        image.texture = active_inactive.inactive.clone();
                    }
                }
            }
        } else {
            for (_, name, _, _, mut visibility) in ui_query.iter_mut() {
                if name
                    .map(|name| name.eq_ignore_ascii_case("Area Agent"))
                    .unwrap_or_default()
                {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
    if let Some(tooltip_value) = tooltip_value {
        tooltip.value = Some(tooltip_value);
        local.tooltip_control = true;
    } else if local.tooltip_control {
        local.tooltip_control = false;
        tooltip.value = None;
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

fn prepare_my_turn(
    player: Res<PlayerId>,
    mut player_turn: ResMut<PlayerTurn>,
    mut agent_locations: ResMut<AgentLocations>,
) {
    for (agent_id, _) in agent_locations.locations.iter_mut() {
        player_turn.initialize_agent(*agent_id);
    }
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
