
use crate::state_machine::Interrupt;
use crate::common::{SoundManager, ResourceHandler, TakeBox};
use crate::ui::UI;
use super::{Generator, Traverser, EntityEnum, Map, MapData, UIHandler, UpdateHandler, EntityHandler, ThroughResult,
            UniqTile, MapID, Update, Entity, ToTile, MapHandler};
use uuid::Uuid;
use crate::err;
use std::collections::HashMap;

pub struct GameData {
    pub save_id: Uuid,
    pub persistent: bool,
    pub world: MapHandler,
    pub entities: EntityHandler,
    pub updates: UpdateHandler,
    pub resources: TakeBox<ResourceHandler>,
    pub ui: UIHandler,
    pub sound: TakeBox<SoundManager>,
    pub gen: Generator
}

impl GameData {
    pub fn from_the_beginning(resources: Box<ResourceHandler>, ui: Box<UI>, sound: Box<SoundManager>) -> GameData {
        let mut this = GameData {
            save_id: Uuid::new_v4(),
            persistent: true,
            world: MapHandler::new(),
            entities: EntityHandler::new(),
            updates: UpdateHandler::new(),
            gen: Generator::new(),
            ui: UIHandler::new(ui),
            sound: TakeBox::newb(sound),
            resources: TakeBox::newb(resources),
        };
        // UI:
        this.ui.initial_setup();
        // RESOURCE HANDLER:
        let res = this.resources.as_mut().attach_to_game(&this.save_id.to_string());
        this.resources.as_mut().choke(err!(res), this.ui.writable.as_mut());
        // SOUND:
        this.sound.as_mut().background("mus_test", this.resources.as_mut());
        // MAP GENERATION:
        Generator::load_base_data(&mut this);
        let v = Generator::make("start", &mut this);
        let map_id = this.resources.as_mut().choke(v.ok_or("the starting map could not be created".to_string()), this.ui.writable.as_mut());
        if !this.entities.player_data.created {
            let traverser = Traverser::new(UniqTile { map: map_id, tile: this.world[map_id].map.random_tile() });
            EntityEnum::Player.call(&mut this, traverser, &None);
        }
        return this;
    }

    pub fn from_map(maps: &[&str], resources: Box<ResourceHandler>, ui: Box<UI>, sound: Box<SoundManager>) -> GameData {
        let mut this = GameData {
            save_id: Uuid::new_v4(),
            persistent: false,
            world: MapHandler::new(),
            entities: EntityHandler::new(),
            updates: UpdateHandler::new(),
            gen: Generator::new(),
            ui: UIHandler::new(ui),
            sound: TakeBox::newb(sound),
            resources: TakeBox::newb(resources),
        };
        // UI:
        this.ui.initial_setup();
        // SOUND:
        this.sound.as_mut().background("mus_test", this.resources.as_mut());
        // MAP GENERATION:
        Generator::load_base_data(&mut this);
        let v = Generator::start_branch(maps, &mut this);
        let map_id = this.resources.as_mut().choke(v.ok_or("the starting map could not be created".to_string()), this.ui.writable.as_mut());
        let traverser = Traverser::new(UniqTile { map: map_id, tile: this.world[map_id].map.random_tile() });
        EntityEnum::Player.call(&mut this, traverser, &None);
        return this;
    }

    pub fn from_save(save_name: Uuid, resources: Box<ResourceHandler>, ui: Box<UI>, sound: Box<SoundManager>) -> GameData {
        let mut this = GameData {
            save_id: save_name,
            persistent: true,
            world: MapHandler::new(),
            entities: EntityHandler::new(),
            updates: UpdateHandler::new(),
            gen: Generator::new(),
            // Does not set up new windows; we will be loading them in.
            ui: UIHandler::new(ui),
            sound: TakeBox::newb(sound),
            resources: TakeBox::newb(resources),
        };
        // RESOURCE HANDLER:
        let res = this.resources.as_mut().attach_to_game(&this.save_id.to_string());
        this.resources.as_mut().choke(err!(res), this.ui.writable.as_mut());
        // UI, MAPS, UPDATES, TEMPLATES:
        this.load_state();
        this.restore_ui_context();
        // SOUND:
        this.sound.as_mut().background("mus_test", this.resources.as_mut());
        // RELOADING COLORS AND RENUMBERING TEMPLATES:
        Generator::load_base_data(&mut this);
        return this;
    }

    pub fn travel(&self, trav: Traverser, dir: u8) -> Option<Traverser> {
        let gate_from = trav.core.gate_for(dir);
        let x = self.through(trav.tile, gate_from);
        if x.is_none() { return None; }
        let (tile_to, gate_to, flip) = x.unwrap();
        Some(Traverser {
            tile: tile_to,
            core: trav.core.to(dir, gate_to, flip)
        })
    }

    pub fn through(&self, tile: UniqTile, gate: u8) -> Option<ToTile> {
        if !self.world.tile_exists(tile) || !self.world.passable(tile) {
            return None;
        }
        match self.world.through(tile,gate) {
            ThroughResult::Exists(to) => return Some(to),
            _ => return None,
        }
    }

    /// the same as travel, but loads the map if it does not exist.
    pub fn load_travel(&mut self, trav: Traverser, dir: u8) -> Option<Traverser> {
        let gate_from = trav.core.gate_for(dir);
        let x = self.load_through(trav.tile, gate_from);
        if x.is_none() { return None; }
        let (tile_to, gate_to, flip) = x.unwrap();
        Some(Traverser {
            tile: tile_to,
            core: trav.core.to(dir, gate_to, flip)
        })
    }
    
