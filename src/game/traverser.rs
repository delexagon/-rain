use super::gamedata::GameData;
use super::identifiers::UniqTile;

// Can traverse the map.
// No code should directly try to access or move through the map without the assistance of a traverser.
#[derive(Clone,Copy)]
pub struct Traverser {
    pub tile: UniqTile,
    ab_is_lr: u8,
    ud_flipped: u8,
    lr_flipped: u8,
}

impl Traverser {
    pub fn new(tile: UniqTile) -> Traverser {
        Traverser {
            tile: tile,
            ab_is_lr: 0,
            ud_flipped: 0,
            lr_flipped: 0,
        }
    }

    pub fn travel(&self, dir: u8, data: &GameData) -> Traverser {
        fn orientation_to_gate(dir: u8, ab_is_lr: u8, ud_flipped: u8, lr_flipped: u8) -> u8 {
            let flipped = ((dir/2)^1)*ud_flipped+dir/2*lr_flipped;
            ab_is_lr*2+flipped+dir-(2*((dir%2)&flipped))-(4*((dir/2)&ab_is_lr))
        }
        let mut new_traverser = Traverser::none();
        if !data.tile_exists(self.tile) {
            return new_traverser;
        }
        let gate_from = orientation_to_gate(dir, self.ab_is_lr, self.ud_flipped, self.lr_flipped);
        let (tile_to, gate_to, flip) = data.through(self.tile, gate_from);
        new_traverser.tile = tile_to;
        new_traverser.ab_is_lr = (gate_to/2)^(dir/2);
        let primary_flipped = (gate_to%2)^(dir%2)^1;
        if dir/2 == 0 { 
            new_traverser.ud_flipped = primary_flipped;
            new_traverser.lr_flipped = flip^self.lr_flipped;
        } else {
            new_traverser.lr_flipped = primary_flipped;
            new_traverser.ud_flipped = flip^self.ud_flipped;
        }
        new_traverser
    }
    
    pub fn none() -> Traverser {
        Traverser {
            tile: UniqTile {map: 0, tile: 0},
            ab_is_lr: 0,
            ud_flipped: 0,
            lr_flipped: 0,
        }
    }
    
    pub fn tile(&self) -> UniqTile {
        return self.tile;
    }
    
    pub fn same_tile(&self, other: &Traverser) -> bool {
        return self.tile.map == other.tile.map && self.tile.tile == other.tile.tile;
    }
    
    pub fn same_orientation(&self, other: &Traverser) -> bool {
        return self.ab_is_lr == other.ab_is_lr && self.lr_flipped == other.lr_flipped && self.ud_flipped == other.ud_flipped;
    }
    
    pub fn tile_exists(&self, data: &GameData) -> bool {
        return data.tile_exists(self.tile);
    }
}
