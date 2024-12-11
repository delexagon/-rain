use std::ops::{Index, IndexMut};
use serde::{Deserialize,Serialize};

const BITS: usize = (usize::BITS as usize)-8;

fn compose(a: u8, b: usize) -> usize {
    return ((a as usize)<<BITS) as usize+b;
}

fn chop(x: usize) -> (u8, usize) {
    return ((x>>BITS) as u8, x&(((1 as usize)<<BITS)-1));
}

// Removals from this vector are O(1),
// Additions are possibly O(n) but only if you've removed beforehand
#[derive(Serialize,Deserialize)]
pub struct RemoveVec<T> {
    dat: Vec<Option<T>>,
    tracker: Vec<u8>,
    first_removed: usize,
    num_removed: usize,
}

impl<T> RemoveVec<T> {
    pub fn new() -> Self { return Self {dat: Vec::new(), tracker: Vec::new(), first_removed: 0, num_removed: 0} }
    pub fn push(&mut self, value: T) -> usize {
        if self.first_removed == self.dat.len() {
            self.dat.push(Some(value));
            self.tracker.push(0);
            self.first_removed += 1;
            // tracker is 0; no composition needed
            return self.dat.len()-1;
        }
        self.dat[self.first_removed] = Some(value);
        self.tracker[self.first_removed] += 1;
        let a = self.tracker[self.first_removed];
        let x = self.first_removed;
        for i in self.first_removed..self.dat.len() {
            if self.dat[self.first_removed].is_none() {
                self.num_removed -= 1;
                self.first_removed = i;
                return compose(a, x);
            }
        }
        self.first_removed = self.dat.len();
        self.num_removed -= 1;
        return compose(a, x);
    }
    // This can break the structure if used incorrectly. Only use it if you're coupling vectors together,
    // meaning that you remove/push only at the same time.
    // Then, you can get the next value from one vector and say that it's used here; so an O(n) search does not
    // have to be done.
    pub fn assert_push(&mut self, value: T, next: usize) -> usize {
        let (_, next_index) = chop(next);
        if self.first_removed == self.dat.len() {
            self.dat.push(Some(value));
            self.tracker.push(0);
            self.first_removed += 1;
            // tracker is 0; no composition needed
            return self.dat.len()-1;
        }
        self.num_removed -= 1;
        self.dat[self.first_removed] = Some(value);
        self.tracker[self.first_removed] += 1;
        let a = self.tracker[self.first_removed];
        let x = self.first_removed;
        if self.dat[next_index].is_none() {
            self.first_removed = next_index;
            return compose(a, x);
        } else {
            // If this is currently filled, we do not do anything.
            return 0;
        }
    }
    pub fn next(&self) -> usize {
        if self.first_removed >= self.dat.len() {
            return self.dat.len();
        }
        return compose(self.tracker[self.first_removed]+1, self.first_removed);
    }
    pub fn chop(index: usize) -> (u8,usize) {
        return chop(index);
    }
    pub fn interior(&mut self) -> &mut Vec<Option<T>> {
        return &mut self.dat;
    }
    pub fn first(&self) -> Option<(usize, &T)> {
        let mut i = 0;
        loop {
            if i >= self.dat.len() {
                return None;
            }
            if self.dat[i].is_some() {
                let a = compose(self.tracker[i], i);
                i += 1;
                return Some((a, self.dat[i].as_ref().unwrap()));
            }
            
            i += 1;
        }
    }
    // Because you can't modify the array when going through iter()
    pub fn nonref_iter(&self, x: usize) -> Option<(usize, &T)> {
        let (_, mut i) = chop(x);
        i += 1;
        loop {
            if i >= self.dat.len() {
                return None;
            }
            if self.dat[i].is_some() {
                let a = compose(self.tracker[i], i);
                return Some((a, self.dat[i].as_ref().unwrap()));
            }
            
            i += 1;
        }
    }
    pub fn iter(&self) -> RemoveIter<T> {
        RemoveIter {iter: self.dat.iter()}
    }
    pub fn iter_mut(&mut self) -> RemoveIterMut<T> {
        RemoveIterMut {iter: self.dat.iter_mut()}
    }
    pub fn has(&self, index: usize) -> bool {
        let (a,x) = chop(index);
        return self.tracker[x] == a && self.dat[x].is_some();
    }
    pub fn remove(&mut self, index: usize) -> Option<T> {
        let (a,x) = chop(index);
        if self.tracker[x] == a {
            let r = self.dat[x].take();
            if x < self.first_removed {
                self.first_removed = x;
            }
            self.num_removed += 1;
            return r;
        }
        return None;
    }
    pub fn len(&self) -> usize { self.dat.len()-self.num_removed }
}

impl<T> Index<usize> for RemoveVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let (a,x) = chop(index);
        if self.tracker[x] != a {
            panic!("index out of bounds: the object has been replaced");
        }
        match &self.dat[x] {
            Some(y) => return &y,
            None => {
                panic!("index out of bounds: the object has been removed");
            }
        };
    }
}

impl<T> IndexMut<usize> for RemoveVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let (a,x) = chop(index);
        if self.tracker[x] != a {
            panic!("index out of bounds: the object has been replaced");
        }
        match &mut self.dat[x] {
            Some(ref mut y) => return y,
            None => {
                panic!("index out of bounds: the object has been removed");
            }
        };
    }
}

pub struct RemoveIter<'a, T> {
    iter: std::slice::Iter<'a, Option<T>>
}

impl<'a, T: 'a> Iterator for RemoveIter<'a, T> {
    // Previous coord, next coord, direction
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(x) => {
                    if x.is_some() {
                        return Some(x.as_ref().unwrap());
                    }
                    continue;
                }
            }
        }
    }
}

pub struct RemoveIterMut<'a, T> {
    iter: std::slice::IterMut<'a, Option<T>>
}

impl<'a, T: 'a> Iterator for RemoveIterMut<'a, T> {
    // Previous coord, next coord, direction
    type Item = &'a mut T;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(x) => {
                    if x.is_some() {
                        return Some(x.as_mut().unwrap());
                    }
                    continue;
                }
            }
        }
    }
}
