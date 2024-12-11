use rand::Rng;
use serde::{Serialize,Deserialize};

use crate::common::TileStyle;
use super::{Map, UniqTile, TileID, void_tile, MapID, ThroughResult};

#[derive(Serialize,Deserialize)]
/// A placeholder for when a map is either not currently loaded,
/// or is unable to load from the predicted file for some reason.
/// As long as the map is invalid, the game will attempt to reload it
/// whenever it sees that it should be loaded (may be too much spam?)
pub struct InvalidMap;
#[derive(Serialize,Deserialize)]
/// The value maps have before being initialized.
/// This should NEVER appear, and if it pops up it is an error most
/// likely in generation.rs
pub struct UninitializedMap;

impl Map for InvalidMap {
    fn id(&self) -> MapID {0}
    fn has_tile(&self, _tile_id: TileID) -> bool {true}
    fn passable(&self, _tile: TileID) -> bool {true}
    fn one_sided_connect(&mut self, _tile: TileID, _gate: u8, _other: UniqTile, _other_gate: u8, _flip: u8) {}
    fn random_tile(&self) -> TileID {rand::thread_rng().gen()}
    fn through(&self, _a: TileID, _b: u8) -> ThroughResult {ThroughResult::Exists((void_tile(), 0, 0))}
    fn tile_connected(&self, _tile: TileID, _gate: u8) -> bool {true}
    fn background_style(&self, _tile: TileID) -> TileStyle {TileStyle {fg: None,bg: None}}
}
impl Map for UninitializedMap {
    fn id(&self) -> MapID {0}
    fn has_tile(&self, _tile_id: TileID) -> bool {true}
    fn passable(&self, _tile: TileID) -> bool {true}
    fn one_sided_connect(&mut self, _tile: TileID, _gate: u8, _other: UniqTile, _other_gate: u8, _flip: u8) {}
    fn random_tile(&self) -> TileID {rand::thread_rng().gen()}
    fn through(&self, _a: TileID, _b: u8) -> ThroughResult {ThroughResult::Exists((void_tile(), 0, 0))}
    fn tile_connected(&self, _tile: TileID, _gate: u8) -> bool {true}
    fn background_style(&self, _tile: TileID) -> TileStyle {TileStyle {fg: None,bg: None}}
}
