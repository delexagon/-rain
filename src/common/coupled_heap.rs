use std::collections::HashMap;
use std::cmp::Ordering;
use std::hash::Hash;
use serde::{Serialize,Deserialize};

fn up<T: Ord>(vec: &mut Vec<T>, i: usize) -> usize {
    let mut leaf = i;
    while leaf != 0 {
        let root = (leaf-1)/2;
        if vec[leaf] < vec[root] {
            vec.swap(root, leaf);
        } else {
            return leaf;
        }
        leaf = root;
    }
    return 0;
}

// Pushes an object down the heap (I think this is known as heapify)
fn down<T: Ord>(vec: &mut Vec<T>, i: usize) -> usize {
    let len = vec.len();
    if len <= 1 {
        return 0;
    }
    let mut root = i;
    let single_node = len % 2 == 0;
    while (len > 2 && single_node && root <= len/2-2) || (!single_node && root <= len/2-1) {
        let left = root*2+1;
        let right = left+1;
        // The least element must be in the root position
        if vec[left] < vec[right] {
            // If root is less than both sides, we stop moving it down
            if vec[root] < vec[left] {
                return root;
            } else {
                vec.swap(left, root);
                root = left;
            }
        } else {
            if vec[root] < vec[right] {
                return root;
            } else {
                vec.swap(right, root);
                root = right;
            }
        }
    }
    // This case can have either one or two leaves
    if single_node && root == len/2-1 {
        if vec[len-1] < vec[root] {
            vec.swap(len-1, root);
        }
    }
    return root;
}

// Shifts a value into place (when it's arbitrarily changed)
fn shift<T: Ord>(vec: &mut Vec<T>, i: usize) -> usize {
    let a = up(vec, i);
    if a != i {
        return a;
    }
    return down(vec, i);
}

fn insert<T: Ord>(vec: &mut Vec<T>, x: T) -> usize {
    vec.push(x);
    return shift(vec,vec.len()-1);
}

fn remove<T: Ord>(vec: &mut Vec<T>, i: usize) -> T {
    let x = vec.swap_remove(i);
    shift(vec,i);
    return x;
}

#[derive(Debug,Serialize,Deserialize)]
struct Coupled<T: Ord, Ext> {
    pub v: Vec<T>,
    pub e: Ext
}

// Two requirements: Gets updates in sequential order
// Can get all the updates for a particular entity
// Both of these operations should be fast

impl<T: Ord, Ext> PartialEq for Coupled<T,Ext> {
    fn eq(&self, other: &Self) -> bool {
        self.v[0] == other.v[0]
    }
} impl<T: Ord, Ext> Eq for Coupled<T,Ext> {}

impl<T: Ord, Ext> PartialOrd for Coupled<T,Ext> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.v.len() == 0 || other.v.len() == 0 {
            return Some(other.v.len().cmp(&self.v.len()));
        }
        Some(self.v[0].cmp(&other.v[0]))
    }
}
impl<T: Ord, Ext> Ord for Coupled<T,Ext> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.v.len() == 0 || other.v.len() == 0 {
            return other.v.len().cmp(&self.v.len());
        }
        self.v[0].cmp(&other.v[0])
    }
}

// idk
#[derive(Debug,Deserialize,Serialize)]
pub struct CoupledHeap<Ext: Eq+Hash+Copy, T: Ord> {
    interior: Vec<Coupled<T, Ext>>,
    into: HashMap<Ext, usize>,
    length: usize
}

impl<Ext: Eq+Hash+Copy, T: Ord> CoupledHeap<Ext, T> {
    pub fn new() -> Self {
        Self {
            interior: Vec::new(),
            into: HashMap::new(),
            length: 0,
        }
    }
    
    pub fn pop(&mut self) -> Option<(Ext, T)> {
        if self.interior.len() == 0 || self.interior[0].v.len() == 0 {
            return None;
        }
        let x = remove(&mut self.interior[0].v, 0);
        let e =  self.interior[0].e;
        self.shift(0);
        self.length -= 1;
        return Some((e,x));
    }
    
