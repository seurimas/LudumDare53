use crate::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Follower {
    pub sign_holder: bool,
    pub corrupted: bool,
    #[serde(skip)]
    pub fleeing: Option<(u32, u32)>,
    pub affinity: Option<PlayerId>,
    pub power: u32,
}

impl Follower {
    pub fn new(power: u32) -> Self {
        Self {
            sign_holder: false,
            corrupted: false,
            fleeing: None,
            affinity: None,
            power,
        }
    }
}
