use std::iter::zip;
use serde::{Serialize,Deserialize};
use serde_json::Value;
use std::collections::{HashSet, HashMap};
use crate::game::{entity_handler::EntityEnum, generation::{DynTile, EuclidMapGen, ObjectGen}};
use super::{MapGate, GateMap, MapEnum, MapData, ThroughResult, Gate, TileID, Traverser, TileStyle, Bridge, Map, GameData, MapID, UniqTile};
use crate::common::{Array2D, TakeBox, item_iterate, StraightBridge, SideToCoord, AdjacentFloorIter};
use rand::Rng;
use crate::errstr;

fn invert(gate: u8) -> u8 {
    return gate^1;
}

#[derive(Serialize,Deserialize)]
pub struct EuclidMap {
    pub id: MapID,
    space: Array2D<u8>,
    gatemap: GateMap,
    default_wall: DynTile,
    floors: Vec<DynTile>
}

impl Map for EuclidMap {
    fn id(&self) -> MapID {
        return self.id;
    }
    
    fn has_tile(&self, tile: TileID) -> bool {
        return tile < self.space.len() as TileID;
    }

    fn random_tile(&self) -> TileID {
        let mut rng = rand::thread_rng();
        for _ in 0..1000 {
            let rand_tile = rng.gen_range(0..self.space.len());
            if self.passable(rand_tile as TileID) {
                return rand_tile as TileID;
            }
        }
        return 0;
    }
    
    fn passable(&self, tile: TileID) -> bool {
        return self.space[tile as usize] > 0;
    }
    
    fn one_sided_connect(&mut self, tile: TileID, gate: u8, other: UniqTile, other_gate: u8, flip: u8) {
        self.gatemap.insert(MapGate {tile: tile, gate: gate}, Gate::Matched((other, other_gate, flip)));
    }
    
    fn through(&self, tile: TileID, gate: u8) -> ThroughResult {
        let uniq_gate = MapGate {tile: tile, gate: gate};
        match self.gatemap.get(&uniq_gate) {
            Some(gate) => return gate.get(),
            None => {
                if self.space[tile as usize] == 0 {
                    return ThroughResult::None;
                }
                let next_tile = self.next(tile as usize, gate);
                if let Some(tile) = next_tile {
                    return ThroughResult::Exists((UniqTile {map: self.id, tile}, invert(gate), 0));
                } else {
                    return ThroughResult::None;
                }
            }
        };
    }
    
    fn tile_connected(&self, tile: TileID, gate: u8) -> bool {
        let next_tile = self.next(tile as usize, gate);
        return self.gatemap.contains_key(&MapGate {tile, gate}) || next_tile != None;
    }
    
    fn background_style(&self, tile: TileID) -> TileStyle {
        let floor_type = self.space[tile as usize];
        if floor_type == 0 {
            self.default_wall.gen(tile as usize)
        } else {
            self.floors[floor_type as usize-1].gen(tile as usize)
        }
    }
}

impl EuclidMap {
    pub fn new(id: MapID) -> Self {
        Self { id: id, space: Array2D::with_capacity(0), gatemap: HashMap::new(), default_wall: DynTile::default(), floors: Vec::new() }
    }

    fn next(&self, tile: usize, gate: u8) -> Option<TileID> {
        // x position: tile%width, y position: tile//width
        let x = tile as usize%self.space.width();
        let y = tile as usize/self.space.width();
        match gate {
            0 if y > 0 => Some((tile - self.space.width()) as TileID),
            1 if y < self.space.height()-1 => Some((tile + self.space.width()) as TileID),
            2 if x > 0 => Some(tile as TileID-1),
            3 if x < self.space.width()-1 => Some(tile as TileID+1),
            _ => None,
        }
    }

    fn tile(&self, t: (usize, usize)) -> usize {
        return self.space.width()*t.1+t.0;
    }
    
    fn internal_connect(&mut self, t1: (usize,usize), dir1: u8, t2: (usize,usize), dir2: u8, flip: u8) {
        self.gatemap.insert(MapGate {tile: self.tile(t1) as u32, gate: dir1}, Gate::Matched((UniqTile {map: self.id, tile: self.tile(t2) as u32}, dir2, flip)));
        self.gatemap.insert(MapGate {tile: self.tile(t2) as u32, gate: dir2}, Gate::Matched((UniqTile {map: self.id, tile: self.tile(t1) as u32}, dir1, flip)));
    }
    
