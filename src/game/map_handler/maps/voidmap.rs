use rand::Rng;
use rand::rngs::ThreadRng;
use serde::{Serialize,Deserialize};

use crate::common::{TileStyle, Rgb, Fg};
use super::{Map, UniqTile, TileID, MapID, ThroughResult};

fn random_ascii(rng: &mut ThreadRng) -> char {
    const CHARSET: &[u8] = b"!@#$%^&*(){}. ><,;':\"/?\\[]`~-_=+|";
    return CHARSET[rng.gen_range(0..CHARSET.len())] as char;
}

#[derive(Serialize,Deserialize)]
pub struct VoidMap {
    pub id: MapID,
}

impl Map for VoidMap {
    fn id(&self) -> MapID {
        return self.id;
    }
    
    fn has_tile(&self, _tile_id: TileID) -> bool {
        return true;
    }
    
    fn passable(&self, _tile: TileID) -> bool {
        return true;
    }
    
    // This doesn't do anything, lmao
    fn one_sided_connect(&mut self, _tile: TileID, _gate: u8, _other: UniqTile, _other_gate: u8, _flip: u8) {
    }

    fn random_tile(&self) -> TileID {
        let mut rng = rand::thread_rng();
        return rng.gen();
    }
    
    fn through(&self, _tile: TileID, _gate: u8) -> ThroughResult {
        let mut rng = rand::thread_rng();
        if rng.gen::<u8>() < 10 {
            return ThroughResult::None;
        }
        let tile = rng.gen::<TileID>();
        let gate = rng.gen_range(0..4);
        let flip = rng.gen_range(0..2);
        return ThroughResult::Exists((UniqTile {map: self.id, tile: tile}, gate, flip));
    }
    
    fn tile_connected(&self, _tile: TileID, _gate: u8) -> bool {
        return true;
    }
    
    fn background_style(&self, _tile: TileID) -> TileStyle {
        let mut rng = rand::thread_rng();

        let ch = random_ascii(&mut rng);
        let bold = rng.gen::<bool>();
        let ital = rng.gen::<bool>();
        let fg = Rgb(rng.gen::<u8>(), rng.gen::<u8>(), rng.gen::<u8>());
        TileStyle {
            fg: Some(Fg {
                ch: ch,
                color: fg,
                bold: bold,
                ital: ital,
            }),
            bg: None
        }
    }
}

impl VoidMap {
    pub fn new(id: MapID) -> Self {
        Self { id: id }
    }
}
