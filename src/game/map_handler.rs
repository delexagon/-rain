mod maps;
mod traverser;
mod los;

pub use maps::*;
pub use los::*;
pub use traverser::Traverser;
use traverser::TraverserCore;
use super::generation::UsedByGeneration;
use super::identifiers::*;
use super::{Time, GameData};

use rand::Rng;
use crate::common::{TileStyle, TakeBox, UITile};
use std::collections::HashMap;
use std::ops::{Index, IndexMut};
use serde::{Serialize,Deserialize};

pub fn void_tile() -> UniqTile {
    let mut rng = rand::thread_rng();
    return UniqTile {map: 0, tile: rng.gen::<TileID>()};
}

pub type Bridge = Vec<MapGate>;

trait RemoveVecCompatible {
    fn has(&self, i: usize) -> bool;
    fn next(&self) -> usize;
}
impl<T> RemoveVecCompatible for Vec<T> {
    // TODO: This is completely broken.
    // Has needs to be dependent on whether the map is currently loaded or not,
    // including things that do object movement and stuff.
    fn has(&self, i: usize) -> bool {
        return i < self.len();
    }
    fn next(&self) -> usize {
        return self.len();
    }
}

// TODO: Create distinct layers for objects.
#[derive(Serialize,Deserialize)]
pub struct MapData {
    pub map: MapEnum,
    pub objects: HashMap<TileID, Vec<Object>>,
    pub last_access: Time,
}

#[derive(Serialize,Deserialize)]
pub struct MapHandler {
    maps: Vec<TakeBox<MapData>>
}

impl MapHandler {
    pub fn new() -> Self {
        let mut this = Self {
            maps: Vec::new(),
        };

        this.maps.push(TakeBox::new(MapData {
            map: VoidMap::new(0).into(),
            objects: HashMap::new(),
            last_access: 0
        }));
        return this;
    }

    /// Iter consumes the surrounding container of the vec,
    /// so this just returns the iterator index.
    /// Only returns maps which are currently loaded.
    pub fn nonref_iter(&self, x: usize) -> Option<(usize, &MapData)> {
        for i in x..self.maps.len() {
            if self.maps[i].is_some() {
                return Some((i, self.maps[i].as_ref()));
            }
        }
        return None;
    }

