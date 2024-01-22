use crate::common::DataBox;
use crate::ui::{UI, Widget, Action, DrawBound, Key};
use crossterm::event::{Event, KeyEvent, KeyCode};
use std::collections::HashMap;

pub struct Capturer<RetAction: Action> {
    child: usize,
    action_map: HashMap<Key, RetAction>,
}


impl<RetAction: Action> Capturer<RetAction> {
    pub fn keymap(&mut self, map: HashMap<Key, RetAction>) {
        self.action_map = map;
    }
    
    pub fn set_child(&mut self, child: usize) {
        self.child = child;
    }
}

impl<RetAction: Action + std::clone::Clone> Widget for Capturer<RetAction> {
    fn new_unboxed() -> Self { Self { child: 0, action_map: HashMap::new() } }
    
    fn draw(me: DataBox<Self>, ui: DataBox<UI>, bound: DrawBound, force: bool) -> bool {
        let id;
        { id = me.read().child; }
        return UI::draw_one(ui, id, bound, force);
    }
    
    fn consume_action(me: DataBox<Self>, ui: DataBox<UI>, event: Event) -> Option<Box<dyn Action>> {
        let id;
        { id = me.read().child; }
        match event {
            Event::Key(key) => {
                let reformat_key = Key {code: key.code, modifiers: key.modifiers};
                let x = me.read();
                let maybe_action = x.action_map.get(&reformat_key);
                match maybe_action {
                    Some(a) => return Some(a.bad_clone()),
                    None => return UI::consume_action_one(ui, id, event),
                }
            },
            _ => return UI::consume_action_one(ui, id, event),
        }
    }
}
