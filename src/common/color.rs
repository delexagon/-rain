
use serde::{Serialize, Deserialize};

#[derive(PartialEq,Eq,Copy,Clone,Serialize,Deserialize,Debug)]
pub struct Rgb(pub u8,pub u8,pub u8);
impl From<Rgb> for crossterm::style::Color {
    fn from(i: Rgb) -> Self {
        Self::Rgb {r: i.0, g: i.1, b: i.2}
    }
}

#[derive(PartialEq,Eq,Copy,Clone,Serialize,Deserialize,Debug)]
pub struct Hsv(pub u16, pub u8, pub u8);
impl From<Hsv> for Rgb {
    fn from(hsv: Hsv) -> Self {
        let h = hsv.0 as f64/360.;
        let s = hsv.1 as f64/100.;
        let v = hsv.2 as f64/100.;
        if hsv.1 == 0 {
            let v = hsv.1*255/100;
            return Rgb(v,v,v);
        }
        let i = (h*6.).floor();
        let f = h*6.-i;
        let p = v*(1.-s);
        let q = v*(1.-s*f);
        let t = v*(1.-s+s*f);
        let i = i as u8;
        let v = (v*255.) as u8;
        let p = (p*255.) as u8;
        let t = (t*255.) as u8;
        let q = (q*255.) as u8;
        match i {
            0 => Rgb(v,t,p),
            1 => Rgb(q,v,p),
            2 => Rgb(p,v,t),
            3 => Rgb(p,q,v),
            4 => Rgb(t,p,v),
            5 => Rgb(v,p,q),
            _ => Rgb(0,0,0)
        }
    }
}

pub const BLACK: Rgb = Rgb(0,0,0);
pub const RED: Rgb = Rgb(255,0,0);
pub const WHITE: Rgb = Rgb(255,255,255);

#[derive(PartialEq,Eq,Clone,Copy,Serialize,Deserialize)]
pub struct Style {
    pub fg: Option<Rgb>,
    pub bg: Option<Rgb>,
    pub bold: bool,
    pub ital: bool,
    pub reverse: bool
}
impl Style {
    pub fn from_fg(fg: Rgb) -> Self {
        Self {
            bold: false,
            ital: false,
            reverse: false,
            fg: Some(fg),
            bg: None
        }
    }
    pub fn bold(&self) -> Self {
        let mut s = *self;
        s.bold = !s.bold;
        return s;
    }
    pub fn reverse(&self) -> Self {
        let mut s = *self;
        s.reverse = !s.reverse;
        return s;
    }
}

impl Default for Style {
    fn default() -> Self { 
        Self {
            fg: None,
            bg: None,
            bold: false,
            ital: false,
            reverse: false
        }
    }
}

#[derive(Clone,Copy,Serialize,Deserialize)]
pub struct Fg {
    pub ch: char,
    pub bold: bool,
    pub ital: bool,
    pub color: Rgb
}

pub type Bg = Rgb;

#[derive(Copy, Clone,Serialize,Deserialize)]
pub struct UITile {
    pub fg: Fg,
    pub bg: Bg,
}

pub const BLANKTILE: UITile = UITile {
    fg: Fg {
        ch: ' ',
        bold: false,
        ital: false,
        color: WHITE
    },
    bg: BLACK
};

pub const FILLEDTILE: UITile = UITile {
    fg: Fg {
        ch: '-',
        bold: false,
        ital: false,
        color: WHITE
    },
    bg: BLACK
};

#[derive(Clone, Copy,Serialize,Deserialize)]
pub struct TileStyle {
    // TODO: This should be a unicode string, not a char
    pub fg: Option<Fg>,
    pub bg: Option<Bg>
}

impl TileStyle {
    pub fn mod_style(&mut self, other: TileStyle) {
        self.fg = self.fg.or(other.fg);
        self.bg = self.bg.or(other.bg);
    }
    
    pub fn extract(&self) -> UITile {
        UITile {
            fg: self.fg.unwrap_or(Fg {
                ch: ' ',
                bold: false,
                ital: false,
                color: WHITE
            }),
            bg: self.bg.unwrap_or(BLACK)
        }
    }
}

pub const NONETILE: TileStyle = TileStyle {fg: None, bg: None};
