use super::widget_package::*;

/// Displays a list of child widgets vertically.
#[derive(Deserialize,Serialize)]
pub struct LinesChildren {
    lines: Vec<u16>,
    current_first_line: usize,
    selected: usize,
}

impl LinesChildren {
    pub fn new() -> Self {
        let var = Self {
            lines: Vec::new(),
            selected: 0,
            current_first_line: 0,
        };
        return var;
    }

    pub fn from_vec(vec: Vec<u16>) -> Self {
        Self {
            lines: vec,
            selected: 0,
            current_first_line: 0,
        }
    }

    pub fn get(&self, line: usize) -> u16 {self.lines[line]}
}

impl Widget for LinesChildren {
    fn child_number(&mut self,desired:usize) -> usize {
        if self.lines.len() != 0 {
            self.lines.len()
        } else {
            for i in 0..desired {
                self.lines.push(1);
            }
            desired
        }
    }
    fn child_sizes(&self,bound:WidgetBound) -> Vec<WidgetBound> {
        let mut v = Vec::new();
        for height in &self.lines {
            v.push(WidgetBound {width: bound.width-1, height: *height});
        }
        return v;
    }

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        buffer.clear();
        let bound = buffer.bound();
        let mut next_child_y = 0;
        for (i, height) in self.lines.iter().enumerate() {
            if next_child_y >= bound.height {
                return;
            }
            
            if i == self.selected {
                buffer.wchar_at((0, next_child_y as usize), '•', Style::default())
            }
            children[i].copy_to((
                (1,next_child_y),(0,0)), 
                WidgetBound {width: bound.width-1, height: *height}, buffer
            );

            next_child_y += height;
        }
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        match event_translation {
            Some(Candidate::Up) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                return EventResult::Changed;
            },
            Some(Candidate::Down) => {
                if self.selected < self.lines.len()-1 {
                    self.selected += 1;
                }
                return EventResult::Changed;
            },
            _ => ()
        };
        return EventResult::PassToChild(self.selected as i32);
    }
}
