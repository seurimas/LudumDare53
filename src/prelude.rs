pub use crate::assets::MyAssets;
pub use crate::game::agent::{Agent, AgentAction, AgentId};
pub use crate::game::ai::AiSeeds;
pub use crate::game::area::{Follower, WorldArea};
pub use crate::game::darkness::EvokingState;
pub use crate::game::persistence::*;
pub use crate::game::player::GamePlayers;
pub use crate::game::player::PlayerId;
pub use crate::game::player::PlayerTurn;
pub use crate::game::tiles::MapTile;
pub use crate::game::tiles::TileLoc;
pub use crate::game::tooltip::{SimpleTooltip, Tooltip};
pub use crate::game::turns::Season;
pub use crate::game::ui::{FONT_SIZE, ONE_UNIT};
pub use crate::state::GameState;
pub use bevy::prelude::*;
pub use bevy::{
    ui::RelativeCursorPosition,
    utils::{HashMap, HashSet},
};
pub use rand::Rng;
pub use rand::{rngs::StdRng, SeedableRng};
pub use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub fn walking_choose<T: Clone>(rng: &mut StdRng, choices: &[T]) -> Option<T> {
    for choice in choices {
        if rng.gen_bool(0.5) {
            return Some(choice.clone());
        }
    }
    None
}

pub fn choose<T: Clone>(rng: &mut StdRng, choices: &[T]) -> Option<T> {
    if choices.len() == 0 {
        return None;
    }
    let index = rng.gen_range(0..choices.len());
    Some(choices[index].clone())
}

pub fn choose_mut<'a, T>(rng: &mut StdRng, choices: &'a mut [T]) -> Option<&'a mut T> {
    if choices.len() == 0 {
        return None;
    }
    let index = rng.gen_range(0..choices.len());
    Some(&mut choices[index])
}

pub fn choose_mut_iter<'a, T, I>(rng: &mut StdRng, mut choices: I, size: usize) -> Option<&'a mut T>
where
    I: Iterator<Item = &'a mut T>,
{
    if size == 0 {
        return None;
    }
    let index = rng.gen_range(0..size);
    choices.nth(index)
}
