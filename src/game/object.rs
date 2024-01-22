use super::gamedata::GameData;
use super::traverser::Traverser;
use super::identifiers::UniqObj;
use crate::common::TileStyle;

#[derive(Clone, Copy)]
pub struct Object {
    // Object ids are unique for their Entity id
    pub entity_id: u32,
    pub id: u32,
    pub style: TileStyle,
}

impl Object {
    pub fn is(&self, obj: UniqObj) -> bool {
        return self.entity_id == obj.entity && self.id == obj.obj;
    }
    
    pub fn id(&self) -> UniqObj {
        return UniqObj {entity: self.entity_id, obj: self.id};
    }
}

// A traverser following a particular object.
#[derive(Clone)]
pub struct ObjTraverser {
    traverser: Traverser,
    obj: UniqObj,
}

impl ObjTraverser {
    // In addition to creating the traverser, places the object on the map.
    pub fn new(traverser: Traverser, obj: Box<Object>, data: &mut GameData) -> ObjTraverser {
        let id = obj.id();
        data.create_obj(traverser.tile, obj);
        return ObjTraverser {traverser: traverser, obj: id};
    }
    
    pub fn traverser(&self) -> Traverser {
        return self.traverser;
    }

    pub fn on_map(&self, data: &GameData) -> bool {
        return self.traverser.tile_exists(data);
    }
    
    pub fn move_obj(&mut self, dir: u8, data: &mut GameData) {
        let to = self.traverser.travel(dir, &data);
        if !to.tile_exists(&data) {
            return;
        }
        data.move_object(self.traverser.tile, to.tile, self.obj);
        self.traverser = to;
    }
}
