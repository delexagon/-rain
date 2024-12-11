use std::path::PathBuf;
use std::fs;
use crate::filesystem::{from_json, to_json, to_msgpack, from_msgpack, Paths, DebugStream, Options, Options2};
use crate::game::GenerationData;
use crate::ui::{UI, Candidate, RawKey};
use crate::err;
use serde::{Serialize,Deserialize};

// Paths will not change during gameplay.
pub struct ResourceHandler {
    pub options: Options,
    debug: DebugStream,
    pub path: Paths,
}

impl ResourceHandler {
    pub fn new(paths: Paths, options: Options, debug: DebugStream) -> Self {
        Self {
            path: paths,
            debug: debug,
            options: options,
        }
    }
    pub fn init(&mut self) {
        if !self.path.saves.exists() {
            self.early_choke(err!(fs::create_dir(&self.path.saves)));
        }
    }

    pub fn attach_to_game(&mut self, game_save_folder: &str) -> Result<(),std::io::Error> {
        let p = self.path.saves.join(game_save_folder);
        if !p.exists() {
            fs::create_dir(&p)?;
        }
        self.path.game_save = p;

        Ok(())
    }

    pub fn static_sounds(&mut self) -> Vec<String> {
        return match from_json(&self.path.static_sounds.clone(), self) {
            Some(x) => x,
            None => Vec::with_capacity(0),
        };
    }

    fn flush_errors(&mut self) {
        self.debug.flush();
    }

    pub fn early_choke<T>(&mut self, res: Result<T, String>) -> T {
        match res {
            Ok(t) => return t,
            Err(e) => {
                self.debug.write(&e);
                panic!(crate::translate!(early_choke_fail));
            }
        };
    }

    pub fn choke<T>(&mut self, res: Result<T, String>, ui: &mut UI) -> T {
        match res {
            Ok(t) => return t,
            Err(e) => {
                self.debug.write(&e);
                ui.stop(self);
                self.flush_errors();
                panic!(crate::translate!(choke_fail), e);
            }
        };
    }

    pub fn err(&mut self, err: &str) {
        self.debug.write(err);
    }

    // For errors that should not stop the program when they occur
    pub fn eat<T>(&mut self, res: Result<T, String>) -> Option<T> {
        match res {
            Ok(t) => return Some(t),
            Err(e) => {
                self.debug.write(&e);
                return None;
            }
        };
    }
    
    pub fn list_saves(&self) -> Result<Vec<String>, std::io::Error> {
        let paths = fs::read_dir(&self.path.saves)?;
        let mut vec = Vec::new();
        for path in paths {
            // Haha this shit can go fuck itself.
            match path {
                Ok(path2) => {
                    match path2.path().file_name() {
                        Some(path3) => {
                            match path3.to_str() {
                                Some(path4) => vec.push(path4.to_owned()),
                                None => ()
                            }
                        },
                        None => ()
                    }
                }, Err(_) => (),
            }
        }
        return Ok(vec);
    }

    pub fn save_options(&mut self) -> bool {
        let options: Options2 = (&self.options).into();
        let path = self.path.options.clone();
        return to_json(&options, &path, self)
    }

    pub fn sound_file(&self, file: &str) -> PathBuf {
        return self.path.sounds.join(file.to_owned()+".mp3");
    }
    
    pub fn save<T: Serialize>(&mut self, file_name: &str, thing: &T) -> bool {
        let path = self.path.game_save.join(file_name);
        return to_msgpack(thing, &path, self);
    }
    pub fn load<T: for <'a>  Deserialize<'a>>(&mut self, file_name: &str) -> Option<T> {
        let path = self.path.game_save.join(file_name);
        return from_msgpack(&path, self);
    }

    pub fn file_str(&mut self, file: &PathBuf) -> Option<String> {
        self.eat(err!(fs::read_to_string(file)))
    }

    pub fn map_key(&self, k: &RawKey) -> Option<Candidate> {self.options.keys.get(k).cloned()}
}
