use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct LineScroll {
    chars: Vec<char>,
    breaks: Vec<usize>,
    display_to: usize,
    speed: usize,
    // Location, style change
    styles: Vec<(usize, Style)>,
    // Location, length in frames
    stops: Vec<(usize, usize)>,
    width: usize
}

impl LineScroll {
    fn read_in(&mut self, string: &str) {
        let mut stop_listening = false;
        let mut current_stop = 0;
        for ch in string.chars() {
            if stop_listening {
                match ch {
                    '\x15' => {
                        stop_listening = false;
                        if current_stop > 0 {
                            self.stops.push((self.chars.len(), current_stop));
                            current_stop = 0;
                        }
                    },
                    '0'..='9' => current_stop = current_stop*10+ch as usize-'0' as usize,
                    _ => (),
                }
            } else {
                match ch {
                    '\x14' => stop_listening = true,
                    _ => self.chars.push(ch),
                }
            }
        }
    }

    pub fn new(string: &str, speed: usize) -> Self {
        let mut this = Self {
            chars: Vec::new(),
            display_to: 0,
            speed,
            styles: Vec::new(),
            stops: Vec::new(),
            breaks: Vec::new(),
            width: 0,
        };
        this.read_in(string);
        this
    }

    pub fn len(&self) -> usize {self.chars.len()}

    pub fn set_style(&mut self, style: Style) {self.styles.clear(); self.styles.push((0, style));}

    pub fn with_style(mut self, style: Style) -> Self {self.styles.clear(); self.styles.push((0, style)); self}
    pub fn finished(mut self) -> Self {self.display_to = self.chars.len(); self}

    pub fn change(&mut self, string: &str) {
        self.chars.clear();
        self.stops.clear();
        self.styles.clear();
        self.read_in(string);
        self.display_to = 0;
        self.breaks.clear();
        line_breaks(&mut self.breaks, &self.chars, self.width);
    }

    pub fn extend(&mut self, string: &str) {
        self.read_in(string);
        self.breaks.clear();
        line_breaks(&mut self.breaks, &self.chars, self.width);
    }

    pub fn done(&self) -> bool {self.display_to >= self.chars.len()}
}

impl Widget for LineScroll {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}
    
    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let mut style = Style::default();
        buffer.clear();
        let mut i = 0;
        let mut next_break = 0;
        let mut next_style = 0;
        while i < self.display_to {
            if next_style < self.styles.len() && self.styles[next_style].0 == i {
                style = self.styles[next_style].1;
                next_style += 1;
            }
            if next_break < self.breaks.len() && self.breaks[next_break] == i {
                buffer.next_line();
                next_break += 1;
            }
            match self.chars[i] {
                '\n' => (),
                '\t' => {
                    for _ in 0..4 {
                        buffer.wchar(' ', style)
                    }
                },
                _ => buffer.wchar(self.chars[i], style)
            };
            i += 1;
        }
    }

    fn update_size(&mut self, bound: WidgetBound, buffer: &mut WidgetBuffer) -> WidgetBound {
        buffer.resize(bound.into());
        self.width = bound.width as usize;
        self.breaks.clear();
        line_breaks(&mut self.breaks, &self.chars, self.width);
        bound
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::Nothing
    }

    fn animates(&self) -> bool {true}

    fn next_frame(&mut self, buffer: &mut WidgetBuffer) -> bool {
        if self.stops.len() > 0 && self.stops[0].0 == self.display_to {
            self.stops[0].1 -= 1;
            if self.stops[0].1 == 0 {
                self.stops.remove(0);
            }
        } else if !self.done() {
            self.display_to += 1;
            self.draw(&mut [], buffer);
        }
        return !self.done();
    }
}
