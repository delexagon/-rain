use crate::common::{Coord, Array2D};
use std::collections::HashSet;
// Functions to iterate over data contained by an Array2D

pub struct LinearIter {
    start_x: usize,
    start_y: usize,
    height: usize,
    width: usize,
    x: usize,
    y: usize,
    variant: u8,
}

pub fn linear_iterate(start: Coord, width: usize, height: usize) -> LinearIter {
    LinearIter {
        start_x: start.0,
        start_y: start.1,
        height: height,
        width: width,
        x: start.0,
        y: start.1,
        variant: 0,
    }
}

impl Iterator for LinearIter {
    // Previous coord, next coord, direction
    type Item = (Coord, Coord, u8);
    
    fn next(&mut self) -> Option<Self::Item> {
        let mut prev = (0, 0);
        if self.variant == 0 {
            if self.y > 0 {
                prev = (self.x, self.y);
                self.y -= 1;
            } else {
                self.variant = 1;
                self.x = self.start_x;
                self.y = self.start_y;
            }
        }
        if self.variant == 1 {
            if self.y < self.height-1 {
                prev = (self.x, self.y);
                self.y += 1;
            } else {
                self.variant = 2;
                self.x = self.start_x;
                self.y = self.start_y;
            }
        }
        if self.variant == 2 {
            if self.x > 0 {
                prev = (self.x, self.y);
                self.x -= 1;
            } else {
                self.variant = 3;
                self.x = self.start_x;
                self.y = self.start_y;
            }
        }
        if self.variant == 3 {
            if self.x < self.width-1 {
                prev = (self.x, self.y);
                self.x += 1;
            } else {
                self.variant = 4;
                self.x = self.start_x;
                self.y = self.start_y;
            }
        }
        if self.variant > 3 {
            return None;
        }

        Some(((self.x, self.y), prev, self.variant))
    }
}

pub fn quadrant_iterate(start: Coord, width: usize, height: usize) -> QuadIter {
    let var0 = start.1 > 0 && start.0 < width-1;
    let var1 = start.1 > 0 && start.0 > 0;
    let var2 = start.1 < height-1 && start.0 > 0;
    let var3 = start.1 < height-1 && start.0 < width-1;
    if var0 {
        QuadIter {
            start_x: start.0,
            start_y: start.1,
            height: height,
            width: width,
            x: start.0+1,
            y: start.1-1,
            existing_variants: [var0, var1, var2, var3],
            variant: 0,
        }
    } else if var1 {
        QuadIter {
            start_x: start.0,
            start_y: start.1,
            height: height,
            width: width,
            x: start.0-1,
            y: start.1-1,
            existing_variants: [var0, var1, var2, var3],
            variant: 1,
        }
    } else if var2 {
        QuadIter {
            start_x: start.0,
            start_y: start.1,
            height: height,
            width: width,
            x: start.0-1,
            y: start.1+1,
            existing_variants: [var0, var1, var2, var3],
            variant: 2,
        }
    } else if var3 {
        QuadIter {
            start_x: start.0,
            start_y: start.1,
            height: height,
            width: width,
            x: start.0+1,
            y: start.1+1,
            existing_variants: [var0, var1, var2, var3],
            variant: 3,
        }
    } else {
        QuadIter {
            start_x: start.0,
            start_y: start.1,
            height: height,
            width: width,
            x: start.0,
            y: start.1,
            existing_variants: [var0, var1, var2, var3],
            variant: 4,
        }
    }
}

pub struct QuadIter {
    start_x: usize,
    start_y: usize,
    height: usize,
    width: usize,
    x: usize,
    y: usize,
    existing_variants: [bool;4],
    variant: u8,
}
impl QuadIter {
    fn next_variant(&mut self) {
        if self.variant < 1 && self.existing_variants[1] {
            self.variant = 1;
            self.x = self.start_x-1;
            self.y = self.start_y-1;
        } else if self.variant < 2 && self.existing_variants[2] {
            self.variant = 2;
            self.x = self.start_x-1;
            self.y = self.start_y+1;
        } else if self.variant < 3 && self.existing_variants[3] {
            self.variant = 3;
            self.x = self.start_x+1;
            self.y = self.start_y+1;
        } else {
            self.variant = 4;
        }
    }
}

impl Iterator for QuadIter {
    // This coord, offset from start, prev along, along dir, prev sidle, sidle dir
    type Item = (Coord, Coord, Coord, u8, Coord, u8);
    
