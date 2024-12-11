use std::path::PathBuf;
use super::Paths;
use crate::ui::{RawKey, Candidate};
use std::collections::HashMap;
use crossterm::event::{KeyModifiers, KeyCode};
use lazy_static::lazy_static;
use serde::{Deserialize,Serialize};
use crate::{err, errstr};
use std::fs;

lazy_static!(
    static ref KEY_NAMES: HashMap<String, KeyCode> = HashMap::from([
        (crate::translate!("up").to_string(), KeyCode::Up),
        (crate::translate!("down").to_string(), KeyCode::Down),
        (crate::translate!("left").to_string(), KeyCode::Left),
        (crate::translate!("right").to_string(), KeyCode::Right),
        (crate::translate!("enter").to_string(), KeyCode::Enter),
        (crate::translate!("escape").to_string(), KeyCode::Esc),
        (crate::translate!("tab").to_string(), KeyCode::Tab),
    ]);

    static ref CANDIDATE_NAMES: HashMap<String, Candidate> = HashMap::from([
        (crate::translate!("up").to_string(), Candidate::Up),
        (crate::translate!("down").to_string(), Candidate::Down),
        (crate::translate!("left").to_string(), Candidate::Left),
        (crate::translate!("right").to_string(), Candidate::Right),
        (crate::translate!("enter").to_string(), Candidate::Enter),
        (crate::translate!("exit").to_string(), Candidate::Exit),
        (crate::translate!("wait").to_string(), Candidate::Wait),
        (crate::translate!("interact").to_string(), Candidate::Interact),
        (crate::translate!("get").to_string(), Candidate::Get),
        (crate::translate!("tab").to_string(), Candidate::Tab),
    ]);
);

pub type KeyMap = HashMap<RawKey, Candidate>;

fn get_key(key_str: &str) -> Result<RawKey, String> {
    let mut split = key_str.rsplit('+');
    let key_name = split.next();
    let key_code = match key_name {
        Some(key) => {
            match KEY_NAMES.get(key) {
                Some(keycode) => *keycode,
                None => {
                    let mut chars = key.chars();
                    if let (Some(ch), None) = (chars.next(), chars.next()) {
                        KeyCode::Char(ch)
                    } else {
                        return Err(errstr!(format!(crate::translate!(no_keycode_match), key_str)))
                    }
                }
            }
        },
        None => return Err(errstr!(format!(crate::translate!(empty_key_string), key_str))),
    };
    let mut modifiers = KeyModifiers::NONE;
    for modifier in split {
        let mut chars = modifier.chars();
        if let (Some(ch), None) = (chars.next(), chars.next()) {
            match ch {
                'c' => modifiers |= KeyModifiers::CONTROL,
                's' => modifiers |= KeyModifiers::SHIFT,
                'a' => modifiers |= KeyModifiers::ALT,
                _ => return Err(errstr!(format!(crate::translate!(unknown_modifier), key_str)))
            }
        } else {
            return Err(errstr!(format!(crate::translate!(one_character_modifiers), key_str)))
        }
    }
    return Ok(RawKey {
        modifiers,
        code: key_code
    })
}

fn get_keymap(path: &PathBuf) -> Result<KeyMap, String> { 
    let f = err!(fs::read_to_string(&path))?;
    let mut strings: HashMap<String, String> = err!(serde_json::from_str(&f))?;
    let mut fin: HashMap<RawKey, Candidate> = HashMap::new();
    for (candidate, key) in strings.drain() {
        match CANDIDATE_NAMES.get(&candidate) {
            Some(cand) => {
                let key = get_key(&key)?;
                fin.insert(key, *cand);
            },
            None => return Err(errstr!(format!(crate::translate!(unknown_candidate), candidate)))
        }
    }
    return Ok(fin);
}

#[derive(Serialize,Deserialize)]
pub struct Options2 {
    pub volume: u8,
    pub text_speed: u8,
}
impl From<&Options> for Options2 {
    fn from(item: &Options) -> Self {
        Self {
            volume: item.volume,
            text_speed: item.text_speed
        }
    }
}

#[derive(Clone)]
pub struct Options {
    pub keys: KeyMap,
    pub volume: u8,
    pub text_speed: u8,
}

impl Options {
    pub fn new(paths: &Paths) -> Result<Self, String> {
        let keymap = get_keymap(&paths.keymap);
        let f = err!(fs::read_to_string(&paths.options))?;
        let other: Options2 = err!(serde_json::from_str(&f))?;
        
        Ok(Options {
            keys: keymap?,
            volume: other.volume,
            text_speed: if other.text_speed > 3 {3} else {other.text_speed}
        })
    }
}
