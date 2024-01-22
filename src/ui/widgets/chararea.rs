use crate::common::{DataBox, UITile, BLANKTILE, FILLEDTILE};
use crate::ui::{UI, Widget, Action, DrawBound, Key};
use crossterm::event::{Event, KeyEvent, KeyCode};
use std::collections::HashMap;

// Calculate where an inner box should be for a given alignment
// In this case, length and width are along the same axis
fn calc_start(align: f64, outer_width: u16, inner_width: u16) -> i32 {
    let inner_alignment_point = (align*(inner_width as f64).round()) as i32;
    let outer_alignment_point = (align*(outer_width as f64).round()) as i32;
    let inner_alignment_offset = outer_alignment_point - inner_alignment_point;
    return inner_alignment_offset;
}

pub const DEFAULT_DIMENSIONS: (u16, u16) = (40,20);

pub struct CharArea<RetAction: Action> {
    chars: Vec<UITile>,
    // Of the inner chars
    width: u16,
    height: u16,
    last_bound: DrawBound,
    row_alignment: f64,
    col_alignment: f64,
    default_ch: char,
    changed: bool,
    action_map: HashMap<Key, RetAction>,
}


impl<RetAction: Action> CharArea<RetAction> {
    fn new_with(width: u16, height: u16) -> Self {
        Self {
            chars: vec!(FILLEDTILE; (width*height) as usize),
            width: width,
            height: height,
            row_alignment: 0.5,
            col_alignment: 0.5,
            last_bound: DrawBound {
                x: 0,
                y: 0,
                height: 32,
                width: 32,
            },
            default_ch: ' ',
            changed: true,
            action_map: HashMap::new(),
        }
    }
    
    pub fn keymap(&mut self, map: HashMap<Key, RetAction>) {
        self.action_map = map;
    }

    pub fn set(&mut self, ch: Vec<UITile>) {
        self.changed = true;
        self.chars = ch;
    }
}

impl<RetAction: Action + std::clone::Clone> Widget for CharArea<RetAction> {
    fn new_unboxed() -> Self { Self::new_with(DEFAULT_DIMENSIONS.0,DEFAULT_DIMENSIONS.1) }
    
    fn draw(me: DataBox<Self>, ui: DataBox<UI>, bound: DrawBound, force: bool) -> bool {
        if !force && !me.read().changed && me.read().last_bound == bound {
            return false;
        }
        {
            let mut write_me = me.write();
            write_me.changed = false;
            write_me.last_bound = bound;
        }
        
        let read_me = me.read();
        
        let area_y_offset_from_window = calc_start(read_me.row_alignment, bound.height, read_me.height);
        let area_x_offset_from_window = calc_start(read_me.col_alignment, bound.width, read_me.width);
        
        let mut write_ui = ui.write();
        
        for row in bound.y..bound.end_y() {
            write_ui.goto(bound.x, row);
            let inner_row = (row as i32 - bound.y as i32) - area_y_offset_from_window;
            for col in bound.x..bound.end_x() {
                let inner_col = (col as i32 - bound.x as i32) - area_x_offset_from_window;
                if inner_row < 0 || inner_col < 0 ||
                   inner_col >= read_me.width as i32 || inner_row >= read_me.height as i32 {
                    write_ui.term.write_uitile(&BLANKTILE);
                } else {
                    write_ui.term.write_uitile(
                        &read_me.chars[(inner_row*read_me.width as i32+inner_col) as usize]);
                }
            }
        }
        return true;
    }
    
    fn consume_action(me: DataBox<Self>, ui: DataBox<UI>, event: Event) -> Option<Box<dyn Action>> {
        match event {
            Event::Key(key) => {
                let reformat_key = Key {code: key.code, modifiers: key.modifiers};
                let x = me.read();
                let maybe_action = x.action_map.get(&reformat_key);
                match maybe_action {
                    Some(a) => return Some(a.bad_clone()),
                    None => return None,
                }
            },
            _ => return None,
        }
    }
}
