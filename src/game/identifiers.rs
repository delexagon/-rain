use serde::{Serialize,Deserialize};

pub type TileID = u32;
pub type MapID = usize;
pub type BridgeID = usize;

#[derive(Clone, Copy, Hash, Ord, Eq, PartialEq, PartialOrd, Debug, Serialize,Deserialize)]
pub struct UniqTile {
    pub map: MapID,
    pub tile: TileID,
}

#[derive(Clone, Copy, Hash, Ord, Eq, PartialEq, PartialOrd, Debug, Serialize,Deserialize)]
pub struct MapGate {
    pub tile: TileID,
    pub gate: u8,
}

#[derive(Clone, Copy, Hash, Ord, Eq, PartialEq, PartialOrd, Debug)]
pub struct UniqGate {
    pub map: MapID,
    pub tile: TileID,
    pub gate: u8,
}

// Tile, gate, flip, whether this tile can be passed into
pub type ToTile = (UniqTile, u8, u8);
pub enum ThroughResult {
    Exists(ToTile),
    // The name of the map that needs to be loaded from file
    Load(MapID),
    // The location of the bridge that needs to load in generator.bridges for this map
    Generate(BridgeID),
    None
}

use crate::game::{EntityID, TemplateID};

use super::Traverser;

#[derive(Clone, Copy, PartialEq, Eq, Serialize,Deserialize)]
pub struct Object {
    // Object ids are unique for their Entity id
    pub entity_id: Option<EntityID>,
    pub template_id: TemplateID,
}
pub type ObjTile = (Object, UniqTile);
pub type ObjTraverser = (Object, Traverser);