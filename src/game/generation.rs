use super::{GameData, MapData, Map, UniqTile, Traverser, EntityEnum, Object, Template, map_handler::{Bridge, SparseMap, EuclidMap}};
use serde::{Serialize,Deserialize};
use crate::filesystem::from_json;
use crate::common::Array2D;
use crate::errstr;
use std::sync::OnceLock;
use uuid::Uuid;

use std::mem::replace;
use std::collections::HashMap;

mod structs;
mod color_generation;
pub use structs::*;

// Ironically, it turns out making a static global variable
// is much, MUCH easier than trying to send state to serde.
// Thanks for nothing, serde.
pub static COLORS: OnceLock<HashMap<String, ColorType>> = OnceLock::new();

fn inverse_len(x: usize) -> usize {
    return usize::MAX-x;
}

#[derive(Serialize,Deserialize)]
pub struct Generator {
    pub template_names: HashMap<String, usize>,
    pub generation: Vec<UsedByGeneration>,
    /// Generation is generations for existing maps,
    /// which are in line with a map of the same id in data.world.  
    /// This variable stores generations which are just meant to
    /// connect multiple maps together without generating world material
    /// on its own.
    pub generation_without_map: Vec<UsedByGeneration>
}

impl Generator {
    pub fn new() -> Self {
        Self {
            template_names: HashMap::new(),
            // TODO: Unshittify this
            generation: vec!(UsedByGeneration::default()),
            generation_without_map: Vec::new()
        }
    }

    pub fn template_object(&self, name: &str) -> Option<Object> {
        if let Some(template) = self.template_names.get(name) {
            return Some(Object {template_id: *template, entity_id: None});
        }
        None
    }
    
    pub fn load_base_data(data: &mut GameData) {
        let a = from_json::<HashMap<String, ColorType>>(&data.resources.as_ref().path.colors.clone(), data.resources.as_mut());
        COLORS.set(data.choke(a.ok_or(errstr!("los datos JSON para colores son inválidos"))));
        
        let b = from_json::<Vec<Template>>(&data.resources.as_ref().path.templates.clone(), data.resources.as_mut());
        let mut templates = data.choke(b.ok_or(errstr!("los datos JSON para plantillas son inválidos")));
        for template in templates.drain(..) {
            if !data.gen.template_names.contains_key(&template.name) {
                let name = template.name.to_string();
                let id = data.entities.add_template(template);
                data.gen.template_names.insert(name, id);
            }
        }
    }