    pub fn load_through(&mut self, tile: UniqTile, gate: u8) -> Option<ToTile> {
        if !self.world.tile_exists(tile) || !self.world.passable(tile) {
            return None;
        }
        self.world[tile.map].last_access = self.updates.current_time;
        let first = self.world.through(tile,gate);
        match first {
            ThroughResult::Exists(to) => return Some(to),
            ThroughResult::Load(map_id) => {
                match self.resources.as_mut().load(&(map_id.to_string()+".map")) {
                    Some(x) => {
                        self.load_map(map_id, x);
                    },
                    None => self.world.invalidate(map_id) // TODO: something
                }
            },
            ThroughResult::Generate(bridge_id) => {
                Generator::expand(tile.map, bridge_id, self);
            },
            ThroughResult::None => return None,
        };
        let second = self.world.through(tile,gate);
        match second {
            ThroughResult::Exists(to) => return Some(to),
            _ => return None,
        }
    }

    pub fn object_passable(&self, tile: UniqTile) -> bool {
        for obj in self.world.objects_on(tile) {
            if !self.entities.template(obj.template_id).passable {
                return false
            }
        }
        true
    }

    pub fn object_see_through(&self, tile: UniqTile) -> bool {
        for obj in self.world.objects_on(tile) {
            if !self.entities.template(obj.template_id).see_through {
                return false
            }
        }
        true
    }

    fn load_map(&mut self, map_id: usize, (mut map, mut entities): (Box<MapData>, Vec<(usize, Box<Entity>, Vec<Update>)>)) {
        let mut fucking_god_make_it_stop = HashMap::new();
        for (old_id, entity, updates) in entities.drain(..) {
            let new_id = self.entities.make_entity(*entity);
            fucking_god_make_it_stop.insert(old_id, new_id);
            self.updates.insert(new_id, updates);
        }
        for obj_list in map.objects.values_mut() {
            for obj in obj_list.iter_mut() {
                match obj.entity_id {
                    Some(old_id) => {
                        match fucking_god_make_it_stop.get(&old_id) {
                            Some(new_id) => obj.entity_id = Some(*new_id),
                            None => (),
                        }
                    },
                    None => (),
                }
            }
        }
        self.world.replace(map_id, map);
    }

    pub fn choke<T>(&mut self, result: Result<T, String>) -> T {
        return self.resources.as_mut().choke(result, self.ui.writable.as_mut());
    }

    pub fn next_update(&mut self) -> Result<(), Interrupt> {
        let some_update = self.updates.next();
        match some_update {
            Some((entity, upd)) => {
                upd.behavior.call(entity, self)?;
            },
            None => return Err(Interrupt::ForcedExit),
        };
        self.drop_maps();
        Ok(())
    }

    fn drop_map(&mut self, map: MapID) {
        if !self.persistent {
            return;
        }
        let mapdata = self.world.take(map);
        let mut entities = Vec::new();
        for obj_list in mapdata.objects.values() {
            for obj in obj_list.iter() {
                match obj.entity_id {
                    Some(entid) => {
                        let ent = self.entities.pack(entid, &mut self.updates).unwrap();
                        entities.push(ent);
                    },
                    None => (),
                }
            }
        }
        self.resources.as_mut().save(&(map.to_string()+".map"), &(mapdata, entities));
    }
    
    fn drop_maps(&mut self) {
        if !self.persistent {
            return;
        }
        let mut next = self.world.nonref_iter(0);
        while next.is_some() {
            let (i, mapdata) = next.unwrap();
            if self.updates.current_time > mapdata.last_access+400 {
                self.drop_map(i);
            }
            next = self.world.nonref_iter(i+1);
        }
    }

    pub fn save_state(&mut self) {
        if !self.persistent {
            return;
        }
        // Remove the UI from the UIHandler; we don't want to save it.
        let ui = self.ui.writable.take();
        let widgets = ui.get_context_widgets(1);
        let ui_handler = &self.ui;
        let loaded_maps = &self.world;
        let updates = &self.updates;
        let generation = &self.gen;
        let entities = &self.entities;
        self.resources.as_mut().save("current_state.save", &(widgets, loaded_maps, updates, generation, entities, ui_handler));
        self.ui.writable.replace(ui)
    }

    pub fn load_state(&mut self) {
        // Resources should already be inside the correct save folder
        let loaded = self.resources.as_mut().load("current_state.save");
        let (widgets, 
             loaded_maps, 
             updates,
             generation,
             entities, 
             ui_handler) = self.resources.as_mut().choke(loaded.ok_or("For some reason, this file could not load.".to_string()), self.ui.writable.as_mut());
        self.world = loaded_maps;
        self.updates = updates;
        self.gen = generation;
        self.entities = entities;
        let ui = self.ui.writable.take();
        self.ui = ui_handler;
        self.ui.writable.replace(ui);
        self.ui.writable.as_mut().replace_context_2(1, widgets);
    }
    
    pub fn take(&mut self) -> (Box<ResourceHandler>, Box<UI>, Box<SoundManager>) {
        return (self.resources.take(), self.ui.writable.take(), self.sound.take());
    }
    pub fn replace(&mut self, res: Box<ResourceHandler>, ui: Box<UI>, sound: Box<SoundManager>) {
        self.ui.writable.replace(ui);
        self.resources.replace(res);
        self.sound.replace(sound);
    }

    pub fn restore_ui_context(&mut self) {
        self.ui.writable.as_mut().set_context(1);
    }
}
