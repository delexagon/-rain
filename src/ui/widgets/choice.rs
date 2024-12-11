use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct Choice {
    lines: Vec<String>,
    pub selected: usize,
    looping: bool,
}

impl Choice {
    pub fn from_vec(vec: Vec<String>, selected: usize, looping: bool) -> Self {
        Self {
            lines: vec,
            selected,
            looping,
        }
    }

    pub fn shift_left(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        } else if self.looping {
            self.selected = self.lines.len()-1;
        }
    }
    pub fn shift_right(&mut self) {
        if self.selected < self.lines.len()-1 {
            self.selected += 1;
        } else if self.looping {
            self.selected = 0;
        }
    }
}

impl Widget for Choice {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}
    
    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        buffer.clear();
        let len = self.lines.len();
        if len == 0 {
            return;
        }
        let bound = buffer.bound();
        let string_len = self.lines[self.selected].chars().count() as u16;
        let start = inner_cut(bound.width, string_len, 0.5);
        buffer.move_to(start, 0);
        let _ = buffer.wstr(&self.lines[self.selected], Style::default());
        if self.looping || self.selected > 0 {
            buffer.wchar_at((0, 0), '<', Style::default());
        }
        if self.looping || self.selected < self.lines.len()-1 {
            buffer.wchar_at((bound.width as usize-1, 0), '>', Style::default());
        }
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        // interactivity is manual; use shift_left and shift_right
        // on receiving arrow keys
        EventResult::Nothing
    }
}
