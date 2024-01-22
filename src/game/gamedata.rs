use std::collections::HashMap;
use std::collections::BinaryHeap;
use std::path::PathBuf;
use std::any::Any;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use crate::common::{
    Style, UITile, NORMALSTYLE, REDSTYLE, TileStyle, KeyAction,
    Communicator, GameMessage, UIMessage,
    DataBox,
};
use crate::ui::{UI, Widget, Action, Lines, CharArea, WindowManager, Key, KeyModifiers, KeyCode};
use super::map::Map;
use super::object::Object;
use super::entity::Entity;
use super::identifiers::{UniqTile, UniqObj};
use uuid::Uuid;

#[derive(Clone)]
pub enum GameAction {
    Exit,
} crate::to_action!(GameAction);

pub struct GameData {
    next_update_id: u32,
    current_time: u32,

    // No map is given the short id '0'.
    maps: Vec<Box<dyn Map>>,
    short_to_long: Vec<Uuid>,
    objects: Vec<HashMap<u32, Vec<Box<Object>>>>,
    styles: Vec<HashMap<u32, TileStyle>>,
    next_entity_id: u32,
    pub entities: HashMap<u32, Box<dyn Entity>>,
    // Update time (inverted), update id, object id
    update_heap: BinaryHeap<(u32, u32, UniqObj)>,
    
    debug_file: File,
    pub ui: DataBox<UI>,
    root_widget: usize,
    
    pub exiting: bool,
}

impl GameData {
    pub fn new() -> GameData {
        let mut this = GameData {
            maps: Vec::new(),
            short_to_long: Vec::new(),
            
            current_time: 0,
            next_update_id: 0,
            next_entity_id: 0,
            objects: Vec::new(),
            styles: Vec::new(),
            update_heap: BinaryHeap::new(),
            entities: HashMap::new(),
            ui: UI::new(),
            debug_file: File::create("debug.txt").unwrap(),
            root_widget: 0,
            
            exiting: false,
        };
        let my_widget = WindowManager::new();
        let x: Box<dyn Action> = Box::new(GameAction::Exit);
        my_widget.write().keymap(HashMap::from([
            (Key { code: KeyCode::Esc, modifiers: KeyModifiers::empty(), }, x),
        ]));
        this.root_widget = this.ui.write().add_widget(Box::new(my_widget));
        this.short_to_long.push(Uuid::nil());
        return this;
    }
    
    pub fn debug(&mut self, thing: &str) {
        self.debug_file.write(thing.as_bytes()).expect("Debugger error");
    }
    
    pub fn add_child_widget(&mut self, widget_id: usize) {
        self.ui.read().widget::<WindowManager>(self.root_widget).unwrap().write().add_child(widget_id);
    }
    
    pub fn add_child_widget_keep_context(&mut self, widget_id: usize) {
        self.ui.read().widget::<WindowManager>(self.root_widget).unwrap().write().add_child_leave_controlled(widget_id);
    }
    
    pub fn root_widget(&self) -> usize {self.root_widget}
    
    pub fn action_loop(&mut self) {
        while !self.exiting && self.update_heap.len() > 0 {
            let some_update = self.update_heap.pop();
            match some_update {
                Some((inv_time, upd_id, obj_id)) => {
                    if self.entities.contains_key(&obj_id.entity) {
                        let time = u32::MAX-inv_time;
                        self.current_time = time;
                        let act_fn = self.entities[&obj_id.entity].act();
                        act_fn(obj_id, self);
                    }
                },
                None => (),
            };
        }
    }
    
    pub fn action(&mut self) -> Option<Box<dyn Action>> {
        let action_any = UI::get_action(self.ui.clone());
        let action = action_any.any().downcast_ref::<GameAction>();
        match action {
            Some(GameAction::Exit) => {self.exiting = true; return None},
            None => return Some(action_any),
        }
    }
    
