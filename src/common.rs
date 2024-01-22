mod communication;
mod color;
mod options;
mod databox;
mod entity_component;

// We want both parts to be able to access any of these structures, by design.
pub use communication::{Communicator, GameMessage, UIMessage, KeyAction};
pub use color::{Rgb, Style, UITile, TileStyle, NORMALSTYLE, REDSTYLE, BLANKTILE, FILLEDTILE};
pub use entity_component::EntityComponentSystem;
pub use databox::DataBox;
