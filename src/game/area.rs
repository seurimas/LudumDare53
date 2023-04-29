use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Follower {
    pub sign_holder: bool,
    pub corrupted: bool,
    pub affinity: PlayerId,
    pub power: u32,
}

#[derive(Component)]
pub struct WorldArea {
    pub name: String,
    pub world_position: (i32, i32),
    pub followers: Vec<Follower>,
    pub agents: Vec<Agent>,
}

impl WorldArea {
    pub fn new(name: &str, x: i32, y: i32) -> Self {
        WorldArea {
            name: name.to_string(),
            world_position: (x, y),
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
}

pub struct AreaPlugin;

impl Plugin for AreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(render_area_ui.run_if(in_state(GameState::Playing)));
    }
}

fn render_area_ui(
    mut text_query: Query<(&Name, &mut Text)>,
    map_query: Query<(&MapTile, &WorldArea)>,
) {
    for (name, mut text) in text_query.iter_mut() {
        if name.eq_ignore_ascii_case("Area Name") {
            for (tile, area) in map_query.iter() {
                if tile.selected {
                    text.sections[0].value = area.name.clone();
                }
            }
        }
    }
}