    fn next(&mut self) -> Option<Self::Item> {
        let here = (self.x, self.y);
        
        let offset: Coord;
        let prev_along: Coord;
        let prev_sidle: Coord;
        let along_dir: u8;
        let sidle_dir: u8;
        // Need to move to the next viable position
        if self.variant == 0 {
            let x_diff = self.x-self.start_x;
            let y_diff = self.start_y-self.y;
            if x_diff > y_diff {
                // Along = x, Sidle = y
                offset = (x_diff, y_diff);
                prev_along = (self.x-1, self.y);
                along_dir = 3;
                prev_sidle = (self.x, self.y+1);
                sidle_dir = 0;
            } else {
                offset = (y_diff, x_diff);
                prev_sidle = (self.x-1, self.y);
                sidle_dir = 3;
                prev_along = (self.x, self.y+1);
                along_dir = 0;
            }
            if self.x < self.width-1 {
                self.x += 1;
            } else {
                if self.y > 0 {
                    self.y -= 1;
                    self.x = self.start_x+1;
                } else {
                    self.next_variant();
                }
            }
        } else if self.variant == 1 {
            let x_diff = self.start_x-self.x;
            let y_diff = self.start_y-self.y;
            if x_diff > y_diff {
                offset = (x_diff, y_diff);
                // Along = x, Sidle = y
                prev_along = (self.x+1, self.y);
                along_dir = 2;
                prev_sidle = (self.x, self.y+1);
                sidle_dir = 0;
            } else {
                offset = (y_diff, x_diff);
                prev_sidle = (self.x+1, self.y);
                sidle_dir = 2;
                prev_along = (self.x, self.y+1);
                along_dir = 0;
            }
            if self.x > 0 {
                self.x -= 1;
            } else {
                if self.y > 0 {
                    self.y -= 1;
                    self.x = self.start_x-1;
                } else {
                    self.next_variant();
                }
            }
        } else if self.variant == 2 {
            let x_diff = self.start_x-self.x;
            let y_diff = self.y-self.start_y;
            if x_diff > y_diff {
                offset = (x_diff, y_diff);
                // Along = x, Sidle = y
                prev_along = (self.x+1, self.y);
                along_dir = 2;
                prev_sidle = (self.x, self.y-1);
                sidle_dir = 1;
            } else {
                offset = (y_diff, x_diff);
                prev_sidle = (self.x+1, self.y);
                sidle_dir = 2;
                prev_along = (self.x, self.y-1);
                along_dir = 1;
            }
            if self.x > 0 {
                self.x -= 1;
            } else {
                if self.y < self.height-1 {
                    self.y += 1;
                    self.x = self.start_x-1;
                } else {
                    self.next_variant();
                }
            }
        } else if self.variant == 3 {
            let x_diff = self.x-self.start_x;
            let y_diff = self.y-self.start_y;
            if x_diff > y_diff {
                // Along = x, Sidle = y
                offset = (x_diff, y_diff);
                prev_along = (self.x-1, self.y);
                along_dir = 3;
                prev_sidle = (self.x, self.y-1);
                sidle_dir = 1;
            } else {
                offset = (y_diff, x_diff);
                prev_sidle = (self.x-1, self.y);
                sidle_dir = 3;
                prev_along = (self.x, self.y-1);
                along_dir = 1;
            }
            if self.x < self.width-1 {
                self.x += 1;
            } else {
                if self.y < self.height-1 {
                    self.y += 1;
                    self.x = self.start_x+1;
                } else {
                    self.next_variant();
                }
            }
        } else {
            return None;
        }

        Some((here, offset, prev_along, along_dir, prev_sidle, sidle_dir))
    }
}

pub struct CoordIter {
    height: usize,
    width: usize,
    x: usize,
    y: usize,
}

pub fn coord_iterate(width: usize, height: usize) -> CoordIter {
    CoordIter {
        height: height,
        width: width,
        x: 0,
        y: 0,
    }
}

impl Iterator for CoordIter {
    // Previous coord, next coord, direction
    type Item = Coord;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.width == 0 || self.height == 0 || self.y >= self.height {
            return None;
        }
        
        let a = (self.x, self.y);
        self.x += 1;
        if self.x >= self.width {
            self.x = 0;
            self.y += 1;
        }
        return Some(a);
    }
}

pub struct ItemIter<'a, T: Copy> {
    i: usize,
    item: T,
    arr: &'a Array2D<T>
}

pub fn item_iterate<T: Eq+Copy>(arr: &Array2D<T>, item: T) -> ItemIter<T> {
    return ItemIter {i: 0, arr: arr, item: item};
}

