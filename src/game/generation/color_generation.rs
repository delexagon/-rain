use crate::common::{Fg, PersistentHash, Rgb, TileStyle, Hsv};
use super::Generator;
use serde::{Serialize, Deserialize};

const MASK0: usize = (1<<8)-1;
const MASK_16: usize = (1<<16)-1;
const MASK1: usize = (1<<16)-(1<<8);
const MASK2: usize = (1<<24)-(1<<16);
const MASK3: usize = (1<<32)-(1<<24);

pub trait Color {
    fn to_rgb(&self, x: usize) -> Rgb;
}

impl Color for Rgb {
    fn to_rgb(&self, _x: usize) -> Rgb {
        return *self;
    }
}
impl Color for Hsv {
    fn to_rgb(&self, _x: usize) -> Rgb {
        return Rgb::from(*self);
    }
}

#[derive(Serialize,Copy,Clone)]
#[serde(tag="type",content="data")]
pub enum ColorType {
    Rgb(Rgb),
    RgbBound(RgbBound),
    Hsv(Hsv),
    HsvBound(HsvBound),
} impl ColorType {
    fn instantiate(&self) -> Self {
        match self {
            Self::Rgb(_) | Self::Hsv(_) => self.clone(),
            Self::RgbBound(x) => Self::RgbBound(x.instantiate()),
            Self::HsvBound(x) => Self::HsvBound(x.instantiate()),
        }
    }
}
impl Color for ColorType {
    fn to_rgb(&self, i: usize) -> Rgb {
        match self {
            Self::Rgb(x) => x.to_rgb(i),
            Self::RgbBound(x) => x.to_rgb(i),
            Self::Hsv(x) => x.to_rgb(i),
            Self::HsvBound(x) => x.to_rgb(i),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag="type",content="data")]
pub enum ColorType2 {
    Rgb(Rgb),
    RgbBound(RgbBound),
    Hsv(Hsv),
    HsvBound(HsvBound),
}
#[derive(Deserialize)]
#[serde(untagged)]
enum MaybeAString {
    ColorType(ColorType2), String(String)
}
use serde::Deserializer;
impl<'de> Deserialize<'de> for ColorType {
    fn deserialize<D>(deserializer: D) -> Result<ColorType, D::Error>
    where D: Deserializer<'de> {
        let result = MaybeAString::deserialize(deserializer)?;
        match result {
            MaybeAString::ColorType(t) => Ok(match t {
                ColorType2::Rgb(x) => ColorType::Rgb(x),
                ColorType2::RgbBound(x) => ColorType::RgbBound(x),
                ColorType2::Hsv(x) => ColorType::Hsv(x),
                ColorType2::HsvBound(x) => ColorType::HsvBound(x),
            }),
            MaybeAString::String(string) => match super::COLORS.get().unwrap().get(&string) {
                None => Err(serde::de::Error::custom(format!("key {string} not in hashmap"))),
                Some(v) => Ok(v.instantiate())
            }
        }
    }
}

#[derive(Serialize,Deserialize,Copy,Clone)]
pub struct RgbBound {
    lower: Rgb,
    upper: Rgb,
    #[serde(default)]
    hash: PersistentHash
} impl Color for RgbBound {
    fn to_rgb(&self, i: usize) -> Rgb {
        let random = self.hash.hash(i);
        return Rgb(
            ( random&MASK0     ) as u8%(self.upper.0-self.lower.0)+self.lower.0,
            ((random&MASK1)>>8 ) as u8%(self.upper.1-self.lower.1)+self.lower.1,
            ((random&MASK2)>>16) as u8%(self.upper.2-self.lower.2)+self.lower.2
        );
    }
}
impl RgbBound {
    pub fn new(lower: Rgb, upper: Rgb) -> Self {
        return Self {
            lower: lower,
            upper: upper,
            hash: PersistentHash::new()
        }
    }

    pub fn instantiate(&self) -> Self {
        return Self {
            lower: self.lower,
            upper: self.upper,
            hash: PersistentHash::new()
        }
    }
}

#[derive(Serialize,Deserialize,Copy,Clone)]
pub struct HsvBound {
    lower: Hsv,
    upper: Hsv,
    #[serde(default)]
    hash: PersistentHash
} impl Color for HsvBound {
    fn to_rgb(&self, i: usize) -> Rgb {
        let random = self.hash.hash(i);
        return Hsv(
            ( random&MASK_16   ) as u16%(self.upper.0-self.lower.0)+self.lower.0,
            ((random&MASK2)>>16) as u8 %(self.upper.1-self.lower.1)+self.lower.1,
            ((random&MASK3)>>24) as u8 %(self.upper.2-self.lower.2)+self.lower.2
        ).into();
    }
}
impl HsvBound {
    pub fn new(lower: Hsv, upper: Hsv) -> Self {
        return Self {
            lower: lower,
            upper: upper,
            hash: PersistentHash::new()
        }
    }

    pub fn instantiate(&self) -> Self {
        return Self {
            lower: self.lower,
            upper: self.upper,
            hash: PersistentHash::new()
        }
    }
}

pub trait Char {
    fn to_char(&self, x: usize) -> (char, bool, bool);
}
impl Char for char {
    fn to_char(&self, _x: usize) -> (char, bool, bool) {(*self, false, false)}
}

#[derive(Serialize, Deserialize,Copy,Clone)]
#[serde(tag="type",content="data")]
pub enum CharType {
    Single(char),
} impl Char for CharType {
    fn to_char(&self, i: usize) -> (char,bool,bool) {
        match self {
            Self::Single(x) => x.to_char(i),
        }
    }
}

#[derive(Serialize,Deserialize,Copy,Clone)]
struct DynFg<Ch: Char, Col: Color> {
    ch: Ch,
    color: Col
}

#[derive(Serialize,Deserialize,Copy,Clone)]
pub struct Tile<Ch: Char, Col: Color> {
    fg: Option<DynFg<Ch, Col>>,
    bg: Option<Col>
} impl<Ch: Char, Col: Color> Tile<Ch,Col> {
    pub fn gen(&self, i: usize) -> TileStyle {
        return TileStyle {
            fg: match &self.fg {
                Some(fg) => {
                    let (ch, bold, ital) = fg.ch.to_char(i);
                    let color = fg.color.to_rgb(i);
                    Some(Fg {
                        ch: ch,
                        bold: bold,
                        ital: ital,
                        color: color
                    })
                }, None=>None
            },
            bg: match &self.bg {
                Some(bg) => Some(bg.to_rgb(i)), None=>None,
            }
        }
    }
}

pub type DynTile = Tile<CharType, ColorType>;
impl Default for DynTile {
    fn default() -> Self { Self {fg: None, bg: None} }
}
