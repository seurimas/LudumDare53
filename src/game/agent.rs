use crate::prelude::*;

use super::{player, ui::ActiveInactiveImages, WIN_SIGN_COUNT};

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentAction {
    None,
    Move(u32, u32, #[serde(skip)] String),
    Prostelytize,
    Brutalize,
    Corrupt,
    CorruptAgent,
    Sacrifice,
}

pub const CORRUPT_POWER: u32 = 30;
pub const HIDE_BUTTON: &'static str = "HIDE";

impl AgentAction {
    pub fn describe(&self) -> String {
        match self {
            AgentAction::None => "View next agent in area, if any.".to_string(),
            AgentAction::Move(_, _, name) => format!("Travel\n\nMove to {}.\nWhen you leave, a powerful follower in this area\nmay become an agent.", name),
            AgentAction::Prostelytize => {
                "Prostelytize\n\nSpread the word of darkness to the locals to gain new followers.".to_string()
            }
            AgentAction::Brutalize => {
                "Brutalize\n\nBrutalize the locals, scaring away the weak\nand attracting the strong."
                    .to_string()
            }
            AgentAction::Corrupt => {
                "Corrupt\n\nAttempt to corrupt a follower,\ngaining access to greater power.".to_string()
            }
            AgentAction::CorruptAgent => {
                "Corrupt Agent\n\nAttempt to corrupt this agent.\nYou will lose a corrupted follower.".to_string()
            }
            AgentAction::Sacrifice => {
                "Sacrifice\n\nSacrifice a local, hoping to unlock\na Sign of Corruption.".to_string()
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
                if area.get_non_player_agent_power(agent.id.player)
                    > area.get_player_power(agent.id.player)
                {
                    Some("A hertical agent would overpower your followers.".to_string())
                } else if area.get_player_power(agent.id.player) > area.get_value() / 5 {
                    None
                } else {
                    Some("The locals are not afraid of your followers.".to_string())
                }
            }
            AgentAction::Corrupt => {
                if area.get_player_power(agent.id.player) <= CORRUPT_POWER {
                    Some(format!(
                        "You need {} power to corrupt a follower.",
                        CORRUPT_POWER
                    ))
                } else if area.corrupted_followers(agent.id.player) > 0 {
                    Some(HIDE_BUTTON.to_string())
                } else {
                    None
                }
            }
            AgentAction::CorruptAgent => {
                if agent.corrupted {
                    Some(format!("{} is already corrupted.", agent.name))
                } else if area.corrupted_followers(agent.id.player) == 0 {
                    Some(HIDE_BUTTON.to_string())
                } else if area.get_possible_sign_holder_count(agent.id) > 0 {
                    Some(format!(
                        "You cannot corrupt agents while there is still a chance to find a sign holder.\nBegin the sacrifices."
                    ))
                } else {
                    None
                }
            }
            AgentAction::Sacrifice => {
                if area.corrupted_count(agent.id.player) == 0 {
                    Some("You have no corrupted followers to enact sacrifices.".to_string())
                } else if area.get_player_power(agent.id.player) <= area.get_value() / 3 {
                    Some(format!("The locals would stop your public sacrifice."))
                } else if area.get_possible_sign_holder_count(agent.id) == 0 {
                    Some(format!(
                        "There are no possible sign holders here.\nRecruit more or search elsewhere."
                    ))
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
            AgentAction::Corrupt => "Corrupt.wav",
            AgentAction::Move(_, _, _) => "Move.wav",
            _ => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Agent {
    pub name: String,
    pub id: AgentId,
    // pub world_position: (u32, u32),
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
            // world_position,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
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
            .add_system(signs_tooltip.run_if(in_state(GameState::Playing)))
            .add_system(corrupted_tooltip.run_if(in_state(GameState::Playing)))
            .add_system(agent_tooltip.run_if(in_state(GameState::Playing)));
    }
}

#[derive(Default)]
struct AgentUiState {
    x: TileLoc,
    y: TileLoc,
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
    tile_input: Res<TileInputState>,
) {
    let mut tooltip_value = None;
    if let Some((tile, world_area)) = tile_input
        .selected
        .and_then(|(entity, _, _)| map_query.get(entity).ok())
    {
        if local.x != tile.x || local.y != tile.y {
            local.x = tile.x;
            local.y = tile.y;
            local.agent_idx = world_area
                .get_unassigned_player_agent(&player_turn)
                .unwrap_or(0);
        }
        if let Some(active_agent) = world_area.get_nth_player_agent(*player_id, local.agent_idx) {
            for (entity, name, mut text, rcp, mut visibility) in ui_query.iter_mut() {
                if let Some(name) = name {
                    if name.eq_ignore_ascii_case("agent_name") {
                        text.unwrap().sections[0].value = active_agent.name.clone();
                    } else if name.eq_ignore_ascii_case("agent_power") {
                        text.unwrap().sections[0].value = active_agent.power.to_string();
                    } else if name.eq_ignore_ascii_case("agent_signs") {
                        text.unwrap().sections[0].value = active_agent.signs.to_string();
                    } else if name.eq_ignore_ascii_case("Area Agent") {
                        *visibility = Visibility::Visible;
                    }
                } else if let Ok(action) = action_query.get(entity) {
                    let invalid = action.invalid_reasons(active_agent, world_area);
                    if let Some(invalid_reason) = &invalid {
                        if invalid_reason.eq(HIDE_BUTTON) {
                            *visibility = Visibility::Hidden;
                        } else {
                            *visibility = Visibility::Visible;
                        }
                    } else {
                        *visibility = Visibility::Visible;
                    }
                    if rcp.map(|rcp| rcp.mouse_over()).unwrap_or_default() {
                        if let Some(invalid_reason) = invalid {
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
                            if *action == AgentAction::None {
                                local.agent_idx += 1;
                            } else if action.invalid_reasons(active_agent, world_area).is_none() {
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
            for (entity, name, _, _, mut visibility) in ui_query.iter_mut() {
                if name
                    .map(|name| name.eq_ignore_ascii_case("Area Agent"))
                    .unwrap_or_default()
                {
                    *visibility = Visibility::Hidden;
                } else if action_query.get(entity).is_ok() {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    } else {
        for (entity, name, _, _, mut visibility) in ui_query.iter_mut() {
            if name
                .map(|name| name.eq_ignore_ascii_case("Area Agent"))
                .unwrap_or_default()
            {
                *visibility = Visibility::Hidden;
            } else if action_query.get(entity).is_ok() {
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
    mut tile_query: Query<(Entity, &mut MapTile, &WorldArea)>,
    input: Res<Input<MouseButton>>,
    mut tile_input: ResMut<TileInputState>,
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
                        if let Some((tile_entity, _, _)) = tile_query.iter().find(|(_, tile, _)| {
                            tile.x == agent_x as TileLoc && tile.y == agent_y as TileLoc
                        }) {
                            tile_input.selected =
                                Some((tile_entity, agent_x as TileLoc, agent_y as TileLoc));
                        }
                    }
                }
            } else if *tooltip_control {
                tooltip.value = None;
                *tooltip_control = false;
            }
        }
    }
}

fn corrupted_tooltip(
    mut corrupted_idx: Local<usize>,
    mut tooltip_control: Local<bool>,
    player_turn: Res<PlayerTurn>,
    mut tooltip: ResMut<Tooltip>,
    agent_label: Query<(&RelativeCursorPosition, &Name)>,
    mut tile_query: Query<(Entity, &mut MapTile, &WorldArea)>,
    input: Res<Input<MouseButton>>,
    mut tile_input: ResMut<TileInputState>,
) {
    for (cursor, name) in agent_label.iter() {
        if name.eq_ignore_ascii_case("Corrupted") {
            if cursor.mouse_over() {
                *tooltip_control = true;
                let corrupted_count = tile_query
                    .iter()
                    .filter(|(_, _, area)| area.corrupted_count(player_turn.player_id) > 0)
                    .count();
                if corrupted_count > 0 {
                    tooltip.value = Some(format!(
                        "{} areas have corrupted followers. Click to focus one.",
                        corrupted_count,
                    ));
                } else {
                    tooltip.value = Some("You have no corrupted followers.\nSpread the words of darkness to grow your power.\nThen corrupt your followers.".to_string());
                }
                if corrupted_count > 0 && input.just_pressed(MouseButton::Left) {
                    *corrupted_idx += 1;
                    if let Some((tile_entity, mut tile, _)) = tile_query
                        .iter_mut()
                        .filter(|(_, _, area)| area.corrupted_count(player_turn.player_id) > 0)
                        .nth(*corrupted_idx % corrupted_count)
                    {
                        tile_input.selected = Some((tile_entity, tile.x, tile.y));
                    }
                }
            } else if *tooltip_control {
                tooltip.value = None;
                *tooltip_control = false;
            }
        }
    }
}

fn signs_tooltip(
    mut tooltip_control: Local<bool>,
    player_turn: Res<PlayerTurn>,
    mut tooltip: ResMut<Tooltip>,
    agent_label: Query<(&RelativeCursorPosition, &Name)>,
    tile_query: Query<(&MapTile, &WorldArea)>,
) {
    for (cursor, name) in agent_label.iter() {
        if name.eq_ignore_ascii_case("Signs") {
            if cursor.mouse_over() {
                *tooltip_control = true;
                let signs_count = tile_query
                    .iter()
                    .map(|(_, area)| area.sign_count(player_turn.player_id))
                    .sum::<u32>();
                if signs_count > 0 {
                    tooltip.value = Some(format!(
                        "Your agents control {} signs. You need {} to win.",
                        signs_count, WIN_SIGN_COUNT,
                    ));
                } else {
                    tooltip.value = Some("You have no signs.\nCorrupted followers can enact sacrifices to discover signs.".to_string());
                }
            } else if *tooltip_control {
                tooltip.value = None;
                *tooltip_control = false;
            }
        }
    }
}