impl<T: Eq+Copy> Iterator for ItemIter<'_, T> {
    // Previous coord, next coord, direction
    type Item = Coord;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.i >= self.arr.len() {
                return None;
            }
            
            let coord = self.arr.to_coord(self.i);
            if self.arr[coord] == self.item {
                self.i += 1;
                return Some(coord);
            }
            
            self.i += 1;
        }
    }
}

pub struct ToCoord {
    end: Coord,
    current: Coord,
    done: bool
}

pub fn between_coords(left: Coord, right: Coord) -> ToCoord {
    return ToCoord {current: left, end: right, done: false};
}

impl Iterator for ToCoord {
    type Item = Coord;
    
    fn next(&mut self) -> Option<Self::Item> {
        let x;
        if self.current.0 > self.end.0 {
            x = Some(self.current);
            self.current.0 -= 1;
        } else if self.current.0 < self.end.0 {
            x = Some(self.current);
            self.current.0 += 1;
        } else if self.current.1 > self.end.1 {
            x = Some(self.current);
            self.current.1 -= 1;
        } else if self.current.1 < self.end.1 {
            x = Some(self.current);
            self.current.1 += 1;
        } else {
            if self.done {
                x = None
            } else {
                x = Some(self.current);
                self.done = true;
            }
        }
        return x;
    }
}

#[derive(Debug)]
pub struct SideToCoord {
    start: Coord,
    end: Coord,
    current: Coord,
    side: u8
} impl SideToCoord {
    pub fn new(start: Coord, end: Coord, side: u8) -> Option<Self> {
        match side {
            0 | 1 => if start.1 != end.1 {return None;}
            2 | 3 => if start.0 != end.0 {return None;}
            _ => return None
        }
        Some(Self {
            start,
            end,
            current: start,
            side
        })
    }
    pub fn end(mut self) -> Self {
        self.current = self.end;
        match self.side {
            0 | 1 => self.current.0 += 1,
            2 | 3 => self.current.1 += 1,
            _ => ()
        }
        self
    }
}

impl Iterator for SideToCoord {
    type Item = (Coord, u8);
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.current.0 > self.end.0 || self.current.1 > self.end.1 {
            return None;
        }
        match self.side {
            0 => {
                self.current.0 += 1;
                Some(((self.current.0-1, self.current.1+1), self.side))
            },
            1 => {
                self.current.0 += 1;
                Some(((self.current.0-1, self.current.1-1), self.side))
            }
            2 => {
                self.current.1 += 1;
                Some(((self.current.0+1, self.current.1-1), self.side))
            }
            3 => {
                self.current.1 += 1;
                Some(((self.current.0-1, self.current.1-1), self.side))
            },
            _ => None
        }
    }
}

impl DoubleEndedIterator for SideToCoord {
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.side {
            0 if self.current.0 > self.start.0 => {
                self.current.0 -= 1;
                Some(((self.current.0, self.current.1+1), self.side))
            },
            1 if self.current.0 > self.start.0 => {
                self.current.0 -= 1;
                Some(((self.current.0, self.current.1-1), self.side))
            }
            2 if self.current.1 > self.start.1 => {
                self.current.1 -= 1;
                Some(((self.current.0+1, self.current.1), self.side))
            }
            3 if self.current.1 > self.start.1 => {
                self.current.1 -= 1;
                Some(((self.current.0-1, self.current.1), self.side))
            },
            _ => None
        }
    }
}

pub enum StraightBridge {
    // 
    Single(Coord, u8),
    // From left to right
    Length(Coord,Coord, u8)
}

pub struct AdjacentFloorIter<'a, T: Copy+Eq, F1: Fn(&T) -> bool, F2: Fn(&T) -> bool> {
    i: usize,
    floor: F1,
    ignore: F2,
    arr: &'a Array2D<T>
} impl<'a, T: Copy+Eq, F1: Fn(&T) -> bool, F2: Fn(&T) -> bool> AdjacentFloorIter<'a, T, F1,F2> {
    pub fn new(arr: &'a Array2D<T>, floor: F1, ignore: F2) -> Self {
        Self {i: 0, arr, floor, ignore}
    }
}

fn dir_fits_condition<T, F: Fn(&T) -> bool>(arr: &Array2D<T>, condition: &F, (x,y): (usize,usize), dir: u8) -> bool {
    match dir {
        0 if y > 0 => {
            condition(&arr[(x,y-1)])
        },
        1 if y < arr.height()-1 => {
            condition(&arr[(x,y+1)])
        },
        2 if x > 0 => {
            condition(&arr[(x-1,y)])
        },
        3 if x < arr.width()-1 => {
            condition(&arr[(x+1,y)])
        },
        _ => false
    }
}

