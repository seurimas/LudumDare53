use crate::prelude::*;

mod map_desc;
mod runes;
pub use map_desc::*;
pub use runes::*;

pub struct PersistencePlugin;

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        // app.add_system(save_periodically.run_if(in_state(GameState::Playing)));
    }
}

// fn save_periodically(

// ) {

// }
