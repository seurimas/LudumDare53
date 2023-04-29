use crate::{prelude::*, state::GameState};
use bevy_asset_loader::prelude::*;

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::Playing),
        )
        .add_collection_to_loading_state::<_, MyAssets>(GameState::Loading);
    }
}

#[derive(AssetCollection, Resource)]
pub struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 128., tile_size_y = 128., columns = 4, rows = 4))]
    #[asset(path = "map.png")]
    pub map: Handle<TextureAtlas>,
}
