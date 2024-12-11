use serde::{Serialize,Deserialize};

// This form of box is takeable.
// This means that it may be taken from, but it must be replaced afterwards.
// It is a runtime error to attempt to take the box without replacing it beforehand.
// It is also a runtime error to attempt to replace something into a box that has not been taken from.
#[derive(Serialize,Deserialize)]
pub struct TakeBox<T: ?Sized> {
    this: Option<Box<T>>
}

impl<T: ?Sized> TakeBox<T> {
    pub fn new(x: T) -> Self where T: Sized {
        Self {this: Some(Box::new(x))}
    }
    pub fn newb(x: Box<T>) -> Self {
        Self {this: Some(x)}
    }
    pub fn none() -> Self {
        Self {this: None}
    }

    pub fn is_some(&self) -> bool {self.this.is_some()}
    pub fn is_none(&self) -> bool {self.this.is_none()}
    
    // Note Rust DOES NOT throw an error
    // if these functions are used and this box is taken from while a reference exists!
    // I do not understand this.
    pub fn as_ref(&self) -> &T {
        return self.this.as_ref().unwrap().as_ref();
    }
    pub fn as_mut(&mut self) -> &mut T {
        return self.this.as_mut().unwrap().as_mut();
    }
    
    pub fn take(&mut self) -> Box<T> {
        return self.this.take().unwrap();
    }
    
    pub fn replace(&mut self, x: Box<T>) {
        self.this.replace(x);
    }
}

impl<T: ?Sized> Default for TakeBox<T> {
    fn default() -> Self { 
        Self {this: None}
    }
}
