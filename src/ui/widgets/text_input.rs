
use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct TextInput {
    string: Vec<char>,
    cursor: usize,
    max_length: usize
}

impl TextInput {
    pub fn new(max_length: usize) -> Self {
        Self {
            string: Vec::with_capacity(max_length),
            max_length,
            cursor: 0
        }
    }

    pub fn set(&mut self, string: &str) {
        self.string = string.chars().take(self.max_length).collect();
        self.cursor = self.string.len();
    }

    pub fn string(&self) -> String {
        return self.string.iter().collect();
    }
}

impl Widget for TextInput {
    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        buffer.clear();
        for i in 0..self.max_length {
            let ch = if self.string.len() > i {self.string[i]} else {'_'};
            if i == self.cursor {
                buffer.wchar(ch, Style::default().reverse());
            } else {
                buffer.wchar(ch, Style::default())
            }
        }
    }

    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        match event {
            // Backspace is interpreted as ctrl+h for some reason sometimes,
            // which is apparently not caught by Crossterm.
            Event::Key(KeyEvent {code: KeyCode::Char('h'), modifiers: mods, ..}) if mods.contains(KeyModifiers::CONTROL) => {
                if self.cursor > 0 {
                    self.string.remove(self.cursor-1);
                    self.cursor -= 1;
                }
                EventResult::Changed
            },
            Event::Key(KeyEvent {code: KeyCode::Backspace, ..}) => {
                if self.cursor > 0 {
                    self.string.remove(self.cursor-1);
                    self.cursor -= 1;
                }
                EventResult::Changed
            },
            Event::Key(KeyEvent {code: KeyCode::Left, ..}) => {
                if self.cursor != 0 {
                    self.cursor -= 1;
                }
                EventResult::Changed
            },
            Event::Key(KeyEvent {code: KeyCode::Right, ..}) => {
                if self.cursor < self.string.len() {
                    self.cursor += 1;
                }
                EventResult::Changed
            },
            Event::Key(KeyEvent {code: KeyCode::Char(ch), ..}) => {
                if self.string.len() < self.max_length {
                    self.string.insert(self.cursor, ch);
                    self.cursor += 1;
                }
                EventResult::Changed
            },
            _ => EventResult::Nothing,
        }
    }
}
