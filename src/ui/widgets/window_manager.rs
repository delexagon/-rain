use super::widget_package::*;

fn draw_header(buffer: &mut WidgetBuffer, x: u16, y: u16, width: u16, highlighted: bool) {
    buffer.move_to(x,y);
    if highlighted {
        let _ = buffer.wstr(&(0..width).map(|_| "▄").collect::<String>(), Style::from_fg(Rgb(137,207,240)));
    } else {
        let _ = buffer.wstr(&(0..width).map(|_| "▄").collect::<String>(), Style::default());
    }
}

enum ClickLocation {
    Header(usize),
    PassOn(usize),
    BottomRight(usize),
    None,
}

#[derive(Default)]
enum Grab {
    #[default]
    None,
    BottomRight((u16, u16), (u16,u16)),
    Header((u16, u16), (u16, u16))
}

fn subtract_minimum(a: u16, b: u16, min: u16) -> u16 {
    if b+min > a {min} else {a-b}
}

#[derive(Serialize,Deserialize)]
pub struct WindowManager {
    front_to_back: Vec<(usize, (u16,u16), WidgetBound)>,
    #[serde(skip)]
    current_grab: Grab,
}

impl WindowManager {
    pub fn new() -> Self {
        Self {
            front_to_back: Vec::new(),
            current_grab: Grab::None,
        }
    }
    
    // Clicks should be offset by whatever
    fn what_does_this_click_do(&self, click_x: u16, click_y: u16) -> ClickLocation {
        for i in 0..self.front_to_back.len() {
            let (child_num, (x,y),this_bound) = self.front_to_back[i];
            // y is correct; 1 is added.
            if click_y == y-1 && click_x >= x && click_x < x+this_bound.width {
                return ClickLocation::Header(i);
            } else if click_y+1 == y+this_bound.height && 
                (click_x == x+this_bound.width-1 || click_x == x+this_bound.width) {
                // A 2-character wide grab width because 1 character seemed to be too small
                // (crossterm does not read click locations well)
                return ClickLocation::BottomRight(i);
            } else if click_y >= y && click_y < y+this_bound.height && click_x >= x && click_x < this_bound.width+x {
                return ClickLocation::PassOn(child_num);
            }
        }
        return ClickLocation::None;
    }
    
    pub fn selected_child(&self) -> usize {
        return self.front_to_back[0].0;
    }
    
    fn change_selected(&mut self, which: usize) {
        self.front_to_back.swap(which, 0);
    }
}

impl Widget for WindowManager {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {
        let mut vec = vec![WidgetBound {width: 0,height:0}; self.front_to_back.len()];
        for i in 0..self.front_to_back.len() {
            vec[self.front_to_back[i].0] = self.front_to_back[i].2;
        }
        return vec;
    }

    fn child_number(&mut self, desired: usize) -> usize {
        for i in 0..desired {
            self.front_to_back.push((i, (0,0), WidgetBound {width: 30, height: 10}));
        }
        desired
    }

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let bound = buffer.bound();
        for _row in 0..bound.height {
            buffer.blank_till_end(Style::default());
        }
        if self.front_to_back.len() == 0 {return}
        let first = self.front_to_back[0].0;
        for (child_num, (x,y),bound) in &self.front_to_back {
            draw_header(buffer, *x, y-1, bound.width, *child_num == first);
            children[*child_num].copy_to(((*x,*y),(0,0)), *bound, buffer);
        }
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        if let Event::Mouse(mouse) = event {
            match self.current_grab {
                Grab::BottomRight(..) | Grab::Header(..) => match mouse.kind {
                    // Release it, there's a problem;
                    // shouldn't be able to click when dragging
                    MouseEventKind::Down(MouseButton::Left) |
                    MouseEventKind::Moved => {
                        self.current_grab = Grab::None;
                    },
                    // The expected behavior
                    MouseEventKind::Up(MouseButton::Left) => {
                        self.current_grab = Grab::None;
                        // The event has been consumed
                        return EventResult::Nothing;
                    },
                    MouseEventKind::Drag(MouseButton::Left) => {
                        match self.current_grab {
                            // Move the window
                            Grab::Header(original_pos, original_mouse) => {
                                let (child_num, old_loc, bound) = self.front_to_back[0];
                                let new_x = subtract_minimum(original_pos.0+mouse.column, original_mouse.0, 0);
                                // The minimum y is 1, because we need space for the header.
                                let new_y = subtract_minimum(original_pos.1+mouse.row, original_mouse.1, 1);
                                self.front_to_back[0] = (
                                    child_num,
                                    (new_x, new_y),
                                    bound
                                );
                                // The event has been consumed
                                return EventResult::Changed;
                            },
                            // Resize the window
                            Grab::BottomRight(original_size, original_mouse) => {
                                let (child_num, old_loc, bound) = self.front_to_back[0];
                                let new_width = subtract_minimum(original_size.0+mouse.column, original_mouse.0, 1);
                                // Minimum height of 1
                                let new_height = subtract_minimum(original_size.1+mouse.row, original_mouse.1, 1);
                                self.front_to_back[0] = (
                                    child_num,
                                    old_loc,
                                    WidgetBound {
                                        width: new_width,
                                        height: new_height
                                    }
                                );
                                // The event has been consumed
                                return EventResult::Changed;
                            },
                            Grab::None => ()
                        };
                    },
                    _ => ()
                },
                Grab::None => ()
            };
            // Standard behavior; no current grab or mouse is irrelevant to grab
            let location = self.what_does_this_click_do(mouse.column, mouse.row);
            match (location, mouse.kind) {
                (ClickLocation::Header(child), MouseEventKind::Down(MouseButton::Left)) => {
                    let (_, (x,y), bound) = self.front_to_back[child];
                    self.current_grab = Grab::Header((x, y), (mouse.column, mouse.row));
                    self.change_selected(child);
                    return EventResult::Changed;
                },
                (ClickLocation::BottomRight(child), MouseEventKind::Down(MouseButton::Left)) => {
                    let (_, (x,y), bound) = self.front_to_back[child];
                    self.current_grab = Grab::BottomRight((bound.width, bound.height), (mouse.column, mouse.row));
                    self.change_selected(child);
                    return EventResult::Changed;
                },
                (ClickLocation::PassOn(child), MouseEventKind::Down(_)) |
                (ClickLocation::PassOn(child), MouseEventKind::ScrollUp) |
                (ClickLocation::PassOn(child), MouseEventKind::ScrollDown) => {
                    self.change_selected(child);
                    return EventResult::Changed;
                },
                (ClickLocation::PassOn(child), _) => {
                    return EventResult::PassToChild(child as i32);
                },
                _ => return EventResult::Nothing
            }
        } else {
            if let Event::FocusLost = event {
                self.current_grab = Grab::None;
            }
            return EventResult::PassToChild(self.selected_child() as i32);
        }
    }
}
