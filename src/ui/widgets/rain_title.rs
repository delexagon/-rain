use crate::common::{PersistentHash, Array2D, ResourceHandler};
use super::widget_package::*;

const RAND_BITS: usize = 16;
const DENSITY_STEP: usize = 250;
const MAX_DENSITY: usize = 25000;
const BELOW_STEP: usize = 4;

fn density_calc(timestep_level: usize) -> usize {
    let a = timestep_level/DENSITY_STEP;
    MAX_DENSITY.min(a*a)+1
}

// Special widgets to handle the title screen.
// I'm not really sure how to do this otherwise

#[derive(Deserialize,Serialize)]
pub struct TitleTop {
    title_image: Array2D<char>,
    title_alignment: (f64,f64),
    hash: PersistentHash,
    step: usize,
}

impl TitleTop {
    pub fn from_file(title_img: &str, title_alignment: (f64,f64), res: &mut ResourceHandler) -> Self {
        let title_image = match res.file_str(&res.path.misc.join(title_img)) {
            Some(x) => Array2D::from_str(&x),
            None => Array2D::new(),
        };
        Self {
            hash: PersistentHash::new(),
            step: BELOW_STEP,
            title_alignment,
            title_image
        }
    }
    
    pub fn hash(&self) -> PersistentHash {self.hash}
    pub fn step(&self) -> usize {self.step}

    const BACKGROUND: &'static [u8] = &crate::translate!(title_background);
    const STREAK_BITS: usize = 2;

    fn draw_background(&self, buffer: &mut WidgetBuffer) {
        // The bottom density level is self.step, and then it goes upwards from there.
        // We need to write upwards 
        let bound = buffer.bound();
        let max_height = bound.height as usize+BELOW_STEP;
        let mut y = max_height;
        let mut timestep_level = self.step-BELOW_STEP;
        while y > 0 {
            y -= 1;
            timestep_level += 1;
            let density_level = density_calc(timestep_level);
            for x in 0..bound.width as usize {
                // Providing a max width of u16 before repeating
                let hash_in = (timestep_level<<16) + x;
                let random = self.hash.hash(hash_in);
                let this_density = random&((1<<RAND_BITS)-1);
                if this_density < density_level {
                    let streak_length = ((random>>RAND_BITS)&((1<<Self::STREAK_BITS)-1))+1;
                    let min = if y<=streak_length {0} else {y-streak_length};
                    for dy in min..y {
                        if let Some(' ') = buffer.char((x,dy)) {
                            self.wbackground(x,dy,bound,buffer);
                        }
                    }
                }
            }
        }
    }

    fn wbackground(&self, x: usize, y: usize, bound: WidgetBound, buffer: &mut WidgetBuffer) {
        let exact = x*bound.height as usize+y;
        let (ch, offset) = (exact/8, (8-1)-(exact%8));
        let here = if ch >= Self::BACKGROUND.len() {'0'} else {('0' as u8 | ((Self::BACKGROUND[ch]>>offset)&1)) as char};
        buffer.wchar_at((x,y), here, Style::default());
    }
}

// TODO: Idk, just make this fucking work holy shit.
impl Widget for TitleTop {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let bound = buffer.bound();
        let img_start_x = inner_cut(bound.width, self.title_image.width() as u16, self.title_alignment.0) as usize;
        let img_start_y = inner_cut(bound.height, self.title_image.height() as u16, self.title_alignment.1) as usize;
        for y in 0..self.title_image.height() {
            for x in 0..self.title_image.width() {
                buffer.wchar_at((x+img_start_x,y+img_start_y), self.title_image[(x,y)], Style::default())
            }
        }

        self.draw_background(buffer);
    }

    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::Nothing
    }

    fn animates(&self) -> bool {true}

    fn next_frame(&mut self, buffer: &mut WidgetBuffer) -> bool {
        buffer.clear();

        self.step += 1;
        self.draw(&mut [], buffer);
        true
    }
}

#[derive(Deserialize,Serialize)]
pub struct TitleBottom {
    hash: PersistentHash,
    step: usize,

    lines: Vec<String>,
    selected: usize,
    interior: Array2D<u8>,
    background: Array2D<char>
}

impl TitleBottom {
    pub fn from(lines: Vec<String>, hash: PersistentHash, step: usize, bg_file: &str, res: &mut ResourceHandler) -> Self {
        let bg_image = match res.file_str(&res.path.misc.join(bg_file)) {
            Some(x) => Array2D::from_str(&x),
            None => Array2D::new(),
        };
        Self {
            hash,
            step,
            lines,
            selected: 0,
            interior: Array2D::new(),
            background: bg_image
        }
    }

    pub fn set(&mut self, line: usize, text: String) {
        self.lines[line] = text;
    }

