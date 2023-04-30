use crate::prelude::*;

use super::{player, ui::ActiveInactiveImages};

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentAction {
    None,
    Move(u32, u32, String),
    Prostelytize,
    Brutalize,
    Corrupt,
    Sacrifice,
}

pub const CORRUPT_POWER: u32 = 30;

impl AgentAction {
    pub fn describe(&self) -> String {
        match self {
            AgentAction::None => "This agent is unassigned.".to_string(),
            AgentAction::Move(_, _, name) => format!("Move to {}.", name),
            AgentAction::Prostelytize => {
                "Prostelytize to the locals, hoping to gain new followers.".to_string()
            }
            AgentAction::Brutalize => {
                "Brutalize the locals, scaring away the weak\nand attracting the strong."
                    .to_string()
            }
            AgentAction::Corrupt => {
                "Attempt to corrupt a follower,\ngaining access to greater power.".to_string()
            }
            AgentAction::Sacrifice => {
                "Sacrifice a local, hoping to unlock\na Sign of Corruption.".to_string()
            }
        }
    }

    pub fn invalid_reasons(&self, agent: &Agent, area: &WorldArea) -> Option<String> {
        match self {
            AgentAction::None => None,
            AgentAction::Move(x, y, name) => {
                if name.len() <= 3 {
                    Some("Must select a location to travel to.".to_string())
                } else if *x == area.world_position.0 && *y == area.world_position.1 {
                    Some("Must select a different location to travel to.".to_string())
                } else {
                    None
                }
            }
            AgentAction::Prostelytize => {
                if area.can_prostelytize(agent) {
                    None
                } else {
                    Some("You find no chances in the remaining minds here.".to_string())
                }
            }
            AgentAction::Brutalize => {
                if area.get_player_power(agent.id.player) > area.get_value() / 5 {
                    None
                } else {
                    Some("The heretics would overpower your followers.".to_string())
                }
            }
            AgentAction::Corrupt => {
                if area.get_player_power(agent.id.player) <= CORRUPT_POWER {
                    Some(format!(
                        "You need {} power to corrupt a follower.",
                        CORRUPT_POWER
                    ))
                } else if area.corrupted_followers(agent.id.player) > 0 {
                    Some("You already have a corrupted follower to enact sacrifices.".to_string())
                } else {
                    None
                }
            }
            AgentAction::Sacrifice => {
                if area.corrupted_count(agent.id.player) == 0 {
                    Some("You have no corrupted followers to enact sacrifices.".to_string())
                } else if area.get_player_power(agent.id.player) <= area.get_value() / 3 {
                    Some(format!("The heretics would stop your public sacrifice."))
                } else {
                    None
                }
            }
        }
    }

    pub fn sting(&self) -> &'static str {
        match self {
            AgentAction::Brutalize => "Brutalize.wav",
            AgentAction::Prostelytize => "Prostelytize.wav",
            AgentAction::Sacrifice => "Sacrifice.wav",
            _ => "",
        }
    }

    pub fn check_sum(&self) -> u32 {
        match self {
            AgentAction::None => 0,
            AgentAction::Move(x, y, _) => (*x + *y) as u32,
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
    pub world_position: (u32, u32),
    pub power: u32,
    pub corrupted: bool,
    pub stamina: u32,
    pub signs: u32,
}

impl Agent {
    pub fn new(name: String, id: AgentId, world_position: (u32, u32), power: u32) -> Self {
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
    pub locations: HashMap<AgentId, (u32, u32)>,
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
    x: u32,
    y: u32,
    agent_idx: u32,
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
    assets: Res<MyAssets>,
    audio: Res<Audio>,
) {
    let mut tooltip_value = None;
    if let Some((tile, world_area)) = map_query.iter().find(|(tile, _)| tile.selected) {
        if local.x != tile.x || local.y != tile.y {
            local.x = tile.x;
            local.y = tile.y;
            local.agent_idx = world_area
                .get_unassigned_player_agent(&player_turn)
                .unwrap_or(0);
            println!(
                "Showing agent {} {:?}",
                local.agent_idx,
                world_area.get_unassigned_player_agent(&player_turn)
            );
        }
        if let Some(active_agent) = world_area.get_nth_player_agent(*player_id, local.agent_idx) {
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
                    } else if name.eq_ignore_ascii_case("Area Agent") {
                        *visibility = Visibility::Visible;
                    }
                } else if let Ok(action) = action_query.get(entity) {
                    if rcp.map(|rcp| rcp.mouse_over()).unwrap_or_default() {
                        if let Some(invalid_reason) =
                            action.invalid_reasons(active_agent, world_area)
                        {
                            tooltip_value = Some(invalid_reason);
                        } else {
                            tooltip_value = Some(action.describe());
                        }
                    }
                }
            }
            for (entity, interaction) in interaction_query.iter_mut() {
                match interaction {
                    Interaction::Clicked => {
                        if let Ok(action) = action_query.get(entity) {
                            if action.invalid_reasons(active_agent, world_area).is_none() {
                                player_turn.set_action(active_agent.id, action.clone());
                                if let Some(sound) = assets.action_stings.get(action.sting()) {
                                    audio.play(sound.clone());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            for (entity, active_inactive, mut image) in image_query.iter_mut() {
                if let Some(action) = action_query.get(entity).ok() {
                    if player_turn.get_action(active_agent.id) == Some(action.clone()) {
                        image.texture = active_inactive.active.clone();
                    } else if action.invalid_reasons(active_agent, world_area).is_some() {
                        image.texture = active_inactive.deactivated.clone();
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

fn update_agent_label(
    player_turn: Res<PlayerTurn>,
    mut text_query: Query<(&Name, &mut Text)>,
    areas: Query<&WorldArea>,
) {
    let unassigned_agents = player_turn.get_unassigned_agents();

    let corrupted_count: u32 = areas
        .iter()
        .map(|area| area.corrupted_count(player_turn.player_id))
        .sum();

    let sign_count = areas
        .iter()
        .map(|area| area.sign_count(player_turn.player_id))
        .sum::<u32>();
    for (name, mut text) in text_query.iter_mut() {
        if name.eq_ignore_ascii_case("Agents-value") {
            text.sections[0].value = unassigned_agents.to_string();
        } else if name.eq_ignore_ascii_case("Corrupted-value") {
            text.sections[0].value = corrupted_count.to_string();
        } else if name.eq_ignore_ascii_case("Signs-value") {
            text.sections[0].value = sign_count.to_string();
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
