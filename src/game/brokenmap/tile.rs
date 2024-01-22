use super::gate::{Gate, MatchedGate, LooseGate};
use super::super::identifiers::UniqTile;

pub struct Tile {
    gates: [Gate; 4],
}

impl Tile {
    pub fn new(map_id: u32) -> Tile {
        Tile {
            gates: [
                Gate::None,
                Gate::None,
                Gate::None,
                Gate::None,
            ],
        }
    }
    
    // Takes: self, gate to go through
    // Returns: (tile as a result of going through, )
    // Side effects: Loads maps if tiles are not present
    pub fn through(&self, gate: u8) -> (UniqTile, u8, u8) {
        let gate = &self.gates[gate as usize];
        let tile = gate.load();
        if tile.is_some() {
            return (tile.unwrap(), gate.return_gate(), gate.flip());
        }
        return (UniqTile {map: 0, tile: 0}, 0, 0);
    }
    
    pub fn has_gate(&self, gate: u8) -> bool {
        (&self.gates[gate as usize]).connected()
    }
    
    pub fn set_gate(&mut self, gate_from: u8, tile: UniqTile, gate_to: u8, flip: u8) {
        self.gates[gate_from as usize] = Gate::Matched(MatchedGate {
            tile: tile, gate: gate_to, flip: flip,
        });
    }
}
