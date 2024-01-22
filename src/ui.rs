mod widgets;
mod terminal_base;
mod ui;

pub use ui::{UI, Widget, Action, Key, NoAction};
use ui::DrawBound;
use terminal_base::Terminal;
pub use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
pub use widgets::*;
