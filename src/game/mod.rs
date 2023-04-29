use bevy::{app::PluginGroupBuilder, prelude::PluginGroup};

pub mod agent;
pub mod area;
pub mod darkness;
pub mod multiplayer;
pub mod player;
pub mod tiles;
pub mod tooltip;
pub mod turn_ui;
pub mod ui;

pub struct GamePlugins;

impl PluginGroup for GamePlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = group.add(agent::AgentPlugin);
        group = group.add(area::AreaPlugin);
        group = group.add(darkness::DarknessPlugin);
        group = group.add(tiles::TilesPlugin);
        group = group.add(ui::UiPlugin);
        group = group.add(tooltip::TooltipPlugin);
        group = group.add(turn_ui::TurnUiPlugin);

        group
    }
}