    pub fn push(&mut self, e: Ext, x: T) {
        if !self.into.contains_key(&e) {
            let coupled = Coupled {
                v: vec!(x),
                e: e
            };
            self.interior.push(coupled);
            
            self.into.insert(e, self.interior.len()-1);
            self.shift(&self.interior.len()-1);
        } else {
            let i = *self.into.get(&e).unwrap();
            // In this case, the new event does not drift to the top and
            // we do not have to shift self.interior.
            if self.interior[i].v.len() > 0 && self.interior[i].v[0] < x {
                insert(&mut self.interior[i].v, x);
            } else {
                insert(&mut self.interior[i].v, x);
                self.shift(i);
            }
        }
        self.length += 1;
    }

    pub fn insert(&mut self, e: Ext, heap: Vec<T>) -> usize {
        let coupled = Coupled {
            v: heap,
            e: e
        };
        self.interior.push(coupled);
        self.into.insert(e, self.interior.len()-1);
        return self.shift(&self.interior.len()-1);
    }
    
    pub fn remove(&mut self, e: Ext) -> Vec<T> {
        let i = *self.into.get(&e).unwrap();
        let x = self.interior.swap_remove(i).v;
        self.length -= x.len();
        self.into.remove(&e);
        self.shift(i);
        return x;
    }
    
    // You must run fix on the same e after running modify.
    // For the moment, I do not know a way to fix this.
    pub fn modify(&mut self, e: Ext) -> &mut Vec<T> {
        let i = *self.into.get(&e).unwrap();
        return &mut self.interior[i].v;
    }
    
    // I don't know how to make it so that you don't need to run this.
    // Ideally, the heap would be fixed as soon as the reference is dropped
    pub fn fix(&mut self, e: Ext) {
        let i = *self.into.get(&e).unwrap();
        self.shift(i);
    }
    
    // These are the same as ^, but the HashMap values
    // have to be updated as the items are swapped
    fn up(&mut self, i: usize) -> usize {
        // Cases in which we do nothing
        if i == 0 || self.interior[i] >= self.interior[(i-1)/2] {
            return i;
        }
        let mut leaf = i;
        while leaf != 0 {
            let root = (leaf-1)/2;
            if self.interior[leaf] < self.interior[root] {
                // Replacing the location must occur AFTER the swap
                self.interior.swap(root, leaf);
                self.into.insert(self.interior[leaf].e, leaf);
            } else {
                self.into.insert(self.interior[leaf].e, leaf);
                return leaf;
            }
            leaf = root;
        }
        self.into.insert(self.interior[0].e, 0);
        return 0;
    }
    
    // Pushes an object down the heap (I think this is known as heapify)
    fn down(&mut self, i: usize) -> usize {
        let len = self.interior.len();
        // Cases in which we do nothing
        if (i*2+1 >= len || self.interior[i*2+1] > self.interior[i])
        && (i*2+2 >= len || self.interior[i*2+2] > self.interior[i]) {
            return i;
        }
        
        let single_node = len % 2 == 0;
        
        let mut root = i;
        while (len > 2 && single_node && root <= len/2-2) || (!single_node && root <= len/2-1) {
            let left = root*2+1;
            let right = left+1;
            // The least element must be in the root position
            if self.interior[left] < self.interior[right] {
                // If root is less than both sides, we stop moving it down
                if self.interior[root] < self.interior[left] {
                    self.into.insert(self.interior[root].e, root);
                    return root;
                } else {
                    self.interior.swap(left, root);
                    self.into.insert(self.interior[root].e, root);
                    root = left;
                }
            } else {
                if self.interior[root] < self.interior[right] {
                    self.into.insert(self.interior[root].e, root);
                    return root;
                } else {
                    self.interior.swap(right, root);
                    self.into.insert(self.interior[root].e, root);
                    root = right;
                }
            }
        }
        // This case can have either one or two leaves
        if single_node && root == len/2-1 {
            if self.interior[len-1] < self.interior[root] {
                self.interior.swap(len-1, root);
                self.into.insert(self.interior[root].e, root);
                root = len-1;
            }
        }
        self.into.insert(self.interior[root].e, root);
        return root;
    }
    
    // Shifts a value into place (when it's arbitrarily changed)
    fn shift(&mut self, i: usize) -> usize {
        let a = self.up(i);
        if a != i {
            return a;
        }
        return self.down(i);
    }
    
    pub fn len(&self) -> usize {self.length}
}
