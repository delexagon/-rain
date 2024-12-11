use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct LinesHorizontal {
    lines: Vec<String>,
    selected: usize,
    selectable: bool,
}

impl LinesHorizontal {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            selected: 0,
            selectable: true,
        }
    }

    pub fn from_vec(vec: Vec<String>) -> Self {
        Self {
            lines: vec,
            selected: 0,
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

impl Widget for LinesHorizontal {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}
    
    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let len = self.lines.len();
        if len == 0 {
            return;
        }
        let bound = buffer.bound();
        if len == 1 {
            let string_len = self.lines[0].chars().count() as u16;
            let start = inner_cut(bound.width, string_len, 0.5);
            buffer.move_to(start, 0);
            if self.selectable {
                let _ = buffer.wstr(&self.lines[0], Style::default().reverse());
            } else {
                let _ = buffer.wstr(&self.lines[0], Style::default());
            }
            return;
        }
        let space = 1./((len-1) as f64);
        for i in 0..len {
            let alignment = space*i as f64;
            let string_len = self.lines[i].chars().count() as u16;
            let start = inner_cut(bound.width, string_len, alignment);
            buffer.move_to(start, 0);
            if self.selectable && i == self.selected {
                let _ = buffer.wstr(&self.lines[i], Style::default().reverse());
            } else {
                let _ = buffer.wstr(&self.lines[i], Style::default());
            }
        }
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        match event_translation {
            Some(Candidate::Left) => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                return EventResult::Changed;
            },
            Some(Candidate::Right) => {
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