    pub fn ent_clone<T: Clone + 'static>(&self, ent_id: u32) -> Option<Box<T>> {
        if !self.entities.contains_key(&ent_id) {
            return None;
        }
        match self.entities[&ent_id].as_any().downcast_ref::<T>() {
            Some(ent) => return Some(Box::new(ent.clone())),
            None => return None,
        };
    }
    
    pub fn add_update(&mut self, time_until: u32, obj_id: UniqObj) -> u32 {
        let time_of_update = self.current_time+time_until;
        let inverted_time = u32::MAX-time_of_update;
        let update_id = self.next_update_id;
        self.next_update_id += 1;
        self.update_heap.push((inverted_time, update_id, obj_id));
        return update_id;
    }
    
    pub fn next_ent_id(&self) -> u32 {
        return self.next_entity_id;
    }
    
    pub fn replace_ent(&mut self, ent_id: u32, ent: Box<dyn Entity>) {
        if ent.get_id() != ent_id || !self.entities.contains_key(&ent_id) {
            return;
        }
        self.entities.insert(ent_id, ent);
    }
    
    // Note if the short map id isn't right, this function throws away the map.
    pub fn add_boxed_ent(&mut self, ent: Box<dyn Entity>) {
        if ent.get_id() != self.next_entity_id {
            return;
        }
        let this_id = self.next_entity_id;
        self.next_entity_id += 1;
        self.entities.insert(this_id, ent);
    }

    pub fn next_map_id(&self) -> (Uuid, u32) {
        return (Uuid::new_v4(), self.maps.len() as u32+1);
    }
    
    // Note if the short map id isn't right, this function throws away the map.
    pub fn add_boxed_map(&mut self, map: Box<dyn Map>) -> u32 {
        let short_id = map.short_id();
        if map.long_id() == Uuid::nil() || short_id != self.maps.len() as u32+1 {
            return 0;
        }
        self.short_to_long.push(map.long_id().clone());
        self.maps.push(map);
        
        self.objects.push(HashMap::new());
        self.styles.push(HashMap::new());
        return short_id;
    }
    
    // Any code not in Traverser that calls this function is a BUG!
    pub fn through(&self, tile: UniqTile, gate: u8) -> (UniqTile, u8, u8) {
        if !self.has_map(tile.map) {
            return (UniqTile {map: 0, tile: 0}, 0, 0)
        }
        return self.maps[tile.map as usize-1].through(tile.tile, gate);
    }
    
    pub fn get_long_id(&self, map_id: u32) -> Uuid {
        return self.short_to_long[map_id as usize].clone();
    }
    
    pub fn has_map(&self, short_id: u32) -> bool {
        return short_id != 0 && short_id-1 < self.maps.len() as u32;
    }
    
    pub fn tile_exists(&self, id: UniqTile) -> bool {
        return self.has_map(id.map) && self.maps[id.map as usize - 1].has_tile(id.tile)
    }
    
    pub fn get_tile(&self, short_id: u32, tile_id: u32) -> UniqTile {
        return UniqTile {map: short_id, tile: tile_id};
    }
    
    fn get_map_folder_path(long_id: &Uuid) -> PathBuf {
        let mut path = PathBuf::new(); // MAP_DATA_FOLDER_PATH.to_path_buf();
        path.push(long_id.to_string());
        return path;
    }
    
    pub fn drop(&mut self, long_id: &Uuid) {
        let map_path = Self::get_map_folder_path(&long_id);
        // Unimplemented
    }
    
    pub fn load(&mut self, long_id: &Uuid) {
        // Unimplemented
    }
    
    // Moves an object from tile1 to tile2
    pub fn move_object(&mut self, tile1: UniqTile, tile2: UniqTile, obj_id: UniqObj) -> UniqTile {
        if !self.tile_exists(tile1) {
            return UniqTile {map: 0, tile: 0};
        }
        let mut obj = None;
        if self.objects[tile1.map as usize-1].contains_key(&tile1.tile) {
            let objects_on_tile = &self.objects[tile1.map as usize-1][&tile1.tile];
            for index in 0..objects_on_tile.len() {
                let obj_here = &(objects_on_tile[index]);
                if obj_here.is(obj_id) {
                    obj = Some(self.objects[tile1.map as usize-1].get_mut(&tile1.tile).unwrap().swap_remove(index));
                    if self.objects[tile1.map as usize-1][&tile1.tile].len() == 0 {
                        self.objects[tile1.map as usize-1].remove(&tile1.tile);
                    }
                    break;
                }
            }
        }
        let mut obj_to_move;
        match obj {
            Some(object) => obj_to_move = object,
            None => return tile1,
        }
        self.create_obj(tile2, obj_to_move);
        return tile2;
    }
    
    pub fn create_obj(&mut self, tile: UniqTile, obj: Box<Object>) {
        if !self.objects[tile.map as usize-1].contains_key(&tile.tile) {
            self.objects[tile.map as usize-1].insert(tile.tile, Vec::new());
        }
        self.objects[tile.map as usize-1].get_mut(&tile.tile).unwrap().push(obj);
    }
    
    pub fn ui_tile(&self, tile: UniqTile) -> UITile {
        if !self.tile_exists(tile) {
            return UITile {ch: '#', sty: NORMALSTYLE};
        }
        let mut style = self.maps[tile.map as usize - 1].background_style(tile.tile);
        if self.objects[tile.map as usize-1].contains_key(&tile.tile) {
            for obj in self.objects[tile.map as usize-1][&tile.tile].iter() {
                style.mod_style(obj.style);
            }
        }
        return style.extract();
    }
    
    fn tile_can_connect(&self, tile: UniqTile, gate: u8) -> bool {
        if !self.has_map(tile.map) {
            return false;
        }
        return !self.maps[tile.map as usize - 1].tile_connected(tile.tile, gate);
    }
    
    pub fn connect(&mut self, first: UniqTile, gate1: u8, second: UniqTile, gate2: u8, flip: u8) {
        if !self.tile_exists(first) || !self.tile_exists(second) { return; }
        if !self.tile_can_connect(first, gate1) || !self.tile_can_connect(second, gate2) { return; }
        
        self.maps[first.map as usize-1].one_sided_connect(first.tile, gate1, second, gate2, flip);
        self.maps[second.map as usize-1].one_sided_connect(second.tile, gate2, first, gate1, flip);
    }
}
