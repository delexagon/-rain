use super::Paths;
use std::fs::File;
use std::io::{BufWriter, Write, Error};
use crate::err;

pub struct DebugStream {
    // filesystem, ui, terminal, game, audio
    stream: BufWriter<File>,
}

impl DebugStream {
    pub fn new(paths: &Paths) -> Result<Self, String> {
        Ok(Self {stream: BufWriter::new(err!(File::create(&paths.error_log))?)})
    }
    
    fn try_write(&mut self, s: &str) -> Result<(), Error> {
        self.stream.write(s.as_bytes())?;
        self.stream.write(b"\n")?;
        Ok(())
    }

    // Levels:
    pub fn write(&mut self, s: &str) {
        let err = self.try_write(s);
        match err {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Perdón, ocurrió un error.");
                eprintln!("Primero, algo pasó: {},", s);
                eprintln!("Intenté escribirle esto al archivo de depuración, y eso falló también porque de {}.", e);
                eprintln!("Lo que sea, me rindo. Adiós!");
                panic!();
            }
        }
    }
    
    pub fn flush(&mut self) {
        let err = self.stream.flush();
        match err {
            Ok(_) => (),
            Err(e) => {
                eprintln!("Perdón, mientras intentaba escribir al archivo de errores, Rust me lanzó un error.");
                eprintln!("Especificamente, dijo {}.", e);
                eprintln!("No sé qué significa eso, pero quizá tú sí!");
                eprintln!("Pues nada, me rindo. Adiós!");
                panic!();
            }
        }
    }
}

