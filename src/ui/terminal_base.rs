use crate::common::{UIResources, Style};

use std::io;
use std::io::Write;
use std::time::Duration;
use crate::err;

use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::{
    cursor::{MoveTo, Hide, Show},
    event::{
        read, poll, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture, Event,
    },
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, size},
    style,
};

pub struct Terminal {
    out: io::Stdout,
    cur_style: Style,
    supports_keyboard_enhancement: bool,
}

impl Terminal {
    /// Enters an alternate terminal screen;
    /// chokes the program in a number of circumstances if there
    /// appears to be a problem manipulating the terminal.
    pub fn start(res: &mut UIResources) -> Terminal {
        let stdout = io::stdout();
        
        let supports_keyboard_enhancement = matches!(
            crossterm::terminal::supports_keyboard_enhancement(),
            Ok(true)
        );

        let mut t = Terminal {
            out: stdout,
            cur_style: Style { fg: None, bg: None, bold: false, ital: false, reverse: false },
            supports_keyboard_enhancement: supports_keyboard_enhancement,
        };
        
        res.early_choke(err!(enable_raw_mode()));

        if supports_keyboard_enhancement {
            let err = queue!(
                t.out,
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                )
            );
            res.early_choke(err!(err));
        }

        let err = execute!(
            t.out,
            EnableBracketedPaste,
            EnableFocusChange,
            EnableMouseCapture,
            EnterAlternateScreen,
            MoveTo(0, 0),
            style::SetAttribute(style::Attribute::Reset),
            Hide,
        );
        res.early_choke(err!(err));
        
        return t;
    }
    
    /// Returns (u16,u16) size of the terminal, representing (columns, rows)
    pub fn size(&self, res: &mut UIResources) -> Option<(u16,u16)> {
        return res.eat(err!(size()));
    }

    /// Hangs until some Event (as decreed by Crossterm) is received
    pub fn event_hang(&self, res: &mut UIResources) -> Option<Event> {
        return res.eat(err!(read()));
    }
    
    /// Returns an event for wait seconds, or None if there is none currently queued.
    pub fn event_hang_for(&self, wait: Duration, res: &mut UIResources) -> Option<Event> {
        if res.eat(err!(poll(wait)))? == true {
            return res.eat(err!(read()));
        }
        return None;
    }

    pub fn wchar(&mut self, thing: char, res: &mut UIResources) {
        res.eat(err!(queue!(self.out, style::Print(thing))));
    }
    
    pub fn set_style(&mut self, style: Style, res: &mut UIResources) {
        if style == self.cur_style {
            return;
        }
        res.eat(err!(queue!(self.out, style::SetAttribute(style::Attribute::Reset))));
        if style.ital {
            res.eat(err!(queue!(self.out, style::SetAttribute(style::Attribute::Italic))));
        }
        if style.bold {
            res.eat(err!(queue!(self.out, style::SetAttribute(style::Attribute::Bold))));
        }
        if style.reverse {
            res.eat(err!(queue!(self.out, style::SetAttribute(style::Attribute::Reverse))));
        }
        if let Some(fg) = style.fg {
            res.eat(err!(queue!(
                self.out,
                style::SetForegroundColor(fg.into()),
            )));
        }
        if let Some(bg) = style.bg {
            res.eat(err!(queue!(
                self.out,
                style::SetBackgroundColor(bg.into()),
            )));
        }
        self.cur_style = style;
    }

    pub fn move_to(&mut self, coord: (u16, u16), res: &mut UIResources) {
        res.eat(err!(queue!(self.out, MoveTo(coord.0, coord.1))));
    }
    
    pub fn hide_cursor(&mut self, res: &mut UIResources) {
        res.eat(err!(queue!(self.out, Hide)));
    }
    
    pub fn finish(&mut self, res: &mut UIResources) {
        res.eat(err!(self.out.flush()));
    }
    
    pub fn stop(&mut self, res: &mut UIResources) {
        if self.supports_keyboard_enhancement {
            res.eat(err!(queue!(self.out, PopKeyboardEnhancementFlags)));
        }

        res.eat(err!(execute!(
            self.out,
            LeaveAlternateScreen,
            DisableBracketedPaste,
            // Causes println spam
            PopKeyboardEnhancementFlags,
            DisableFocusChange,
            DisableMouseCapture,
            Show
        )));

        res.eat(err!(disable_raw_mode()));
    }
}
