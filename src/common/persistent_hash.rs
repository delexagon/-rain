use std::num::Wrapping;
use rand::{thread_rng, Rng};
use serde::{Serialize, Deserialize};

// Pseudorandom number generator
// Seems to be adequate
#[derive(Serialize, Deserialize,Copy,Clone)]
pub struct PersistentHash {
    a: Wrapping<usize>,
    b: Wrapping<usize>,
    c: Wrapping<usize>,
}

impl Default for PersistentHash {
    fn default() -> Self {Self::new()}
}

impl PersistentHash {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        Self {a: Wrapping(rng.gen()), b: Wrapping(rng.gen()), c: Wrapping(rng.gen())}
    }
    
    fn hashw(&self, mut v: Wrapping<usize>) -> Wrapping<usize> {
        v = (((v >> 16) ^ v) * self.a) ^ self.c;
        v = ((v >> 16) ^ v) * self.b;
        (v >> 16) ^ v
    }
    
    pub fn hash(&self, v: usize) -> usize {
        self.hashw(Wrapping(v)).0
    }
    
    pub fn hash2(&self, x: usize, y: usize) -> usize {
        let a = self.hashw(Wrapping(x))*Wrapping(y);
        self.hashw(a).0
    }
}
