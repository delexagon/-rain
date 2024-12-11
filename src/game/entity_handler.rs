use super::{Traverser, UIHandler, UniqTile, ObjTile, entity_handler::OnInteract, Object, ObjTraverser, UpdateHandler, Update, GameData};
use crate::state_machine::Interrupt;
use crate::common::{TileStyle, RemoveVec};
use serde::{Serialize,Deserialize};

mod behaviors;
pub use behaviors::*;
mod entities;
pub use entities::*;

fn no() -> bool {false}
fn yes() -> bool {true}

pub type EntityID = usize;
pub type TemplateID = usize;

#[derive(Serialize,Deserialize)]
pub struct Template {
    pub style: TileStyle,
    pub name: String,
    // Indicates whether the player or monsters can move through the tile
    #[serde(default="yes")]
    pub passable: bool,
    // Indicates whether lines of sight will continue through the tile
    #[serde(default="yes")]
    pub see_through: bool,
    pub description: String,
    pub on_interact: Option<OnInteract>
}

#[derive(Serialize,Deserialize)]
pub struct Entity {
    pub hp: Option<u32>,
    pub speed: usize,
    pub loc: Option<ObjTraverser>,
    pub contains: Option<Vec<Object>>,
}

/// Things that we only want to have one of, for the player specifically.
/// Data involving things displayed to the screen
/// may also be contained in UIHandler.
#[derive(Serialize,Deserialize)]
pub struct PlayerData {
    pub created: bool,
    pub entity: EntityID,
}

// Entities and UpdateLocs MUST remain consistent with each other
#[derive(Serialize,Deserialize)]
pub struct EntityHandler {
    entities: RemoveVec<Entity>,
    templates: Vec<Template>,
    pub player_data: PlayerData
}

impl EntityHandler {
    pub fn new() -> Self {
        Self {
            entities: RemoveVec::new(),
            templates: Vec::new(),
            player_data: PlayerData {
                created: false,
                entity: 0,
            }
        }
    }
    
    pub fn add_template(&mut self, template: Template) -> TemplateID {
        self.templates.push(template);
        return self.templates.len()-1;
    }
    
    pub fn template(&self, id: TemplateID) -> &Template {
        return &self.templates[id];
    }
    
    pub fn make_entity(&mut self, entity: Entity) -> EntityID {
        let id = self.entities.push(entity);
        return id;
    }

    pub fn pack(&mut self, id: EntityID, updates: &mut UpdateHandler) -> Option<(usize, Entity, Vec<Update>)> {
        if !self.entities.has(id) {
            return None;
        }
        let entity = self.entities.remove(id).unwrap();
        let ent_updates = updates.remove(id);
        return Some((id, entity, ent_updates));
    }
    
    pub fn next_id(&self) -> usize {
        return self.entities.next();
    }
}

impl std::ops::Index<EntityID> for EntityHandler {
    type Output = Entity;

    fn index(&self, index: EntityID) -> &Self::Output {
        return &self.entities[index];
    }
}

impl std::ops::IndexMut<EntityID> for EntityHandler {
    fn index_mut(&mut self, index: EntityID) -> &mut Self::Output {
        return &mut self.entities[index];
    }
}
