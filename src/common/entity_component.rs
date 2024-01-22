use std::any::Any;
use std::collections::HashMap;

pub struct EntityComponentSystem {
    pub id: u32,
    pub components: Vec<Box<dyn Any>>,
}

impl EntityComponentSystem {
    pub fn new() -> EntityComponentSystem { EntityComponentSystem { id: 0, components: Vec::new(), } }
    
    pub fn create_component<T: 'static>(&mut self) -> usize {
        let new_component: Box<Vec<T>> = Box::new(Vec::new());
        self.components.push(new_component);
        return self.components.len();
    }
    
    pub fn component<T: 'static>(&mut self, component: usize) -> &mut Vec<T> {
        return self.components[component].downcast_mut::<Vec<T>>().unwrap();
    }
    
    pub fn fetch<T: 'static>(&mut self, component: usize, me: usize) -> &mut T {
        return self.components[component].downcast_mut::<Vec<T>>().unwrap().get_mut(me).unwrap();
    }
    
    pub fn replace<T: 'static>(&mut self, component: usize, me: usize, thing: T) {
        self.components[component].downcast_mut::<Vec<T>>().unwrap()[me] = thing;
    }
}
