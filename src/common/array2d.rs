use super::{linear_iterate,quadrant_iterate,coord_iterate,CoordIter,QuadIter,LinearIter};
use std::ops::{IndexMut, Index};
use serde::{Deserialize,Serialize};

// x,y
pub type Coord = (usize,usize);
pub fn center(width: usize, height: usize) -> Coord {
    (width/2, height/2)
}

#[derive(Serialize,Deserialize)]
pub struct Array2D<T> {
    vec: Vec<T>,
    width: usize,
    height: usize,
}

impl Array2D<char> {
    pub fn from_str(x: &str) -> Array2D<char> {
        let mut itr = x.split('\n');
        let mut arr = Array2D::new();
        arr.extend(itr.next().unwrap().chars());
        let width = arr.abslen();
        for i in itr {
            arr.extend(i.chars());
        }
        arr.sized(width)
    }
    
    // This is an abomination that Rust has forced me to bring into existence
    pub fn from_strs(x: Vec<String>) -> Array2D<char> {
        let len = x.len();
        if len == 0 {
            return Array2D::new();
        }
        let mut itr = x.iter();
        let mut arr = Array2D::new();
        arr.extend(itr.next().unwrap().chars());
        let width = arr.abslen();
        arr.reserve_exact(width*len);
        for i in itr {
            arr.extend(i.chars());
        }
        arr.sized(width)
    }
}

impl<T,I,J> From<J> for Array2D<T> where T: Copy, I: IntoIterator<Item = T>, J: IntoIterator<Item = I> {
    fn from(x: J) -> Self {
        let mut itr = x.into_iter();
        let mut next = itr.next();
        let mut arr = Array2D::new();
        let width;
        if next.is_some() {
            arr.extend(next.unwrap());
            width = arr.abslen();
            next = itr.next();
        } else {
            return arr;
        }
        while next.is_some() {
            arr.extend(next.unwrap());
            next = itr.next();
        }
        arr.sized(width)
    }
}

impl<T> Array2D<T> {
    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            vec: Vec::new(),
        }
    }

    pub fn get(&self, coord: (usize,usize)) -> Option<&T> {
        if self.within(coord) {
            Some(&self[coord])
        } else {
            None
        }
    }
    
    pub fn with_capacity(size: usize) -> Self {
        Self {
            width: 0,
            height: 0,
            vec: Vec::with_capacity(size),
        }
    }
    
    pub fn dim(&self) -> (usize,usize) {(self.width,self.height)}
    
    pub fn reserve_exact(&mut self, n: usize) {
        self.vec.reserve_exact(n);
    }
    
    pub fn new_sized(width: usize, height: usize, value: T) -> Self where T: Copy {
        let mut vec = Vec::with_capacity(width*height);
        for _ in 0..width*height {
            vec.push(value);
        }
        return Self {
            width: width,
            height: height,
            vec: vec,
        };
    }
    
    fn oob(&self, x: usize, y: usize) -> bool {
        return x >= self.width || y >= self.height;
    }
    pub fn within(&self, coord: (usize,usize)) -> bool {
        return !self.oob(coord.0,coord.1);
    }

    pub fn set_all(&mut self, item: T) where T: Copy {
        self.vec.iter_mut().for_each(|i| *i = item);
    }
    
    pub fn push(&mut self, item: T) {
        self.vec.push(item);
    }
    
    pub fn size(&mut self, width: usize) {
        self.width = width;
        self.height = self.vec.len()/width;
    }
    
    pub fn sized(mut self, width: usize) -> Self {
        self.width = width;
        self.height = self.vec.len()/width;
        return self;
    }

    pub fn resize(&mut self, width: usize, height: usize, value: T) where T: Copy {
        self.width = width;
        self.height = height;
        self.vec.resize(width*height, value);
    }

    /// Resizes, and keeps the previous content of the array at the (0,0) corner.
    pub fn resize_preserve(&mut self, (width, height): (usize,usize), value: T) -> &mut Self where T: Copy {
        if self.width == width {
            if self.height != height {
                self.vec.resize(width*height, value);
                self.height = height;
            }
        } else if self.width > width {
            // Smaller than before.
            // First row does not have to move
            for row in 1..self.height.min(height) {
                // Copy each row slightly backwards, to where it should be now.
                self.vec.copy_within(self.width*row..self.width*row+width, width*row);
            }
            self.vec.resize(width*height, value);
            if height > self.height {
                self.vec[width*self.height..].fill(value);
            }
            self.height = height;
            self.width = width;
        } else {
            // width has increased
            // There's the possibility the vector is not large enough.
            self.vec.resize(width*height, value);
            let mut row = self.height.min(height);
            while row > 1 {
                row -= 1;
                self.vec.copy_within(self.width*row..self.width*row+self.width, width*row);
                self.vec[width*row+self.width..width*row+width].fill(value);
            }
            if self.vec.len() > 0 {
                self.vec[self.width..width].fill(value);
            }
            self.height = height;
            self.width = width;
        }
        self
    }

    pub fn row(&self, row: usize) -> &[T] {
        return &self.vec[self.width*row..self.width*(row+1)];
    }
    pub fn row_mut(&mut self, row: usize) -> &mut [T] {
        return &mut self.vec[self.width*row..self.width*(row+1)];
    }
    
    pub fn coord_iter(&self) -> CoordIter {
        return coord_iterate(self.width, self.height);
    }
    pub fn linear_iter(&self, coord: Coord) -> LinearIter {
        return linear_iterate(coord, self.width, self.height);
    }
    pub fn quad_iter(&self, coord: Coord) -> QuadIter {
        return quadrant_iterate(coord, self.width, self.height);
    }
    
    pub fn to_coord(&self, i: usize) -> Coord {
        (i%self.width(), i/self.width())
    }
    pub fn to_1d(&self, c: Coord) -> usize {
        self.width*c.1+c.0
    }

    pub fn fill(&mut self, value: T) where T: Clone {
        self.vec.fill(value);
    }
    
    pub fn width(&self) -> usize {self.width}
    pub fn height(&self) -> usize {self.height}
    pub fn abslen(&self) -> usize {self.vec.len()}
    pub fn len(&self) -> usize {self.width*self.height}
    pub fn center(&self) -> Coord {center(self.width, self.height)}
    
    pub fn vec(self) -> Vec<T> {self.vec}
}

impl<T> std::fmt::Display for Array2D<T> where T: std::fmt::Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                write!(f, "{} ", self.vec[y*self.width+x])?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

impl<A> Extend<A> for Array2D<A> {
    fn extend<T>(&mut self, iter: T)
       where T: IntoIterator<Item = A> {
       self.vec.extend(iter);
    }
}

impl<A> Index<Coord> for Array2D<A> {
    type Output = A;

    // Required method
    fn index(&self, index: Coord) -> &Self::Output {
        if self.oob(index.0, index.1) {
            panic!("Index ({}, {}) out of bounds for array with dimensions ({}, {}) and total tile number {}", index.0, index.1, self.width, self.height, self.len());
        }
        return &self.vec[self.width*index.1+index.0];
    }
}

impl<A> IndexMut<Coord> for Array2D<A> {
    // Required method
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output {
        if self.oob(index.0, index.1) {
            panic!("Index out of bounds");
        }
        return &mut self.vec[self.width*index.1+index.0];
    }
}

impl<A> Index<usize> for Array2D<A> {
    type Output = A;

    // Required method
    fn index(&self, index: usize) -> &Self::Output {
        return &self.vec[index];
    }
}
