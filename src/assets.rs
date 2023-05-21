use crate::{prelude::*, state::GameState};
use bevy_asset_loader::prelude::*;

pub struct GameAssetsPlugin;

impl Plugin for GameAssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::MainMenu),
        )
        .add_collection_to_loading_state::<_, MyAssets>(GameState::Loading);
    }
}

#[derive(AssetCollection, Resource)]
pub struct MyAssets {
    #[asset(texture_atlas(tile_size_x = 128., tile_size_y = 128., columns = 4, rows = 4))]
    #[asset(path = "map.png")]
    pub map: Handle<TextureAtlas>,
    #[asset(
        paths(
            "tile.glb#Scene2",
            "tile.glb#Scene0",
            "tile.glb#Scene1",
            "tile.glb#Scene4",
            "tile.glb#Scene3",
        ),
        collection(typed)
    )]
    pub tiles: Vec<Handle<Scene>>,
    #[asset(path = "anglodavek/Anglodavek-a55E.ttf")]
    pub fancy_font: Handle<Font>,
    #[asset(path = "breath-fire-iii/BreatheFireIii-PKLOB.ttf")]
    pub font: Handle<Font>,
    #[asset(path = "tile_hover.wav")]
    pub tile_hover: Handle<AudioSource>,
    #[asset(path = "tile_click.wav")]
    pub tile_click: Handle<AudioSource>,
    #[asset(path = "EvokeDarkness.wav")]
    pub evoke_darkness: Handle<AudioSource>,
    #[asset(
        paths(
            "Brutalize.wav",
            "Prostelytize.wav",
            "Sacrifice.wav",
            "Corrupt.wav",
            "Move.wav"
        ),
        collection(typed, mapped)
    )]
    pub action_stings: HashMap<String, Handle<AudioSource>>,
    #[asset(path = "Win.wav")]
    pub win: Handle<AudioSource>,
    #[asset(path = "Lose.wav")]
    pub lose: Handle<AudioSource>,
    #[asset(
        paths(
            "Next.png",
            "NextActive.png",
            "NextDeactivated.png",
            "Move.png",
            "MoveActive.png",
            "MoveDeactivated.png",
            "Prostelytize.png",
            "ProstelytizeActive.png",
            "ProstelytizeDeactivated.png",
            "Brutalize.png",
            "BrutalizeActive.png",
            "BrutalizeDeactivated.png",
            "Sacrifice.png",
            "SacrificeActive.png",
            "SacrificeDeactivated.png",
            "Corrupt.png",
            "CorruptActive.png",
            "CorruptDeactivated.png",
            "CorruptAgent.png",
            "CorruptAgentActive.png",
            "CorruptAgentDeactivated.png"
        ),
        collection(typed, mapped)
    )]
    pub action_buttons: HashMap<String, Handle<Image>>,
}
