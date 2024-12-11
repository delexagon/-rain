use crate::err;
use std::path::PathBuf;
use std::env::current_exe;
use std::fs::canonicalize;

pub struct Paths {
    pub data_dir: PathBuf,
    pub maps: PathBuf,
    pub colors: PathBuf,
    pub error_log: PathBuf,
    pub templates: PathBuf,
    pub saves: PathBuf,
    pub temp_save: PathBuf,
    pub sounds: PathBuf,
    pub static_sounds: PathBuf,
    pub misc: PathBuf,
    pub game_save: PathBuf,
    pub keymap: PathBuf,
    pub options: PathBuf,
}

impl Paths {
    pub fn new() -> Result<Self, String> {
        let mut base_dir = err!(current_exe())?;
        base_dir.pop();
        // Goes to main directory if this is in debug (via cargo run)
        if cfg!(debug_assertions) {
            base_dir.pop();
            base_dir.pop();
        }
        base_dir = err!(canonicalize(base_dir))?;
        let data_dir = base_dir.join(crate::translate!("data"));

        let options = data_dir.join(concat!(crate::translate!("options"),".json"));

        let error_log = data_dir.join(crate::translate!("error_log.txt"));

        let resources = data_dir.join(crate::translate!("resources"));
        let colors = resources.join(concat!(crate::translate!("colors"),".json"));
        let templates = resources.join(concat!(crate::translate!("templates"),".json"));
        let maps = resources.join(crate::translate!("maps"));
        let sounds = resources.join(crate::translate!("sounds"));
        let static_sounds = sounds.join(concat!(crate::translate!("static"),".txt"));
        let misc = resources.join(crate::translate!("miscellaneous"));

        let saves = data_dir.join(crate::translate!("saves"));
        let temp_save = saves.join(crate::translate!("temporary"));
        let game_save = saves.join(crate::translate!("none"));

        let keymap = data_dir.join(concat!(crate::translate!("keys"),".json"));

        return Ok(Self {
            data_dir,
            maps,
            colors,
            templates,
            saves,
            temp_save,
            sounds,
            static_sounds,
            misc,
            keymap,
            error_log,
            game_save,
            options,
        })
    }
}