fn adjacent_condition<T, F: Fn(&T) -> bool>(arr: &Array2D<T>, condition: &F, (x,y): (usize,usize)) -> [bool; 4] {
    [
        y > 0 && condition(&arr[(x,y-1)]),
        y < arr.height()-1 && condition(&arr[(x,y+1)]),
        x > 0 && condition(&arr[(x-1,y)]),
        x < arr.width()-1 && condition(&arr[(x+1,y)])
    ]
}

fn in_dir_unchecked((x,y): (usize,usize), dir: u8) -> (usize,usize) {
    match dir {
        0 => (x,y-1),
        1 => (x,y+1),
        2 => (x-1,y),
        3 => (x+1,y),
        _ => (0,0)
    }
}

impl<T: Eq+Copy, F1: Fn(&T) -> bool, F2: Fn(&T) -> bool> Iterator for AdjacentFloorIter<'_, T, F1,F2> {
    // Associated item, is left right (not up down), floor is on greater side, start coord (left), end coord (right)
    type Item = (T, StraightBridge);
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.i >= self.arr.len() {
                return None;
            }

            let (x,y) = self.arr.to_coord(self.i);
            let here = self.arr[self.i];
            if (self.ignore)(&here) {self.i += 1; continue;}
            let dir_continue = match adjacent_condition(self.arr, &|v| v == &here, (x,y)) {
                [true,_,_,_] => {self.i += 1; continue;}
                [_,_,true,_] => {self.i += 1; continue;}
                [_,true,_,true] => {self.i += 1; continue;}
                [_,false,_,false] => 5,
                [_,true,_,false] => 1,
                [_,false,_,true] => 3,
            };
            if dir_continue == 3 {
                let mut last_x = (x,y);
                let mut side = 2;
                while last_x.0 < self.arr.width() && self.arr[last_x] == here {
                    match adjacent_condition(&self.arr, &self.floor, last_x) {
                        [true,false,_,_] => if side == 2 || side == 1 {side = 1} else {self.i += 1; continue;},
                        [false,true,_,_] => if side == 2 || side == 0 {side = 0} else {self.i += 1; continue;},
                        [true,true,_,_] => {self.i += 1; continue;},
                        _ => (),
                    }
                    last_x = (last_x.0+1, last_x.1);
                }
                if side == 2 {self.i += 1; continue;}
                last_x = (last_x.0-1, last_x.1);
                self.i += 1;
                return Some((here, StraightBridge::Length((x, y), last_x, side)));
            } else if dir_continue == 1 {
                let mut last_y = (x,y);
                let mut side = 0;
                while last_y.1 < self.arr.height() && self.arr[last_y] == here {
                    match adjacent_condition(&self.arr, &self.floor, last_y) {
                        [_,_,true,false] => if side == 0 || side == 3 {side = 3} else {self.i += 1; continue;},
                        [_,_,false,true] => if side == 0 || side == 2 {side = 2} else {self.i += 1; continue;},
                        [_,_,true,true] => {self.i += 1; continue;},
                        _ => (),
                    }
                    last_y = (last_y.0, last_y.1+1);
                }
                if side == 0 {self.i += 1; continue;}
                last_y = (last_y.0, last_y.1-1);
                self.i += 1;
                return Some((here, StraightBridge::Length((x, y), last_y, side)));
            } else {
                self.i += 1;
                return match adjacent_condition(&self.arr, &self.floor, (x,y)) {
                    [true,false,false,false] => Some((here, StraightBridge::Single(in_dir_unchecked((x,y), 0), 1))),
                    [false,true,false,false] => Some((here, StraightBridge::Single(in_dir_unchecked((x,y), 1), 0))),
                    [false,false,true,false] => Some((here, StraightBridge::Single(in_dir_unchecked((x,y), 2), 3))),
                    [false,false,false,true] => Some((here, StraightBridge::Single(in_dir_unchecked((x,y), 3), 2))),
                    _ => continue
                };
            }
        }
    }
}

pub struct ContiguousIter<'a, T: Copy+Eq> {
    i: usize,
    ignore: &'a HashSet<T>,
    arr: &'a Array2D<T>,
    my_copy: Array2D<bool>
} impl<'a, T: Copy+Eq> ContiguousIter<'a, T> {
    pub fn new(arr: &'a Array2D<T>, ignore: &'a HashSet<T>) -> Self {
        Self {
            i: 0, arr, ignore,
            my_copy: Array2D::new_sized(arr.width(), arr.height(), false)
        }
    }
}
