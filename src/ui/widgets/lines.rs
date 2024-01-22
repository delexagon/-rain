use crate::common::DataBox;
use crate::ui::{UI, Widget, Action, DrawBound};
use crossterm::event::{Event, KeyEvent, KeyCode};

fn break_lines(lines: &Vec<String>, width: usize) -> Vec<String> {
    let mut broken_lines = Vec::with_capacity(lines.len());
    for line in lines.iter() {
        if line.len() < width {
            broken_lines.push(line.clone());
        } else {
            let mut i = 0;
            while i < line.len() {
                if i+width >= line.len() {
                    broken_lines.push(String::from(&line[i..]));
                    break;
                }
                broken_lines.push(String::from(&line[i..i+width]));
                i += width;
            }
        }
    }
    return broken_lines;
}

pub struct Lines<RetAction> where RetAction: Action + std::clone::Clone {
    lines: Vec<String>,
    changed: bool,
    current_first_line: usize,
    selected: usize,
    selectable: bool,
    last_bound: DrawBound,
    pub select_func: Option<fn(DataBox<Self>, DataBox<UI>, usize) -> RetAction>,
    associated_selection_actions: Vec<RetAction>,
}

impl<RetAction: Action + std::clone::Clone> Lines<RetAction> {
    pub fn add_line(&mut self, st: String) {
        self.lines.push(st);
    }
    
    pub fn get_selection(me: DataBox<Self>, ui: DataBox<UI>) -> Option<RetAction> {
        if me.read().select_func == None {
            return None;
        }
        else {
            let line = me.read().selected;
            let select_func;
            { select_func = me.read().select_func.unwrap(); }
            return Some(select_func(me, ui, line));
        }
    }
}

impl<RetAction: Action + std::clone::Clone> Widget for Lines<RetAction> {
    fn new_unboxed() -> Self {
        let mut var = Self {
            lines: Vec::new(),
            selected: 0,
            current_first_line: 0,
            select_func: None,
            changed: true,
            selectable: true,
            last_bound: DrawBound {
                height: 32,
                width: 32,
                x: 0,
                y: 0,
            },
            associated_selection_actions: Vec::new(),
        };
        return var;
    }

    fn draw(me: DataBox<Self>, ui: DataBox<UI>, bound: DrawBound, force: bool) -> bool {
        if !force && !me.read().changed && me.read().last_bound == bound {
            return false;
        }
        if bound.y >= bound.height || bound.x >= bound.width {
          return false;
        }
        {
        let mut write_ui = ui.write();
        let read_me = me.read();
        
        let end = bound.end_x();
        
        let end_modified = if end > bound.width {bound.width as usize} else {end as usize};
        
        let start2 = bound.x as usize;
        
        for row in bound.y .. bound.end_y() {
            write_ui.goto(bound.x, row);
            let row2 = row as usize;
            if read_me.lines.len() <= row2 {
                write_ui.term.write::<String>(&(" ".repeat(end_modified-start2)));
                continue;
            }
            let end_string_modified = if end_modified > read_me.lines[row2].len() 
                {read_me.lines[row2].len()} else {end_modified};
            if read_me.selectable && row2 == read_me.selected {
                write_ui.term.reverse();
                write_ui.term.write::<str>(&read_me.lines[row2][start2..end_string_modified]);
                write_ui.term.stop_reverse();
            } else {
                write_ui.term.write::<str>(&read_me.lines[row2][start2..end_string_modified]);
            }
            write_ui.term.write::<String>(&(" ".repeat(end_modified-end_string_modified)));
        }
        }
        me.write().last_bound = bound;
        me.write().changed = false;
        return true;
    }
    
    fn consume_action(me: DataBox<Self>, ui: DataBox<UI>, event: Event) -> Option<Box<dyn Action>> {
        match event {
            Event::Key(KeyEvent{code: KeyCode::Up, .. }) => {
                if me.read().selected > 0 {
                    me.write().selected -= 1;
                    me.write().changed = true;
                }
                return None;
            },
            Event::Key(KeyEvent{code: KeyCode::Down, .. }) => {
                if me.read().selected < me.read().lines.len()-1 {
                    me.write().selected += 1;
                    me.write().changed = true;
                }
                return None;
            },
            Event::Key(KeyEvent{code: KeyCode::Enter, .. }) => {
                let s = Self::get_selection(me, ui);
                match s {
                    Some(i) => return Some(Box::new(i)),
                    None => return None,
                }
            }
            _ => return None,
        }
    }
}
