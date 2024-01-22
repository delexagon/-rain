use std::io::Write;

use uuid::Uuid;

use crate::common::{
    Style, UITile, TileStyle, NORMALSTYLE, REDSTYLE,
    Communicator, GameMessage, UIMessage,
};
use super::identifiers::{UniqTile, UniqObj};
use super::gamedata::GameData;
use super::traverser::Traverser;
//use crate::options::MAP_DATA_FOLDER_PATH;

pub enum MapGenType {
    Room(u16, u16),
}

// Map ettiquette:
// 1. Maps should be made of all contiguous tiles, so that tiles are not loaded unnecessarily.
// 2. Maps should be fairly small, because they are loaded all at once.
// 3. 
pub trait Map {
    fn long_id(&self) -> Uuid;
    fn short_id(&self) -> u32;
    fn has_tile(&self, tile: u32) -> bool;
    fn one_sided_connect(&mut self, tile: u32, gate: u8, other: UniqTile, other_gate: u8, flip: u8);
    // Any code not in Traverser or GameData that calls this function is a BUG!
    fn through(&self, tile: u32, gate: u8) -> (UniqTile, u8, u8);
    fn tile_connected(&self, tile: u32, gate: u8) -> bool;
    fn serialize(&self, output: &mut dyn Write);
    fn background_style(&self, tile: u32) -> TileStyle;
    
    // Past this point, functionality is map dependent.
    // Some functions run on the incorrect type of map may have no effect.
    // This is so that these functionalities are accessible
    fn generate(&mut self, gen: &MapGenType) {}
}

