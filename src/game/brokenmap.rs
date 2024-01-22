use std::io::Write;
use uuid::Uuid;

mod tile;
mod gate;

use crate::common::{TileStyle, REDSTYLE};
use tile::Tile;
use super::map::{MapGenType, Map};
use super::identifiers::UniqTile;

pub struct BrokenMap {
    // Each Map has a 'long' and 'short' id. The long id is the same across all
    // reloads, and is a randomly generated UUID.
    // Every map generated, within  or across games, should have a unique long id.
    // Short ids are generated in the lifetime of the program, and are different
    // between reloads. 
    pub long_id: Uuid,
    pub short_id: u32,
    // Note that the location of tiles is tile_id-1.
    // There is no tile with the id of 0.
    tiles: Vec<Tile>,
}

impl Map for BrokenMap {
    fn long_id(&self) -> Uuid {
        return self.long_id.clone();
    }
    
    fn short_id(&self) -> u32 {
        return self.short_id;
    }
    
    fn has_tile(&self, tile_id: u32) -> bool {
        return tile_id != 0 && tile_id as usize-1 < self.tiles.len();
    }
    
    // Extremely unsafe; does not check for connections
    fn one_sided_connect(&mut self, tile: u32, gate: u8, other: UniqTile, other_gate: u8, flip: u8) {
        if !self.has_tile(tile) {
            return;
        }
        self.tiles[tile as usize-1].set_gate(gate, other, other_gate, flip);
    }
    
    fn through(&self, tile: u32, gate: u8) -> (UniqTile, u8, u8) {
        if !self.has_tile(tile) {
            return (UniqTile {map: 0, tile: 0}, 0, 0)
        }
        return self.tiles[tile as usize-1].through(gate);
    }
    
    fn tile_connected(&self, tile: u32, gate: u8) -> bool {
        if !self.has_tile(tile) {
            return false;
        }
        return self.tiles[tile as usize-1].has_gate(gate);
    }
    
    fn serialize(&self, output: &mut dyn Write) {
        // Unimplemented
    }
    
    fn background_style(&self, tile: u32) -> TileStyle {
        TileStyle {ch:Some('.'),sty:REDSTYLE}
    }
    
    fn generate(&mut self, gen: &MapGenType) {
        match gen {
            MapGenType::Room(width, height) => self.make_room(*width as usize, *height as usize),
            _ => (),
        }
    }
}

impl BrokenMap {
    pub fn new(long_id: Uuid, short_id: u32, gen: &MapGenType) -> BrokenMap {
        let mut this = BrokenMap { long_id: long_id, short_id: short_id, tiles: Vec::new(), };
        this.generate(&gen);
        return this;
    }
    
    fn internal_connect(&mut self, first: u32, gate1: u8, second: u32, gate2: u8, flip: u8) {
        if !self.has_tile(first) || !self.has_tile(second) { return; }
        if self.tile_connected(first, gate1) || self.tile_connected(second, gate2) { return; }
        
        self.one_sided_connect(first, gate1, UniqTile {map: self.short_id, tile: second}, gate2, flip);
        self.one_sided_connect(second, gate2, UniqTile {map: self.short_id, tile: first}, gate1, flip);
    }
    
    pub fn find_unconnected(&mut self, tile: u32) -> [bool;4] {
        [!self.tile_connected(tile, 0),
        !self.tile_connected(tile, 1),
        !self.tile_connected(tile, 2),
        !self.tile_connected(tile, 3)]
    }
    
    pub fn new_tile(&mut self) -> UniqTile {
        let num = self.tiles.len() as u32;
        self.tiles.push(Tile::new(self.short_id));
        UniqTile {map: self.short_id, tile: num+1}
    }
    
    pub fn mini_hallway(&mut self, one: u32, two: u32, flip: u8) {
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