    fn calc_cell(here: u8, left: u8, up: u8, right: u8, _down: u8)->u8 {
        // 0: NONE, 1: DOWN, 2: LEFT, 3: RIGHT, 4: STILL
        match (here,up,left,right) {
            (_,1,_,_) => 1,
            (4,_,_,_) => 0,
            (_,_,1,_) => 3,
            (_,_,_,1) => 2,
            (3,_,_,2) => 4,
            (2,_,3,_) => 4,
            (_,_,3,2) => 4,
            (_,_,3,_) => 3,
            (_,_,_,2) => 2,
            (_,_,_,_) => 0
        }
    }

    fn bg_y(&self, y: usize) -> Option<usize> {
        if self.background.height() > self.interior.height() {
            Some(self.background.height() - self.interior.height() + y)
        } else {
            if y >= self.interior.height() - self.background.height() {
                Some(y - (self.interior.height() - self.background.height()))
            } else {
                None
            }
        }
    }

    fn colors(&self, y: usize) -> [u8; 4] {[
        if y < self.interior.height()/2 {255/self.interior.height() as u8*2*(self.interior.height()/2-y) as u8} else {0},
        if y >= self.interior.height()/2 {255/self.interior.height() as u8*(self.interior.height()-y) as u8} else {255},
        125/self.interior.height() as u8*(self.interior.height()-y) as u8,
        255/self.interior.height() as u8*(self.interior.height()-y) as u8,
    ]}

    fn draw_background(&self, x: usize, y: usize, bg_y: Option<usize>, colors: [u8; 4], buffer: &mut WidgetBuffer) {
        match self.interior[(x,y)] {
            0 => {
                // This is broken, but it's funny so who cares.

                if bg_y.is_some() && buffer.char((x,y)) == Some(self.background[(x%self.background.width(), bg_y.unwrap())]) {
                    buffer.set_char((x,y), ' ');
                }
                buffer.style((x,y)).unwrap().bg = None
            },
            1 => {
                if let Some(bg_y) = bg_y {
                    if let Some(' ') = buffer.char((x,y)) {
                        buffer.set_char((x,y), self.background[(x%self.background.width(), bg_y)]);
                    }
                }
                buffer.style((x,y)).unwrap().bg = Some(Rgb(colors[0],colors[0],colors[1]));
            },
            _ => {
                if let Some(bg_y) = bg_y {
                    if let Some(' ') = buffer.char((x,y)) {
                        buffer.set_char((x,y), self.background[(x%self.background.width(), bg_y)]);
                    }
                }
                buffer.style((x,y)).unwrap().bg = Some(Rgb(0,colors[2],colors[3]));
            }
        };
    }
}

impl Widget for TitleBottom {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let bound = buffer.bound();
        let len = self.lines.len();
        let space = 1./((len-1) as f64);
        for i in 0..len {
            let alignment = space*i as f64;
            let string_len = self.lines[i].chars().count() as u16;
            let start = inner_cut(bound.width-4, string_len, alignment)+2;
            buffer.move_to(start, bound.height-2);
            if i == self.selected {
                let _ = buffer.wstr(&self.lines[i], Style::default().reverse());
            } else {
                let _ = buffer.wstr(&self.lines[i], Style::default());
            }
        }
        
        for y in 0..self.interior.height() {
            let bg_y = self.bg_y(y);
            let colors = self.colors(y);
            for x in 0..self.interior.width() {
                self.draw_background(x,y,bg_y,colors,buffer);
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

    fn update_size(&mut self, bound: WidgetBound, buffer: &mut WidgetBuffer) -> WidgetBound {
        buffer.resize(bound.into());
        buffer.clear();
        self.interior.resize_preserve((bound.width as usize, bound.height as usize), 0);
        return bound;
    }

    fn animates(&self) -> bool {true}

    fn next_frame(&mut self, buffer: &mut WidgetBuffer) -> bool {
        self.step += 1;
        let density_level = density_calc(self.step-1);
        for x in 0..self.interior.width() {
            // Providing a max width of u16 before repeating
            for y in 0..self.interior.height() {
                self.interior[(x,y)] += Self::calc_cell(
                    self.interior[(x,y)],
                    if x > 0 {self.interior[(x-1,y)]&((1<<4)-1)} else {0},
                    if y > 0 {self.interior[(x,y-1)]&((1<<4)-1)} else {0},
                    if x < self.interior.width()-1 {self.interior[(x+1,y)]} else {0},
                    if y < self.interior.height()-1 {self.interior[(x,y+1)]} else {0}
                )<<4;
            }
            let hash_in = ((self.step-1)<<16) + x;
            let random = self.hash.hash(hash_in);
            let this_density = random&((1<<RAND_BITS)-1);
            if this_density < density_level {
                self.interior[(x,0)] = 1<<4;
            }
        }
        for y in 0..self.interior.height() {
            let bg_y = self.bg_y(y);
            let colors = self.colors(y);
            for x in 0..self.interior.width() {
                self.interior[(x,y)] >>= 4;
                self.draw_background(x,y,bg_y,colors,buffer);
            }
        }
        true
    }
}