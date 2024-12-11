use super::widget_package::*;

/// Can only push next lines.
#[derive(Deserialize,Serialize)]
pub struct LinesScroll {
    // Line, character, time left in current stop
    display_to: (usize, usize),
    lines: Vec<Vec<char>>,
    breaks: Vec<Vec<usize>>,
    styles: Vec<Vec<(usize, Style)>>,
    stops: Vec<Vec<(usize, usize)>>,
    width: usize,
    keep: usize,
}

impl LinesScroll {
    fn read_in(chars: &mut Vec<char>, _styles: &mut Vec<(usize, Style)>, stops: &mut Vec<(usize,usize)>, string: &str) {
        let mut stop_listening = false;
        let mut current_stop = 0;
        for ch in string.chars() {
            if stop_listening {
                match ch {
                    '\x15' => {
                        stop_listening = false;
                        if current_stop > 0 {
                            stops.push((chars.len(), current_stop));
                            current_stop = 0;
                        }
                    },
                    '0'..='9' => current_stop = current_stop*10+ch as usize-'0' as usize,
                    _ => (),
                }
            } else {
                match ch {
                    '\x14' => stop_listening = true,
                    _ => chars.push(ch),
                }
            }
        }
    }

    pub fn new(keep: usize) -> Self {
        Self {
            lines: Vec::with_capacity(keep),
            display_to: (0,0),
            styles: Vec::with_capacity(keep),
            stops: Vec::with_capacity(keep),
            keep,
            breaks: Vec::with_capacity(keep),
            width: 0,
        }
    }
    
    pub fn push(&mut self, string: &str) {
        // 😭
        if self.lines.len() >= self.keep {
            self.lines[0].clear();
            self.breaks[0].clear();
            self.styles[0].clear();
            self.stops[0].clear();
            // just because otherwise it would have to reserve data more often
            let a = self.lines.remove(0);
            self.lines.push(a);
            let a = self.breaks.remove(0);
            self.breaks.push(a);
            let a = self.styles.remove(0);
            self.styles.push(a);
            let a = self.stops.remove(0);
            self.stops.push(a);
            if self.display_to.0 == 0 {
                self.display_to = (0,0);
            } else {
                self.display_to.0 -= 1;
            }
        } else {
            self.lines.push(Vec::new());
            self.breaks.push(Vec::new());
            self.styles.push(Vec::new());
            self.stops.push(Vec::new());
        }
        let a = self.lines.len()-1;
        Self::read_in(&mut self.lines[a], &mut self.styles[a], &mut self.stops[a], string);
        line_breaks(&mut self.breaks[a], &self.lines[a], self.width);
    }
    
    pub fn done(&self) -> bool {self.display_to.0 == self.lines.len()-1 && self.display_to.1 >= self.lines[self.display_to.0].len()}
}

