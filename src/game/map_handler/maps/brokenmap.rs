use std::io::Write;
use uuid::Uuid;

use crate::common::{Style, TileStyle, Fg};
use super::{MapGenType, Map, UniqTile, Tile, MapID, TileID, MapGate, TraverserCore};

pub struct BrokenMap {
    // Each Map has a 'long' and 'short' id. The long id is the same across all
    // reloads, and is a randomly generated UUID.
    // Every map generated, within  or across games, should have a unique long id.
    // Short ids are generated in the lifetime of the program, and are different
    // between reloads. 
    pub long_id: Uuid,
    pub short_id: MapID,
    // Note that the location of tiles is tile_id-1.
    // There is no tile with the id of 0.
    tiles: Vec<Tile>,
}

impl Map for BrokenMap {
    fn long_id(&self) -> Uuid {
        return self.long_id.clone();
    }
    
    fn short_id(&self) -> MapID {
        return self.short_id;
    }
    
    fn has_tile(&self, tile_id: TileID) -> bool {
        return tile_id != 0 && tile_id as usize-1 < self.tiles.len();
    }
    
    // Extremely unsafe; does not check for connections
    fn one_sided_connect(&mut self, tile: TileID, gate: u8, other: UniqTile, other_gate: u8, flip: u8) {
        if !self.has_tile(tile) {
            return;
        }
        self.tiles[tile as usize-1].set_gate(gate, other, other_gate, flip);
    }
    
    fn through(&self, tile: TileID, gate: u8) -> Option<(UniqTile, u8, u8)> {
        if !self.has_tile(tile) {
            return None;
        }
        return self.tiles[tile as usize-1].through(gate);
    }
    
    fn tile_connected(&self, tile: TileID, gate: u8) -> bool {
        if !self.has_tile(tile) {
            return false;
        }
        return self.tiles[tile as usize-1].has_gate(gate);
    }
    
    fn serialize(&self, output: &mut dyn Write) {
        // Unimplemented
    }
    
    fn background_style(&self, tile: TileID) -> TileStyle {
        TileStyle {
            fg: Some(Fg {
                ch: '.',
                color: (255,255,255),
                bold: false,
                ital: false,
            }),
            bg: None
        }
    }
}

impl BrokenMap {
    pub fn new(long_id: Uuid, short_id: MapID) -> BrokenMap {
        let mut this = BrokenMap { long_id: long_id, short_id: short_id, tiles: Vec::new(), };
        return this;
    }
    
    // GUARDED BASED ON CONNECTIONS; WILL NOT UNDO EXISTING CONNECTIONS!
    fn internal_connect(&mut self, first: TileID, gate1: u8, second: TileID, gate2: u8, flip: u8) {
        if !self.has_tile(first) || !self.has_tile(second) { return; }
        if self.tile_connected(first, gate1) || self.tile_connected(second, gate2) { return; }
        
        self.one_sided_connect(first, gate1, UniqTile {map: self.short_id, tile: second}, gate2, flip);
        self.one_sided_connect(second, gate2, UniqTile {map: self.short_id, tile: first}, gate1, flip);
    }
    
    pub fn new_tile(&mut self) -> UniqTile {
        let num = self.tiles.len() as TileID;
        self.tiles.push(Tile::new(self.short_id));
        UniqTile {map: self.short_id, tile: num+1}
    }
    
    pub fn mini_hallway(&mut self, one: TileID, two: TileID, flip: u8) {
        let new_tile = self.new_tile();
        let a = self.find_unconnected(one);
        let b = self.find_unconnected(two);
        let mut gate1 = 4;
        let mut gate2 = 4;
        for i in 0..4 {
            if a[i as usize] {
                gate1 = i;
            }
            if b[i as usize] {
                gate2 = i;
            }
        }
        if gate1 != 4 && gate2 != 4 {
            self.internal_connect(new_tile.tile, 0, one, gate1, 0);
            self.internal_connect(new_tile.tile, 1, two, gate2, flip);
        }
    }
    
    pub fn bridge(&self, tile: TileID, side: Option<u8>, length: usize) -> Option<Vec<MapGate>> {
        let gate = if side.is_some() {side.unwrap()} else {self.first_unconnected(tile)};
        return self.straight_bridge(tile, gate, length, true);
    }
    
    pub fn make_room(&mut self, width: usize, height: usize) {
        let mut retvec = Vec::new();
        for _ in 0..height*width {
            retvec.push(self.new_tile());
            let len = retvec.len();
            if len > width {
                self.internal_connect(retvec[len-1].tile, 0, retvec[len-1-width].tile, 1, 0);
            }
            if (len-1)%width > 0 {
                self.internal_connect(retvec[len-1].tile, 2, retvec[len-2].tile, 3, 0);
            }
        }
    }
}
