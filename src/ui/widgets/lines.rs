use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct Lines {
    lines: Vec<String>,
    current_first_line: usize,
    selected: usize,
    selectable: bool,
}

impl Lines {
    pub fn new() -> Self {
        let var = Self {
            lines: Vec::new(),
            selected: 0,
            current_first_line: 0,
            selectable: true,
        };
        return var;
    }

    pub fn from_vec(vec: Vec<String>) -> Self {
        Self {
            lines: vec,
            selected: 0,
            current_first_line: 0,
            selectable: true,
        }
    }

    pub fn set(&mut self, line: usize, text: String) {
        self.lines[line] = text;
    }

    pub fn get(&self, line: usize) -> &str {&self.lines[line]}

    pub fn add_line(&mut self, st: String) {
        self.lines.push(st);
    }
}

impl Widget for Lines {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        buffer.clear();
        let bound = buffer.bound();
        for row in 0..bound.height {
            let row2 = row as usize;
            if self.lines.len() <= row2 {
                buffer.blank_till_end(Style::default());
                continue;
            }
            if self.selectable && row2 == self.selected {
                let _ = buffer.wstr(&self.lines[row2], Style::default().reverse());
            } else {
                let _ = buffer.wstr(&self.lines[row2], Style::default());
            }
            buffer.blank_till_end(Style::default());
        }
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        match event_translation {
            Some(Candidate::Up) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            },
            Some(Candidate::Down) => {
                if self.selected < self.lines.len()-1 {
                    self.selected += 1;
                }
                return EventResult::Changed;
            },
            Some(Candidate::Enter) => {
                if poll.contains(&(my_id, Candidate::Select)) {
                    let line = self.selected;
                    return EventResult::PollResult((my_id, Match::Selection1D(line as u8)));
                }
            },
            _ => ()
        };
        EventResult::Nothing
    }
}
