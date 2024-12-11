//mod brokenmap;
mod voidmap;
mod sparsemap; pub use sparsemap::SparseMap;
mod euclidmap; pub use euclidmap::EuclidMap;
mod gate;
//mod tile;
mod invalidmap; pub use invalidmap::{UninitializedMap, InvalidMap};


use gate::Gate;
//pub use brokenmap::*;
pub use voidmap::VoidMap;
//pub use sparsemap::*;
use super::TraverserCore;
//use tile::Tile;

use serde::{Serialize,Deserialize};
use enum_dispatch::enum_dispatch;
use crate::common::TileStyle;
use super::{TileID, void_tile, ThroughResult, ToTile, Bridge, MapData, MapID, UniqTile, MapGate, Traverser};
use crate::game::GameData;

use std::collections::HashMap;
pub type GateMap = HashMap<MapGate, Gate>;

#[enum_dispatch]
#[derive(Serialize,Deserialize)]
pub enum MapEnum {
    VoidMap, EuclidMap, SparseMap,
    // A map will be put into place soon
    UninitializedMap,
    // For when a map cannot be loaded from a file; points to the void map
    InvalidMap
}


// Map ettiquette:
// 1. Maps should be made of all contiguous tiles, so that tiles are not loaded unnecessarily.
// 2. Maps should be fairly small, because they are loaded all at once.
// 3. 
#[enum_dispatch(MapEnum)]
pub trait Map {
    fn id(&self) -> MapID;
    // Does this tile exist in the map? Should align with one sided connect and others
    fn has_tile(&self, tile: TileID) -> bool;
    
    // Connects one side of this map.
    fn one_sided_connect(&mut self, tile: TileID, gate: u8, other: UniqTile, other_gate: u8, flip: u8);
    
    // Any code not in Traverser or GameData that calls this function is a BUG!
    fn through(&self, tile: TileID, gate: u8) -> ThroughResult;
    
    fn passable(&self, tile: TileID) -> bool;
    
    // Returns whether this tile is potentially connected to another tile.
    // Used by various functions to see if tiles can connect or not.
    fn tile_connected(&self, tile: TileID, gate: u8) -> bool;
    
    // Returns the background of a tile.
    fn background_style(&self, tile: TileID) -> TileStyle;

    fn random_tile(&self) -> TileID;
    
    // Returns which gates from this tile are unconnected
    fn find_unconnected(&self, tile: TileID) -> [bool;4] {
        [!self.tile_connected(tile, 0),
        !self.tile_connected(tile, 1),
        !self.tile_connected(tile, 2),
        !self.tile_connected(tile, 3)]
    }
    
    // Returns the first unconnected side of a tile on the map
    // Returns 0 if no sides are connected
    fn first_unconnected(&self, tile: TileID) -> u8 {
        for i in 0..4 {
            if !self.tile_connected(tile, i) {
                return i;
            }
        }
        return 0;
    }
    
    // Only works internally in a map; need MapHandler functions to do inter-map connections
    fn two_sided_connect(&mut self, tile: TileID, gate: u8, other: TileID, other_gate: u8, flip: u8) {
        self.one_sided_connect(tile, gate, UniqTile {map: self.id(), tile: other}, other_gate, flip);
        self.one_sided_connect(other, other_gate, UniqTile {map: self.id(), tile: tile}, gate, flip);
    }
    
    // Unlike connect() functions, flip must be true on only one glued bridge for the result to be flipped.
    fn glue_one_side(&mut self, my_bridge: &Vec<MapGate>, other: MapID, other_bridge: &Vec<MapGate>) {
        if my_bridge.len() != other_bridge.len() {
            return;
        }
        for i in 0..my_bridge.len() {
            self.one_sided_connect(my_bridge[i].tile, my_bridge[i].gate, UniqTile {map: other, tile: other_bridge[i].tile}, other_bridge[i].gate, 0);
        }
    }
    
    // Unlike connect() functions, flip must be true on only one glued bridge for the result to be flipped.
    fn glue_one_side_flip(&mut self, my_bridge: &Vec<MapGate>, other: MapID, other_bridge: &Vec<MapGate>, first: u8) {
        if my_bridge.len() != other_bridge.len() {
            return;
        }
        let len = my_bridge.len();
        if first==0 {
            for i in 0..len {
                self.one_sided_connect(my_bridge[i].tile, my_bridge[i].gate, UniqTile {map: other, tile: other_bridge[len-i-1].tile}, other_bridge[len-i-1].gate, 1);
            }
        } else {
            for i in 0..len {
                self.one_sided_connect(my_bridge[len-i-1].tile, my_bridge[len-i-1].gate, UniqTile {map: other, tile: other_bridge[i].tile}, other_bridge[i].gate, 1);
            }
        }
    }
    
    // Make a straight bridge starting from a tile.
    // For use inside descendant map files; see examples there.
    fn straight_bridge(&mut self, tile: TileID, side_of_bridge: u8, length: usize, progress_along_forwards_dir: bool) -> Option<Vec<MapGate>> {
        let mut ctile = tile;
        let dir_forward = if progress_along_forwards_dir {3} else {2};
        let mut core = TraverserCore::from(0,side_of_bridge,0);
        let mut ret = Vec::new();
        ret.push(MapGate {tile: ctile, gate: core.gate_for(0)});
        for _i in 0..length-1 {
            let a = self.through(ctile, core.gate_for(dir_forward));
            match a {
                ThroughResult::Exists((x,y,z)) => {
                    if x.map == self.id() {
                        core = core.to(dir_forward,y,z);
                        ctile = x.tile;
                    } else {
                        return None;
                    }
                },
                _ => return None,
            }
            ret.push(MapGate {tile: ctile, gate: core.gate_for(0)});
        }
        return Some(ret);
    }
}
