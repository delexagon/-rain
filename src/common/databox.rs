use std::cell::{RefCell, RefMut, Ref};
use std::rc::Rc;

// This form of box is copyable and writable.
// This means that multiple copies may exist of it, but it is a runtime error to make two copies writable at the same time.
// Note that care must be taken to not create cycles.
pub struct DataBox<T> {
    data: Rc<RefCell<T>>,
}

impl<T> DataBox<T> {
    pub fn new(t: T) -> Self { Self { data: Rc::new(RefCell::new(t)) } }
    
    pub fn write(&self) -> RefMut<T> {
        return self.data.borrow_mut();
    }
    
    pub fn read(&self) -> Ref<T> {
        return self.data.borrow();
    }
    
}

impl<T> Clone for DataBox<T> {
    fn clone(&self) -> Self { Self { data: self.data.clone() } }
}
