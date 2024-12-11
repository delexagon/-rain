use super::{Behavior, EntityID};
use crate::CoupledHeap;
use std::cmp::Ordering;
use serde::{Serialize,Deserialize};

pub type Time = usize;

#[derive(Debug,Serialize,Deserialize)]
pub struct Update {
    pub time: Time,
    pub behavior: Behavior,
}

impl PartialEq for Update {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time
    }
} impl Eq for Update {}

impl PartialOrd for Update {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.time.cmp(&other.time))
    }
}

impl Ord for Update {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

#[derive(Serialize,Deserialize)]
pub struct UpdateHandler {
    pub next_update_id: u32,
    pub current_time: Time,
    update_heap: CoupledHeap<EntityID, Update>
}

impl UpdateHandler {
    pub fn new() -> Self {
        Self {
            current_time: 0,
            next_update_id: 0,
            update_heap: CoupledHeap::new(),
        }
    }
    
    pub fn add_update(&mut self, time_until: Time, ent_id: EntityID, behavior: Behavior) {
        let time_of_update = self.current_time+time_until;
        let update = Update {
            time: time_of_update,
            behavior: behavior,
        };
        self.update_heap.push(ent_id, update);
    }
    
    pub fn next(&mut self) -> Option<(EntityID, Update)> {
        let some_update = self.update_heap.pop();
        match some_update {
            Some(upd) => {
                self.current_time = upd.1.time;
                Some(upd)
            },
            None => None,
        }
    }

    pub fn insert(&mut self, entity: EntityID, vec: Vec<Update>) {
        self.update_heap.insert(entity, vec);
    }

    pub fn remove(&mut self, entity: EntityID) -> Vec<Update> {
        return self.update_heap.remove(entity);
    }
    
    pub fn print(&self) {
        println!("{:?}", self.update_heap);
    }
    
    pub fn len(&self) -> usize {self.update_heap.len()}
}