    fn build_space(&mut self, bridges: &mut HashMap<char, StraightBridge>, arr: &Array2D<char>, floor_map: &HashMap<char, u8>, flipped: &HashSet<char>) {
        self.space = Array2D::new_sized(arr.width(), arr.height(), 0);
        for coord in arr.coord_iter() {
            if floor_map.contains_key(&arr[coord]) {
                self.space[coord] = floor_map[&arr[coord]] + 1;
            }
        }
        
        // TODO: Fix this to floor types
        for (ch, bridge) in AdjacentFloorIter::new(arr, |ch| floor_map.contains_key(ch), |ch| *ch == '#' || floor_map.contains_key(ch)) {
            if bridges.contains_key(&ch) {
                let flip = flipped.contains(&ch);
                let bridge2 = bridges.remove(&ch).unwrap();
                match (bridge, bridge2) {
                    (StraightBridge::Single(t1, dir1), StraightBridge::Single(t2, dir2)) => {
                        self.internal_connect(t1,dir1,t2,dir2,if flip {1} else {0});
                    },
                    (StraightBridge::Length(start1, end1, s1), StraightBridge::Length(start2, end2, s2)) => {
                        if !flip {
                            if let (Some(a), Some(b)) = (SideToCoord::new(start1,end1,s1), SideToCoord::new(start2,end2,s2)) {
                                for ((t1,dir1),(t2, dir2)) in zip(a,b) {
                                    self.internal_connect(t1,dir1,t2,dir2,0);
                                }
                            }
                        } else {
                            if let (Some(a), Some(b)) = (SideToCoord::new(start1,end1,s1), SideToCoord::new(start2,end2,s2)) {
                                for ((t1,dir1),(t2, dir2)) in zip(a,b.end().rev()) {
                                    self.internal_connect(t1,dir1,t2,dir2,1);
                                }
                            }
                        }
                    },
                    _ => ()
                }
            } else {
                bridges.insert(ch, bridge);
            }
        }
    }
    
    fn to_bridge(&mut self, bridge: &StraightBridge, num_bridge: usize) -> Bridge {
        match bridge {
            StraightBridge::Single(t, dir) => {
                self.gatemap.insert(MapGate {tile: self.tile(*t) as u32, gate: *dir}, Gate::Loose(num_bridge));
                return vec!(MapGate {tile: self.tile(*t) as TileID, gate: *dir});
            },
            StraightBridge::Length(start, end, s) => {
                let mut vec = Vec::new();
                for (t, dir) in SideToCoord::new(*start,*end,*s).unwrap() {
                    self.gatemap.insert(MapGate {tile: self.tile(t) as u32, gate: dir}, Gate::Loose(num_bridge));
                    vec.push(MapGate {tile: self.tile(t) as TileID, gate: dir});
                }
                return vec;
            }
        }
    }
    
    pub fn build(id: MapID, mapgen: &EuclidMapGen, data: &mut GameData) -> (MapEnum, Vec<(String, Bridge)>) {
        let mut this = Self::new(id);
        let mut floor_map = HashMap::new();
        for (i, floor) in mapgen.floors.iter().enumerate() {
            floor_map.insert(floor.ch, i as u8);
            this.floors.push(floor.tile.clone());
        }
        this.default_wall = mapgen.default_wall;
        let mut bridges: HashMap<char, StraightBridge> = HashMap::new();
        
        this.build_space(&mut bridges, &mapgen.space, &floor_map, &mapgen.flipped);
        
        let mut num_bridge = 0;
        let mut final_bridges = Vec::new();
        for conn in mapgen.bridges.iter() {
            if bridges.contains_key(&conn.ch) {
                let br = bridges.get(&conn.ch).unwrap();
                let bridge = this.to_bridge(br, num_bridge);
                final_bridges.push((conn.name.to_string(), bridge));
                num_bridge += 1;
            }
        }

        return (this.into(), final_bridges);
    }

    pub fn add_objects<'a>(map_id: usize, mapdata: &mut MapData, object_map: &Array2D<char>, object_key: &'a HashMap<char, ObjectGen>, data: &mut GameData) -> Vec<(EntityEnum, Traverser, &'a Option<Value>)> {
        let mut entv = Vec::new();
        for coord in object_map.coord_iter() {
            if let Some(gen) = object_key.get(&object_map[coord]) {
                let tile = UniqTile {map: map_id, tile: object_map.to_1d(coord) as TileID};
                if gen.entity.is_some() {
                    entv.push((*gen.entity.as_ref().unwrap(), Traverser::new(tile), &gen.entity_info));
                }
                if let Some(template_name) = &gen.object {
                    if let Some(object) = data.gen.template_object(&template_name) {
                        mapdata.objects.insert(tile.tile, vec!(object));
                    } else {
                        data.resources.as_mut().err(&errstr!("There was no template found that matched the template name"));
                    }
                }
            }
        }
        return entv;
    }
}

