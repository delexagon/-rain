use crate::common::{DataBox, UITile, BLANKTILE, FILLEDTILE};
use crate::ui::{UI, Widget, Action, DrawBound, Key};
use crossterm::event::{Event, KeyEvent, KeyCode, MouseEventKind, MouseButton};
use std::collections::HashMap;
use std::iter::zip;

fn draw_header(ui: &DataBox<UI>, x: u16, y: u16, width: u16, highlighted: bool) {
    let mut write_ui = ui.write();
    write_ui.goto(x,y);
    if highlighted {
        write_ui.set_fg(137,207,240);
        write_ui.term.write::<String>(&(0..width).map(|_| "▅").collect::<String>());
        write_ui.term.clear_style();
    } else {
        write_ui.term.write::<String>(&(0..width).map(|_| "▅").collect::<String>());
    }
}

enum ClickEffect {
    Grab(usize),
    PassOn(usize, u16, u16),
    None,
}

pub struct WindowManager {
    // Ordered: front to back
    children: Vec<usize>,
    children_positions: Vec<DrawBound>,
    action_map: HashMap<Key, Box<dyn Action>>,
    current_child: usize,
    grab_starting_position: Option<(u16, u16, u16, u16)>,
    just_moved: bool,
}


impl WindowManager {
    fn new_with() -> Self {
        Self {
            children: Vec::new(),
            action_map: HashMap::new(),
            grab_starting_position: None,
            children_positions: Vec::new(),
            current_child: 0,
            just_moved: false,
        }
    }
    
    // Clicks should be offset by whatever
    fn what_does_this_click_do(&self, click_x: u16, click_y: u16) -> ClickEffect {
        let mut i = self.children.len();
        while i > 0 {
            i -= 1;
            let this_bound = self.children_positions[i];
            // y is correct; 1 is added.
            if click_y == this_bound.y && click_x >= this_bound.x && click_x < this_bound.end_x() {
                return ClickEffect::Grab(i);
            } else if click_y > this_bound.y && click_y <= this_bound.end_y() && click_x >= this_bound.x && click_x < this_bound.end_x() {
                return ClickEffect::PassOn(i, click_x - this_bound.x, click_y - this_bound.y - 1);
            }
        }
        return ClickEffect::None;
    }
    
    pub fn selected(&self) -> usize {
        return self.children[self.children.len()-1];
    }
    
    fn change_selected(&mut self, which: usize) {
        let end = self.children.len()-1;
        self.children.swap(which, end);
        self.children_positions.swap(which, end);
    }
    
    pub fn keymap(&mut self, map: HashMap<Key, Box<dyn Action>>) {
        self.action_map = map;
    }
    
    pub fn add_child(&mut self, window: usize) {
        self.children.push(window);
        self.children_positions.push(DrawBound {x: 0, y: 0, width: 20, height: 10})
    }
    
    pub fn add_child_leave_controlled(&mut self, window: usize) {
        self.children.push(window);
        self.children_positions.push(DrawBound {x: 0, y: 0, width: 20, height: 10});
        let end = self.children.len()-1;
        self.children.swap(end-1, end);
        self.children_positions.swap(end-1, end);
    }
}

impl Widget for WindowManager {
    fn new_unboxed() -> Self { Self::new_with() }
    
    fn draw(me: DataBox<Self>, ui: DataBox<UI>, bound: DrawBound, force: bool) -> bool {
        let read_me = me.read();
        let mut force_redraw = force;
        if read_me.just_moved {
            let mut write_ui = ui.write();
            for row in bound.y..bound.end_y() {
                write_ui.goto(bound.y,row);
                write_ui.term.write::<String>(&(0..bound.width).map(|_| " ").collect::<String>());
            }
            force_redraw = true;
        }
        for i in 0..read_me.children.len() {
            let theoretical_bound = read_me.children_positions[i];
            let real_x = bound.x + theoretical_bound.x;
            let real_y = bound.y + theoretical_bound.y + 1;
            let real_bound = DrawBound {
                // x is added to bound
                x: real_x,
                // 1 is added to y bound; space for grab
                y: real_y,
                // height is limited at edge of screen
                height: {
                    if real_y+theoretical_bound.height >= bound.end_y() {
                        if real_y > bound.end_y() {
                            0
                        } else {
                            bound.end_y() - real_y
                        }
                    } else {
                        theoretical_bound.height
                    }
                },
                width: {
                    if real_x+theoretical_bound.width >= bound.end_x() {
                        if real_x > bound.end_x() {
                            0
                        } else {
                            bound.end_x() - real_x
                        }
                    } else {
                        theoretical_bound.width
                    }
                },
            };
            let header_x = real_bound.x;
            let header_y = real_bound.y-1;
            let header_width = real_bound.width;
            draw_header(&ui, header_x, header_y, header_width, i == read_me.children.len()-1);
            UI::draw_one(ui.clone(), read_me.children[i], real_bound, force_redraw);
        }
        return true;
    }
    
    fn consume_action(me: DataBox<Self>, ui: DataBox<UI>, event: Event) -> Option<Box<dyn Action>> {
        match event {
            Event::Key(key) => {
                let reformat_key = Key {code: key.code, modifiers: key.modifiers};
                let read_me = me.read();
                let maybe_action = read_me.action_map.get(&reformat_key);
                match maybe_action {
                    Some(a) => return Some(a.awful_clone()),
                    None => {
                        let relevant_widget = read_me.selected();
                        return UI::consume_action_one(ui, relevant_widget, event);
                    },
                }
            },
            Event::Mouse(mouse) => {
                match mouse.kind {
                    MouseEventKind::Down(MouseButton::Left) => {
                        let mut write_me = me.write();
                        let effect = write_me.what_does_this_click_do(mouse.column, mouse.row);
                        match effect {
                            ClickEffect::Grab(some_window) => {
                                let start_bound = write_me.children_positions[some_window];
                                write_me.grab_starting_position = Some((start_bound.x, start_bound.y, mouse.column, mouse.row));
                                write_me.change_selected(some_window);
                                return None;
                            },
                            _ => (),
                        }
                    },
                    MouseEventKind::Drag(MouseButton::Left) => {
                        let mut write_me = me.write();
                        match write_me.grab_starting_position {
                            Some((bound_x, bound_y, mouse_x, mouse_y)) => {
                                let window = write_me.children.len()-1;
                                write_me.just_moved = true;
                                let old_bound = write_me.children_positions[window];
                                let new_x = if mouse_x > bound_x+mouse.column {
                                    0
                                } else {
                                    (bound_x+mouse.column)-mouse_x
                                };
                                let new_y = (bound_y+mouse.row)-mouse_y;
                                write_me.children_positions[window] = DrawBound {
                                    x: new_x,
                                    y: new_y,
                                    width: old_bound.width,
                                    height: old_bound.height,
                                };
                            },
                            None => (),
                        }
                    },
                    MouseEventKind::Up(MouseButton::Left) => {
                        me.write().just_moved = false;
                        me.write().grab_starting_position = None;
                    },
                    _ => (),
                }
                
            },
            Event::FocusLost => {
                me.write().just_moved = false;
                me.write().grab_starting_position = None;
            },
            _ => (),
        }
        return None;
    }
}