    fn used<'a>(&'a self, id: usize) -> &'a UsedByGeneration {
        return if id < self.generation.len() {
            &self.generation[id]
        } else {
            &self.generation_without_map[inverse_len(id)]
        };
    }
    fn used_mut<'a>(&'a mut self, id: usize) -> &'a mut UsedByGeneration {
        return if id < self.generation.len() {
            &mut self.generation[id]
        } else {
            &mut self.generation_without_map[inverse_len(id)]
        };
    }

    fn find_connection<'a>(top_id: usize, bridge_name: &str, data: &'a GameData) -> Option<(&'a str, bool, usize)> {
        let mut cur_id = top_id;
        let answer = loop {
            let cur_map = data.gen.used(cur_id);
            if let Some(x) = cur_map.bridge_connect.get(bridge_name) {
                break x;
            }
            if let Some(parent) = cur_map.parent {
                cur_id = parent;
            } else {
                return None;
            }
        };
        return Some((&answer.0, answer.1, cur_id));
    }

    fn find_child_bridge(top_id: usize, bridge_name: &str, data: &mut GameData) -> Option<(usize, usize)> {
        use NameOrID::{Ungenerated, Generated};
        use BridgeLocation::{Child, Here};
        let mut cur_id = top_id;
        let bridge_id = loop {
            match data.gen.used(cur_id).bridge_to.get(bridge_name) {
                Some(Child(v)) => {
                    let v = *v;
                    match &data.gen.used(cur_id).children[v] {
                        Ungenerated(gen_name) => {
                            let god_please_save_me_from_this_hell = gen_name.clone();
                            if let Some(child_id) = Self::create_map(Some(cur_id), &god_please_save_me_from_this_hell, data) {
                                data.gen.used_mut(cur_id).children[v] = Generated(child_id);
                                cur_id = child_id;
                            } else {
                                return None;
                            }
                        },
                        Generated(id) => cur_id = *id
                    }
                },
                Some(Here(bridge_id)) => break *bridge_id,
                None => return None
            };
        };
        return Some((cur_id, bridge_id));
    }

    fn connect(map1: usize, bridge_id1: usize, map2: usize, bridge_id2: usize, flip: bool, data: &mut GameData) {
        let (name1, bridge1) = replace(&mut data.gen.generation[map1].bridges[bridge_id1], (String::with_capacity(0), Vec::with_capacity(0)));
        let (name2, bridge2) = replace(&mut data.gen.generation[map2].bridges[bridge_id2], (String::with_capacity(0), Vec::with_capacity(0)));
        data.world.glue_bridge(map1, &bridge1, map2, &bridge2, flip);
        // These must both have a map in order for this to work.
        // (implied by: there is no way to create a Here(id) on a map without mapgen)
        data.gen.generation[map1].bridges[bridge_id1] = (name1, bridge1);
        data.gen.generation[map2].bridges[bridge_id2] = (name2, bridge2);
    }

    pub fn expand(map_id: usize, bridge_id: usize, data: &mut GameData) {
        if data.gen.used(map_id).bridges.len() == 0 {
            return;
        }
        let bridge_name = &data.gen.used(map_id).bridges[bridge_id].0;
        if let Some((other_name, flip, top_id)) = Self::find_connection(map_id, &bridge_name, data) {
            let other_owned = other_name.to_owned();
            if let Some((to_id, other_bridge_id)) = Self::find_child_bridge(top_id, &other_owned, data) {
                Self::connect(map_id, bridge_id, to_id, other_bridge_id, flip, data);
            }
        }
    }

    /// Needs to be a list from the top overmap to the starting map
    pub fn start_branch(gens: &[&str], data: &mut GameData) -> Option<usize> {
        if gens.len() == 0 {
            return None;
        }
        let first_id = Self::create_map(None, gens[0], data);
        let mut prev_parent = if let Some(id) = first_id {id} else {return None;};
        use NameOrID::{Ungenerated, Generated};
        for next_wanted in &gens[1..] {
            let mut next_loc = None;
            for (i, child) in data.gen.used(prev_parent).children.iter().enumerate() {
                if let Ungenerated(gen_name) = child {
                    if gen_name == next_wanted {
                        next_loc = Some(i);
                        break;
                    }
                }
            }
            if let Some(i) = next_loc {
                if let Some(next_id) = Self::create_map(Some(prev_parent), next_wanted, data) {
                    data.gen.used_mut(prev_parent).children[i] = Generated(next_id);
                    prev_parent = next_id;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        return Some(prev_parent);
    }

    // TODO: ALL BRIDGES THAT CAN BE CONNECTED SHOULD LOAD WHEN A MAP LOADS
    // TODO: ALL BRIDGES THAT CAN BE CONNECTED SHOULD LOAD WHEN A MAP LOADS
    // TODO: ALL BRIDGES THAT CAN BE CONNECTED SHOULD LOAD WHEN A MAP LOADS
    fn create_map(parent: Option<usize>, gen_name: &str, data: &mut GameData) -> Option<usize> {
        let GenerationData {
            mapgen,
            templates,
            mut contains,
            mut connect
        } = from_json(&data.resources.as_ref().path.maps.join(gen_name.to_string()+".json"), data.resources.as_mut())?;
        
        if let Some(mut templates) = templates {
            for template in templates.drain(..) {
                let name = template.name.to_string();
                let id = data.entities.add_template(template);
                data.gen.template_names.insert(name, id);
            }
        }
        
        let mut mapdata = data.world.next_map();
        let mut for_generation = UsedByGeneration::default();
        mapdata.last_access = data.updates.current_time;
        let map_id = data.world.next();
        for_generation.parent = parent;
        for (s1, s2, flip) in connect.drain(..) {
            for_generation.bridge_connect.insert(s1.clone(), (s2.clone(), flip));
            for_generation.bridge_connect.insert(s2, (s1, flip));
        }
        use BridgeLocation::Child;
        use NameOrID::Ungenerated;
        for (id, (map, mut child_bridges)) in contains.drain().enumerate() {
            for_generation.children.push(Ungenerated(map));
            for name in child_bridges.drain(..) {
                for_generation.bridge_to.insert(name, Child(id));
            }
        }
        use BridgeLocation::Here;
        if let Some(mapgen) = mapgen {
            let entv = match mapgen {
                MapGen::Euclid(ref inside_mapgen) => {
                    let (map, bridges) = EuclidMap::build(map_id, &inside_mapgen, data);
                    mapdata.map = map;
                    for (id, (name, bridge)) in bridges.iter().enumerate() {
                        for_generation.bridge_to.insert(name.clone(), Here(id));
                    }
                    for_generation.bridges = bridges;
                    EuclidMap::add_objects(map_id, &mut mapdata, &inside_mapgen.object_maps, &inside_mapgen.object_key, data)
                },
                MapGen::Sparse(inside_mapgen) => {
                    let (map, bridges) = SparseMap::build(map_id, &inside_mapgen, data);
                    mapdata.map = map;
                    for (id, (name, bridge)) in bridges.iter().enumerate() {
                        for_generation.bridge_to.insert(name.clone(), Here(id));
                    }
                    for_generation.bridges = bridges;
                    Vec::new()
                }
            };
    
            data.gen.generation.push(for_generation);
            // Also pushes bridges and the name into here
            let ret =  data.world.finalize(mapdata);
            for (create_ent, trav, info) in entv.iter() {
                create_ent.call(data, *trav, info);
            }
            return Some(ret);
        } else {
            // TODO: fix this
            let i = data.gen.generation_without_map.len();
            data.gen.generation_without_map.push(for_generation);
            return Some(inverse_len(i));
        }
    }
    
    pub fn make(gen_name: &str, data: &mut GameData) -> Option<usize> {
        Self::create_map(None, gen_name, data)
    }
}
