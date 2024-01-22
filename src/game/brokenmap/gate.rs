use super::super::identifiers::UniqTile;

pub enum Gate {
    Matched(MatchedGate),
    Loose(LooseGate),
    None,
}

impl Gate {
    pub fn currently_loaded(&self) -> bool {
        return matches!(&self, Gate::Matched(_));
    }
    
    // Does not load tile ; do not use in outside code
    pub fn get_connected(&self) -> Option<UniqTile> {
        return match self {
            Gate::Matched(gate) => Some(gate.tile),
            _ => None,
        }
    }
    
    pub fn return_gate(&self) -> u8 {
        return match self {
            Gate::Matched(gate) => gate.gate,
            _ => 255,
        }
    }
    
    pub fn flip(&self) -> u8 {
        return match self {
            Gate::Matched(gate) => gate.flip,
            _ => 255,
        }
    }
    
    pub fn load(&self) -> Option<UniqTile> {
        let a = self.get_connected();
        if a.is_some() {
            return a;
        }
        // Unimplemented
        return None;
    }
    
    pub fn connected(&self) -> bool {
        return match self {
            Gate::None => false,
            _ => true,
        }
    }
}

// A gate which links to a currently loaded tile.
// This is the reason gates must be two way; when the other side
// is unloaded, this must be converted to a FitGate.
pub struct MatchedGate {
    pub tile: UniqTile,
    pub gate: u8,
    // Flip is a u8 because I hate converting it from bool and back
    // constantly for bitwise arithmetic
    pub flip: u8,
}

// A gate which links to an ungenerated map. The tile that it links to is not yet decided.
pub struct LooseGate {
    // There should be some struct somewhere that stores these so new maps can find something
    // that looks good to make connections with
    pub uncreated_map_id: usize,
    pub group_id: usize,
}
