
pub type Rgb = (u8,u8,u8);

#[derive(Clone,Copy)]
pub struct Style {
    pub fg: Rgb,
    pub bg: Rgb,
    pub bold: bool,
    pub ital: bool,
}

#[derive(Copy, Clone)]
pub struct UITile {
    pub ch: char,
    pub sty: Style,
}

pub const NORMALSTYLE: Style = Style {
    fg: (255,255,255),
    bg: (0,0,0),
    bold: false,
    ital: false,
};

pub const BLANKTILE: UITile = UITile {
    ch: ' ',
    sty: NORMALSTYLE,
};

pub const FILLEDTILE: UITile = UITile {
    ch: '-',
    sty: NORMALSTYLE,
};

pub const REDSTYLE: Style = Style {
    fg: (255,0,0),
    bg: (0,0,0),
    bold: false,
    ital: false,
};

#[derive(Clone, Copy)]
pub struct TileStyle {
    // TODO: This should be a unicode string, not a char
    pub ch: Option<char>,
    pub sty: Style,
}

impl TileStyle {
    pub fn mod_style(&mut self, other: TileStyle) {
        match other.ch {
            Some(ch) => self.ch = Some(ch),
            None => (),
        }
        self.sty = other.sty;
    }
    
    pub fn extract(&self) -> UITile {
        match self.ch {
            Some(ch) => return UITile {ch: ch, sty: self.sty},
            None => return UITile {ch: ' ', sty: self.sty},
        }
    }
}
