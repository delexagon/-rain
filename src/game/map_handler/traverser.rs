use crate::game::GameData;
use super::{UniqTile, void_tile};
use serde::{Serialize,Deserialize};

#[derive(Clone,Copy,Eq,PartialEq,Serialize,Deserialize)]
pub struct TraverserCore {
    // These are all boolean variables, but are u8 here so we can do arithmatic on them
    pub ab_is_lr: u8,
    pub ud_flipped: u8,
    pub lr_flipped: u8,
}

impl TraverserCore {
    // Makes a TraverserCore with a direction preset to be a particular gate
    pub fn from(dir: u8, gate: u8, other_flipped: u8) -> TraverserCore {
        // Differences: primary_flipped is set face the gate rather than to face away from the gate,
        // other_flipped is always used
        let axis = dir/2;
        let ab_is_lr = (gate/2)^(axis);
        let primary_flipped = (gate%2)^(dir%2);
        let lr_flipped;
        let ud_flipped;
        if dir/2 == 0 { 
            ud_flipped = primary_flipped;
            lr_flipped = other_flipped;
        } else {
            lr_flipped = primary_flipped;
            ud_flipped = other_flipped;
        }
        return TraverserCore {ab_is_lr: ab_is_lr, lr_flipped: lr_flipped, ud_flipped: ud_flipped};
    }

    pub fn to(&self, dir: u8, gate_we_came_in_from: u8, flip: u8) -> TraverserCore {
        let ab_is_lr = (gate_we_came_in_from/2)^(dir/2);
        let primary_flipped = (gate_we_came_in_from%2)^(dir%2)^1;
        let lr_flipped;
        let ud_flipped;
        if dir/2 == 0 { 
            ud_flipped = primary_flipped;
            lr_flipped = flip^self.lr_flipped;
        } else {
            lr_flipped = primary_flipped;
            ud_flipped = flip^self.ud_flipped;
        }
        return TraverserCore {ab_is_lr: ab_is_lr, lr_flipped: lr_flipped, ud_flipped: ud_flipped};
    }
    
    pub fn gate_for(&self, dir: u8) -> u8 {
        let flipped = ((dir/2)^1)*self.ud_flipped+dir/2*self.lr_flipped;
        self.ab_is_lr*2+flipped+dir-(2*((dir%2)&flipped))-(4*((dir/2)&self.ab_is_lr))
    }
}

// Can traverse the map.
// No code should directly try to access or move through the map without the assistance of a traverser.
#[derive(Clone,Copy,Serialize,Deserialize)]
pub struct Traverser {
    pub tile: UniqTile,
    pub core: TraverserCore,
}

impl Traverser {
    pub fn new(tile: UniqTile) -> Traverser {
        Traverser {
            tile: tile,
            core: TraverserCore {
                ab_is_lr: 0,
                ud_flipped: 0,
                lr_flipped: 0,
            }
        }
    }
    
    pub fn none() -> Traverser {
        Traverser {
            tile: void_tile(),
            core: TraverserCore {
                ab_is_lr: 0,
                ud_flipped: 0,
                lr_flipped: 0,
            }
        }
    }
    
    pub fn tile(&self) -> UniqTile {
        return self.tile;
    }
    
    pub fn same_tile(&self, other: &Traverser) -> bool {
        return self.tile.map == other.tile.map && self.tile.tile == other.tile.tile;
    }
    
    pub fn same_orientation(&self, other: &Traverser) -> bool {
        return self.core == other.core;
    }
}
