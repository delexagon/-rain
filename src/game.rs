mod map;
mod vision;
mod gamedata;
mod brokenmap;
mod traverser;
mod identifiers;
mod object;
mod entity;
mod map_handler;

use crate::common::{Communicator, GameMessage, UIMessage};

use map::MapGenType;
use object::{ObjTraverser, Object};
use traverser::Traverser;
use brokenmap::BrokenMap;
use gamedata::GameData;
use entity::You;

pub fn game_start() {
    let mut data = GameData::new();
    let (lid, sid) = data.next_map_id();
    let mut map = Box::new(BrokenMap::new(lid, sid, &MapGenType::Room(10,5)));
    map.mini_hallway(2,7, 1);
    map.mini_hallway(20,46, 0);
    let map_id = data.add_boxed_map(map);
    let ent = Box::new(You::new(data.next_ent_id(), Traverser::new(data.get_tile(map_id, 1)), &mut data));
    data.add_boxed_ent(ent);
    data.action_loop();
}