    pub fn iter(&self) -> std::slice::Iter<'_, TakeBox<MapData>> {
        return self.maps.iter();
    }
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, TakeBox<MapData>> {
        return self.maps.iter_mut();
    }

    pub fn next_map(&self) -> MapData {
        MapData {
            map: UninitializedMap.into(),
            objects: HashMap::new(),
            last_access: 0
        }
    }
    /// Used for maps which were, for some reason, not loaded
    /// correctly from a file.
    pub fn invalidate(&mut self, id: MapID) {
        self.maps[id].replace(Box::new(MapData {
            map: InvalidMap.into(),
            objects: HashMap::new(),
            last_access: 0
        }));
    }
    pub fn finalize(&mut self, mapdata: MapData) -> usize {
        self.maps.push(TakeBox::new(mapdata));
        return self.maps.len()-1;
    }
    
    pub fn next(&self) -> MapID {
        return self.maps.next();
    }
    
    pub fn take(&mut self, id: usize) -> Box<MapData> {
        return self.maps[id].take();
    }
    pub fn replace(&mut self, id: usize, map: Box<MapData>) {
        return self.maps[id].replace(map);
    }
    
    pub fn passable(&self, tile: UniqTile) -> bool {
        return self.maps[tile.map as usize].as_ref().map.passable(tile.tile);
    }
    
    // Any code not in TraverserCore that calls this function is a BUG!
    // ALL map movement, even IN MAP FILES, MUST be interpreted by traverser!
    pub fn through(&self, tile: UniqTile, gate: u8) -> ThroughResult {
        if !self.tile_exists(tile) {
            return ThroughResult::Exists((void_tile(),0,0));
        }
        let result = self.maps[tile.map].as_ref().map.through(tile.tile, gate);
        match result {
            ThroughResult::Exists((t2,_a,_b)) => {
                if self.maps[t2.map].is_none() {
                    return ThroughResult::Load(t2.map);
                } else if let MapEnum::InvalidMap(_) = self.maps[t2.map].as_ref().map {
                    return ThroughResult::Exists((void_tile(),0,0));
                } else {
                    return result;
                }
            },
            _ => return result,
        }
    }
    
    pub fn has_map(&self, id: MapID) -> bool {
        return self.maps.has(id);
    }
    
    pub fn tile_exists(&self, id: UniqTile) -> bool {
        return self.has_map(id.map) && self.maps[id.map].as_ref().map.has_tile(id.tile)
    }
    
    pub fn background(&self, tile: UniqTile) -> TileStyle {
        let style = self.maps[tile.map].as_ref().map.background_style(tile.tile);
        if !self.tile_exists(tile) {
            return TileStyle {fg: None, bg: None};
        }
        return style;
    }
    
    pub fn glue_bridge(&mut self, map1: MapID, bridge1: &Vec<MapGate>, map2: MapID, bridge2: &Vec<MapGate>, flip: bool) {
        if !flip {
            self.maps[map1].as_mut().map.glue_one_side(bridge1, map2, bridge2);
            self.maps[map2].as_mut().map.glue_one_side(bridge2, map1, bridge1);
        } else {
            self.maps[map1].as_mut().map.glue_one_side_flip(bridge1, map2, bridge2, 0);
            self.maps[map2].as_mut().map.glue_one_side_flip(bridge2, map1, bridge1, 1);
        }
    }
    
    pub fn connect(&mut self, first: UniqTile, gate1: u8, second: UniqTile, gate2: u8, flip: u8) {
        if !self.tile_exists(first) || !self.tile_exists(second) { return; }
        
        self.maps[first.map].as_mut().map.one_sided_connect(first.tile, gate1, second, gate2, flip);
        self.maps[second.map].as_mut().map.one_sided_connect(second.tile, gate2, first, gate1, flip);
    }
    
    // Attempts to move an object from tile1 to tile2
    // If tile1 does not exist, or the specified object does not exist on it, returns a random void tile
    // If tile2 does not exist, returns tile1 and does not move the object
    // Otherwise, changes the tile of the object and returns tile2.
    pub fn move_object(&mut self, tile1: UniqTile, tile2: UniqTile, obj_id: Object) -> UniqTile {
        let mut obj = None;
        if !self.maps.has(tile1.map) {
            return void_tile();
        }
        let map_objects = &mut self.maps[tile1.map].as_mut().objects;
        let mut objects_remain = true;
        match map_objects.get_mut(&tile1.tile) {
            None => return void_tile(),
            Some(objects) => {
                for index in 0..objects.len() {
                    if objects[index] == (obj_id) {
                        obj = Some(objects.swap_remove(index));
                        if objects.len() == 0 {
                            objects_remain = false;
                        }
                        break;
                    }
                }
            }
        }
        if !objects_remain {
            map_objects.remove(&tile1.tile);
        }

        match obj {
            Some(object) => {
                return self.create_obj(tile2, object);
            }
            None => return void_tile(),
        }
    }
    
    pub fn create_obj(&mut self, tile: UniqTile, obj: Object) -> UniqTile {
        if !self.maps.has(tile.map) || !self.maps[tile.map].as_ref().map.has_tile(tile.tile) {
            return void_tile();
        }
        let map_objects = &mut self.maps[tile.map].as_mut().objects;
        
        match map_objects.get_mut(&tile.tile) {
            Some(objects) => {
                objects.push(obj);
            },
            None => {
                let v = vec!(obj);
                map_objects.insert(tile.tile, v);
            }
        }
        return tile;
    }

    pub fn find_object(&mut self, tile: UniqTile, obj: Object) -> Option<&mut Object> {
        for maybe_obj in self.maps[tile.map].as_mut().objects.get_mut(&tile.tile)? {
            if *maybe_obj == obj {
                return Some(maybe_obj);
            }
        }
        None
    }
    
    pub fn objects_on(&self, tile: UniqTile) -> &[Object] {
        const EMPTY_VEC: [Object; 0] = [];
        if !self.maps.has(tile.map) {
            return &EMPTY_VEC;
        }
        if let Some(v) = self.maps[tile.map].as_ref().objects.get(&tile.tile) {
            return &v;
        }
        return &EMPTY_VEC;
    }
    pub fn objects_mut(&mut self, tile: UniqTile) -> Option<&mut Vec<Object>> {
        if !self.maps.has(tile.map) {
            return None;
        }
        return self.maps[tile.map].as_mut().objects.get_mut(&tile.tile);
    }
}

impl Index<MapID> for MapHandler {
    type Output = MapData;

    fn index(&self, index: usize) -> &Self::Output {
        return self.maps[index].as_ref();
    }
}

impl IndexMut<MapID> for MapHandler {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        return self.maps[index].as_mut();
    }
}
