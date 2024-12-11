
pub struct CompressedBool {
    bools: u8
}

impl CompressedBool {
    pub fn get(&self, i: u8) -> u8 {
        return (self.bools>>i)&1;
    }
    pub fn set(&mut self, i: u8, x: u8) {
        self.bools = self.bools&(!(1<<i))|(x<<i);
    }
    pub fn getb(&self, i: u8) -> bool {
        return self.get(i) != 0;
    }
    pub fn setb(&mut self, i: u8, x: bool) {
        self.bools = self.bools&(!(1<<i))|(u8::from(x)<<i);
    }
}