impl Widget for LinesScroll {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        if self.lines.len() == 0 {
            return;
        }
        buffer.clear();
        let height = buffer.height() as usize;
        let last_line = self.display_to.0;
        // first_drawn_line.0 => the first line which needs to be referenced to fit on the screen
        // first_drawn_line.1 => the first break which needs to be referenced, as each break is a displayed line.
        //  however, 0 is position 0, so first_drawn_line.1-1 is the actual reference in self.breaks.
        //  the number of lines actually drawn here is actually first_drawn_line.1+1,
        //  because position 0 to whatever is also a drawn line.
        let mut first_drawn_line = (last_line,self.breaks[last_line].len());
        // The inverse height remaining. we need to see where this is 0;
        // i.e. where we have to start drawing from so that what we draw actually
        // fits in the goddamn box
        let mut anticonsumed_height = height;
        // we bring the first drawn line back the breaks,
        // until it is before display_to.
        // the break this is referencing, at the end of this, should be the last line that is actually drawn;
        // that is, first_drawn_line.1-1 is the greatest break less than display_to.1
        while first_drawn_line.1 != 0 && self.breaks[last_line][first_drawn_line.1-1] > self.display_to.1 {
            first_drawn_line.1 -= 1;
        }
        // this block moves back to the start of the last line.
        // the number of lines of text which are actually going to be drawn for the last line
        // is equal to first_drawn_line.1+1, so if this is greater
        // than or equal to the height we can just say the remaining height is 0,
        // only the last line is displayed, and the first break that needs to be displayed is
        // the height - 1 (1 line is consumed by where we currently are)
        // first_drawn_line.1 - (so that we go back to the starting line)
        // i think. holy fucking shit.
        if first_drawn_line.1+1>=anticonsumed_height {
            first_drawn_line = (last_line, first_drawn_line.1-(anticonsumed_height-1));
            anticonsumed_height = 0;
        } else {
            // if the last line is not large enough to fill the entire viewbox,
            // we have to go back to the start of it anyway.
            // the height goes back the number of lines that are present in the last line
            anticonsumed_height -= first_drawn_line.1+1;
            // in the loop below, we will go back by full lines.
            // this means that it will handle the subtraction here.
            first_drawn_line = (last_line, 0);
        }
        // while there is still height left, and lines that we still need to display, we continue.
        // note first_drawn_line.1 will always be 0 while this loop is running.
        while anticonsumed_height != 0 && first_drawn_line.0 != 0 {
            // the total number of lines drawn from a line is the len of breaks + 1.
            // if the number of lines in the previous line is less than the remaining height, 
            // we can buck back the entire line and continue.
            if self.breaks[first_drawn_line.0-1].len()+1 < anticonsumed_height {
                // subtract the number of lines from the height
                anticonsumed_height -= self.breaks[first_drawn_line.0-1].len()+1;
                // move the line back 1.
                first_drawn_line = (first_drawn_line.0-1, 0);
            } else {
                // lines in the previous full line are displayed,
                // but only so many 
                first_drawn_line = (first_drawn_line.0-1, self.breaks[first_drawn_line.0-1].len()+1-anticonsumed_height);
                // the number of lines displayed here is greater than or equal to the height
                // 0 height remains
                anticonsumed_height = 0;
            }
        }
        // hopefully, first_drawn_line.1 is now at the first break we actually need to display.
        // remember that 0 is the 0 position, not actually a break in
        // self.breaks
        let loc = if first_drawn_line.1 == 0 {0} else {self.breaks[first_drawn_line.0][first_drawn_line.1-1]};
        // now we can finally go back through in the forwards direction,
        // and draw until the end.
        let mut cur_draw = (first_drawn_line.0, loc);
        let mut style = Style::default();
        let mut next_style = 0;
        let mut next_break = first_drawn_line.1;
        for i in 0..self.styles[cur_draw.0].len() {
            if cur_draw.1 < self.styles[cur_draw.0][i].0 {
                if i > 0 {
                    next_style = i-1;
                    style = self.styles[cur_draw.0][i-1].1;
                }
                break;
            }
        }
        while cur_draw.0 <= self.display_to.0 && cur_draw != (self.display_to.0, self.display_to.1) {
            let (line, i) = cur_draw;
            if next_style < self.styles[line].len() && self.styles[line][next_style].0 == i {
                style = self.styles[line][next_style].1;
                next_style += 1;
            }
            if next_break < self.breaks[line].len() && self.breaks[line][next_break] == i {
                buffer.next_line();
                next_break += 1;
            }
            match self.lines[line][i] {
                '\n' => (),
                '\t' => {
                    for _ in 0..4 {
                        buffer.wchar(' ', style)
                    }
                },
                _ => buffer.wchar(self.lines[line][i], style)
            };
            cur_draw.1 += 1;
            if cur_draw.1 >= self.lines[line].len() {
                buffer.next_line();
                next_break = 0;
                next_style = 0;
                style = Style::default();
                cur_draw.0 += 1;
                cur_draw.1 = 0;
            }
        }
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::Nothing
    }

    fn update_size(&mut self, bound: super::WidgetBound, buffer: &mut WidgetBuffer) -> WidgetBound {
        buffer.resize(bound.into());
        self.width = bound.width as usize;
        for i in 0..self.breaks.len() {
            self.breaks[i].clear();
            line_breaks(&mut self.breaks[i], &self.lines[i], self.width);
        }
        bound
    }

    fn animates(&self) -> bool {true}

    fn next_frame(&mut self, buffer: &mut WidgetBuffer) -> bool {
        if !self.done() {
            if self.display_to.1 >= self.lines[self.display_to.0].len() && self.display_to.0 < self.lines.len()-1 {
                self.display_to = (self.display_to.0+1, 0);
            } else {
                self.display_to.1 += 1;
            }
            self.draw(&mut [], buffer);
        }
        return !self.done();
    }
}
