use super::{ToTile, ThroughResult};
use serde::{Serialize,Deserialize};

#[derive(Serialize,Deserialize)]
pub enum Gate {
    // tile, gate, flip
    Matched(ToTile),
    // bridge
    Loose(usize),
    None,
}

impl Gate {
    pub fn get(&self) -> ThroughResult {
        match self {
            Self::Matched(to) => {
                return ThroughResult::Exists(*to);
            },
            Self::Loose(bridge) => {
                return ThroughResult::Generate(*bridge);
            },
            Self::None => ThroughResult::None,
        }
    }
}

