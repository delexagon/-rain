use super::{GameData, Traverser, Object, Entity};
use super::behaviors::*;
use macros::func_enum;
use serde_json::Value;


func_enum! {
#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash,serde::Serialize,serde::Deserialize)]
pub enum EntityEnum: fn(data: &mut GameData, trav: Traverser, info: &Option<Value>) {
    fn Player(data: &mut GameData, trav: Traverser, info: &Option<Value>) {
        // There is a global limit of only one player.
        if data.entities.player_data.created {
            return;
        }
        data.entities.player_data.created = true;
        let id = data.entities.next_id();
        // Creating the object
        let obj = Object {
            entity_id: Some(id), template_id: *data.gen.template_names.get("player").unwrap(),
        };
        data.world.create_obj(trav.tile, obj);

        // Creating the entity
        let entity = Entity {
            hp: Some(100),
            speed: 100,
            contains: Some(Vec::new()),
            loc: Some((obj, trav)),
        };
        let _entity_id = data.entities.make_entity(entity);
        
        // Adding an update
        data.updates.add_update(0, id, Behavior::PlayerStartingDraw);
        data.updates.add_update(50, id, Behavior::Player);
    }
}
}
