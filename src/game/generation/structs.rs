use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::game::entity_handler::*;
use crate::common::Array2D;
use super::{Bridge, Generator};
use std::collections::{HashSet, HashMap};
pub use super::color_generation::*;

#[derive(Serialize, Deserialize)]
pub struct NameCh {
    pub ch: char,
    pub name: String
}

#[derive(Deserialize)]
pub struct NameLength {
    pub length: usize,
    pub name: String
}

#[derive(Serialize, Deserialize)]
pub struct ObjectGen {
    pub entity: Option<EntityEnum>,
    pub entity_info: Option<Value>,
    pub object: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct Floor {
    pub ch: char,
    pub tile: DynTile
}

fn from_strings<'de, D>(deserializer: D) -> Result<Array2D<char>, D::Error>
where D: serde::Deserializer<'de> {
    return Ok(Array2D::from_strs(Vec::<String>::deserialize(deserializer)?));
}
fn from_stringses<'de, D>(deserializer: D) -> Result<Vec<Array2D<char>>, D::Error>
where D: serde::Deserializer<'de> {
    return Ok(Vec::<Vec<String>>::deserialize(deserializer)?.drain(..).map(|v| Array2D::from_strs(v)).collect());
}

#[derive(Deserialize)]
pub struct GenerationData {
    pub mapgen: Option<MapGen>,
    // {submap_name: [(exit_name_in_map, rename)]}
    // Not specified here is left unconnected
    pub contains: HashMap<String, Vec<String>>,
    // internal only
    pub connect: Vec<(String, String, bool)>,
    pub templates: Option<Vec<Template>>,
}

#[derive(Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum MapGen {
    Euclid(EuclidMapGen),
    Sparse(SparseMapGen),
}

#[derive(Deserialize)]
pub struct EuclidMapGen {
    pub bridges: Vec<NameCh>,
    #[serde(default)]
    pub flipped: HashSet<char>,
    #[serde(deserialize_with = "from_strings")]
    pub space: Array2D<char>,
    #[serde(deserialize_with = "from_strings")]
    pub object_maps: Array2D<char>,
    pub object_key: HashMap<char, ObjectGen>,
    pub default_wall: DynTile,
    pub floors: Vec<Floor>,
}

#[derive(Deserialize)]
pub struct SparseMapGen {
    pub width: Option<usize>,
    pub liminal_bridges: Vec<NameLength>,
    pub floor: DynTile,
}

#[derive(Serialize,Deserialize,Debug)]
pub enum BridgeLocation {Child(usize), Here(usize)}
#[derive(Serialize,Deserialize,Debug)]
pub enum NameOrID {Ungenerated(String), Generated(usize)}
#[derive(Serialize,Deserialize,Default,Debug)]
pub struct UsedByGeneration {
    pub parent: Option<usize>,
    pub bridges: Vec<(String, Bridge)>,
    pub bridge_to: HashMap<String, BridgeLocation>,
    pub bridge_connect: HashMap<String, (String, bool)>,
    pub children: Vec<NameOrID>
}
