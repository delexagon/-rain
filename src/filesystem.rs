mod options;
mod debug_stream;
mod json;
mod paths;

pub use json::*;
pub use options::{Options2, Options};
pub use debug_stream::DebugStream;
pub use paths::Paths;

pub fn get_resources() -> Result<(Paths, Options, DebugStream), String> {
    let paths = Paths::new()?;
    let options = Options::new(&paths)?;
    let debug = DebugStream::new(&paths)?;
    return Ok((paths, options, debug));
}
