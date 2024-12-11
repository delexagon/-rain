use std::iter::zip;
use serde::{Serialize,Deserialize};
use std::collections::{HashSet, HashMap};
use crate::game::{entity_handler::EntityEnum, generation::{DynTile, SparseMapGen, ObjectGen}};
use super::{MapGate, MapEnum, MapData, ThroughResult, Gate, TileID, Traverser, TileStyle, Bridge, Map, GateMap, GameData, MapID, UniqTile};
use crate::common::{Array2D, TakeBox, item_iterate, StraightBridge, SideToCoord, AdjacentFloorIter};
use rand::{thread_rng, Rng};
use crate::errstr;

const MAX: TileID = TileID::MAX;
const WIDTH: TileID = 1<<(TileID::BITS>>1);

fn invert(gate: u8) -> u8 {
    return gate^1;
}

#[derive(Serialize,Deserialize)]
pub struct SparseMap {
    width: TileID,
    id: MapID,
    gatemap: GateMap,
    floor: DynTile
}

impl Map for SparseMap {
    fn id(&self) -> MapID {
        return self.id;
    }
    
    fn has_tile(&self, tile_id: TileID) -> bool {
        return true;
    }
    
    fn one_sided_connect(&mut self, tile: TileID, gate: u8, other: UniqTile, other_gate: u8, flip: u8) {
        self.gatemap.insert(MapGate {tile: tile, gate: gate}, Gate::Matched((other, other_gate, flip)));
    }
    
    fn through(&self, tile: TileID, gate: u8) -> ThroughResult {
        let uniq_gate = MapGate {tile: tile, gate: gate};
        match self.gatemap.get(&uniq_gate) {
            Some(gate) => return gate.get(),
            None => {
                let next_tile = self.in_gate(tile, gate);
                if next_tile.is_none() || self.gatemap.contains_key(&MapGate {tile: next_tile.unwrap(), gate: gate^1}) {
                    return ThroughResult::None;
                } else {
                    return ThroughResult::Exists((UniqTile {map: self.id, tile: next_tile.unwrap()}, gate^1, 0));
                }
            }
        };
    }
    
    fn passable(&self, tile: TileID) -> bool {
        return true;
    }

    fn random_tile(&self) -> TileID {
        let mut rng = rand::thread_rng();
        return rng.gen();
    }
    
    fn tile_connected(&self, tile: TileID, gate: u8) -> bool {
        let next_tile = self.in_gate(tile, gate);
        return self.gatemap.contains_key(&MapGate {tile: tile, gate: gate}) || (next_tile.is_some() && !self.gatemap.contains_key(&MapGate {tile: next_tile.unwrap(), gate: invert(gate)}));
    }
    
    fn background_style(&self, tile: TileID) -> TileStyle {
        self.floor.gen(tile as usize)
    }
}

impl SparseMap {
    pub fn new(id: MapID, width: TileID) -> Self {
        Self { id, width: if width == 0 {WIDTH} else {width}, gatemap: HashMap::new(), floor: DynTile::default() }
    }

    fn height(&self) -> TileID {MAX/self.width}

    fn liminal_bridge(&mut self, num_bridge: usize, length: usize) -> Vec<MapGate> {
        let mut rng = thread_rng();
        let start = (rng.gen_range(0..self.width as usize), rng.gen_range(0..self.height() as usize));
        let (end, side) = match (rng.gen(), rng.gen()) {
            (false,false) => ((start.0, start.1+length-1), 2),
            (false,true) => ((start.0, start.1+length-1), 3),
            (true,false) => ((start.0+length-1, start.1), 0),
            (true,true) => ((start.0+length-1, start.1), 1),
        };
        let mut vec = Vec::with_capacity(length as usize);
        for (t, dir) in SideToCoord::new(start,end,side).unwrap() {
            self.gatemap.insert(MapGate {tile: self.tile(t) as TileID, gate: dir}, Gate::Loose(num_bridge));
            vec.push(MapGate {tile: self.tile(t) as TileID, gate: dir});
        }
        return vec;
    }

    fn tile(&self, coord: (usize,usize)) -> usize {self.width as usize*coord.1+coord.0}

    pub fn build(id: MapID, mapgen: &SparseMapGen, data: &mut GameData) -> (MapEnum, Vec<(String, Bridge)>) {
        let mut this = Self {
            id, width: if let Some(width) = mapgen.width {width as TileID} else {WIDTH}, gatemap: HashMap::new(), floor: DynTile::default()
        };
        this.floor = mapgen.floor;
        let mut bridges = Vec::new();
        for bridge in &mapgen.liminal_bridges {
            if bridge.length > 0 {
                bridges.push((bridge.name.to_string(), this.liminal_bridge(bridges.len(), bridge.length)));
            }
        }
        (this.into(), bridges)
    }
    
    fn in_gate(&self, tile: TileID, gate: u8) -> Option<TileID> {
        match gate {
            0 if tile >= self.width => Some(tile-self.width),
            1 if tile <= MAX-self.width => Some(tile+self.width),
            2 if tile > 0 && tile%self.width != 0 => Some(tile-1),
            3 if tile != MAX && (tile+1)%self.width != 0 => Some(tile+1),
            _ => None,
        }
    }
}
