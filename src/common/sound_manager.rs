use crate::common::ResourceHandler;
use crate::{false_if_err,err};
use std::collections::HashMap;
use std::time::Duration;

pub fn transition_default() -> Tween {
    Tween::default()
}

pub fn transition_length(millis: u64) -> Tween {
    let mut default = Tween::default();
    default.duration = Duration::from_millis(millis);
    return default;
}

use kira::{
	manager::{backend::DefaultBackend, AudioManager, AudioManagerSettings},
	sound::static_sound::{StaticSoundHandle, StaticSoundData, StaticSoundSettings},
    sound::streaming::{StreamingSoundHandle, StreamingSoundData, StreamingSoundSettings},
    sound::FromFileError,
    tween::{Value, Tween},
    Volume,
};

enum SoundHandle {
    Static(StaticSoundHandle),
    Streaming(StreamingSoundHandle<FromFileError>)
}
impl SoundHandle {
    fn stop(&mut self, tween: Tween) -> Result<(), kira::CommandError> {
        match self {
            Self::Static(handle) => handle.stop(tween),
            Self::Streaming(handle) => handle.stop(tween),
        }
    }
    pub fn set_volume<T>(&mut self, volume: T, tween: Tween) -> Result<(), kira::CommandError> where T: Into<Value<Volume>> {
        match self {
            Self::Static(handle) => handle.set_volume(volume, tween),
            Self::Streaming(handle) => handle.set_volume(volume, tween),
        }
    }
}

pub struct SoundManager {
    manager: AudioManager,
    static_sounds: HashMap<String, StaticSoundData>,
    background: Option<SoundHandle>
}

impl SoundManager {
    pub fn new(mut preloaded: Vec<String>, resources: &mut ResourceHandler) -> Self {
        let mut static_sounds = HashMap::new();
        for name in preloaded.drain(..) {
            match resources.eat(err!(StaticSoundData::from_file(
                resources.sound_file(&name),
                StaticSoundSettings::default().volume(0.)
            ))) {
                Some(audio) => {static_sounds.insert(name, audio);},
                None => (),
            };
        }
        Self {
            manager: resources.early_choke(err!(AudioManager::<DefaultBackend>::new(AudioManagerSettings::default()))),
            static_sounds,
            background: None
        }
    }

    pub fn play(&mut self, which: &str, resources: &mut ResourceHandler) -> bool {
        match self.static_sounds.get(which) {
            Some(audio) => {
                false_if_err!(self.manager.play(audio.clone()), resources);
            },
            None => {
                let stream = false_if_err!(StreamingSoundData::from_file(
                    resources.sound_file(which),
                    StreamingSoundSettings::default().volume(0.)
                ), resources);
                false_if_err!(self.manager.play(stream), resources);
            }
        }
        return true;
    }

    pub fn background(&mut self, which: &str, resources: &mut ResourceHandler) -> bool {
        if self.background.is_some() {
            self.stop_background(resources);
        }
        match self.static_sounds.get(which) {
            Some(audio) => {
                let mut sound = false_if_err!(self.manager.play(audio.clone()), resources);
                false_if_err!(sound.set_volume(resources.options.volume as f64 / 255., Tween::default()), resources);
                false_if_err!(sound.set_loop_region(..), resources);
                self.background = Some(SoundHandle::Static(sound));
            },
            None => {
                let stream = false_if_err!(StreamingSoundData::from_file(
                    resources.sound_file(which),
                    StreamingSoundSettings::default().volume(0.)
                ), resources);
                let mut sound = false_if_err!(self.manager.play(stream), resources);
                false_if_err!(sound.set_volume(resources.options.volume as f64 / 255., Tween::default()), resources);
                false_if_err!(sound.set_loop_region(..), resources);
                self.background = Some(SoundHandle::Streaming(sound));
            }
        }
        return true;
    }

    pub fn set_background_volume(&mut self, volume: u8, resources: &mut ResourceHandler, tween: Tween) -> bool {
        if let Some(sound) = &mut self.background {
            false_if_err!(sound.set_volume(volume as f64 / 255., tween), resources);
        }
        return false;
    }

    pub fn stop_background(&mut self, resources: &mut ResourceHandler) -> bool {
        match &mut self.background {
            Some(sound) => {
                false_if_err!(sound.stop(Tween::default()), resources);
                self.background = None;
            },
            None => ()
        }
        return true;
    }
}
