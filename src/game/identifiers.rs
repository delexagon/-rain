
#[derive(Clone, Copy, Ord, Eq, PartialEq, PartialOrd)]
pub struct UniqTile {
    pub map: u32,
    pub tile: u32,
}

#[derive(Clone, Copy, Ord, Eq, PartialEq, PartialOrd)]
pub struct UniqObj {
    pub entity: u32,
    pub obj: u32,
}
