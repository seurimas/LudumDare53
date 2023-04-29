use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

pub mod agent;
pub mod area;
pub mod map;
pub mod ui;

pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        // group.add(agent::AgentPlugin);
        // group.add(area::AreaPlugin);
        group = group.add(map::MapPlugin);
        group = group.add(ui::UiPlugin);

        group
    }
}
