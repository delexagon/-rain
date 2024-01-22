use crate::common::{UITile, Style, Rgb};

use std::io;
use std::io::Write;
use std::time::Duration;
use std::fmt::Display;

use crossterm::event::{
    poll, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::{
    cursor::{position, MoveTo, Hide, Show},
    event::{
        read, DisableBracketedPaste, DisableFocusChange, DisableMouseCapture, EnableBracketedPaste,
        EnableFocusChange, EnableMouseCapture, Event, KeyCode,
    },
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, size},
    style,
};


pub struct Terminal {
    out: io::Stdout,
    supports_keyboard_enhancement: bool,
}

impl Terminal {
    pub fn start() -> io::Result<Terminal> {
        enable_raw_mode()?;

        let mut stdout = io::stdout();

        let supports_keyboard_enhancement = matches!(
            crossterm::terminal::supports_keyboard_enhancement(),
            Ok(true)
        );

        if supports_keyboard_enhancement {
            queue!(
                stdout,
                PushKeyboardEnhancementFlags(
                    KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                        | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                        | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
                )
            )?;
        }

        execute!(
            stdout,
            EnableBracketedPaste,
            Hide,
            EnableFocusChange,
            EnableMouseCapture,
            EnterAlternateScreen,
            MoveTo(0, 0),
        )?;
        
        let (width, height) = size()?;
        
        return io::Result::Ok(Terminal {
            out: stdout,
            supports_keyboard_enhancement: supports_keyboard_enhancement,
        });
    }
    
    pub fn size(&self) -> io::Result<(u16,u16)> {
        return size();
    }
    
    pub fn get_event(&self) -> io::Result<Event> {
        let event = read()?;
        return Ok(event);
    }
    
    pub fn write<T: Display + ?Sized>(&mut self, thing: &T) -> io::Result<()> {
        queue!(
            self.out,
            style::Print(thing),
        )
    }
    
    pub fn clear_style(&mut self) -> io::Result<()> {
        queue!(self.out, style::SetAttribute(style::Attribute::Reset))
    }
    
    pub fn reverse(&mut self) -> io::Result<()> {
        queue!(self.out, style::SetAttribute(style::Attribute::Reverse))
    }
    
    pub fn stop_reverse(&mut self) -> io::Result<()> {
        queue!(self.out, style::SetAttribute(style::Attribute::NoReverse))
    }
    
    pub fn set_fg(&mut self, color: Rgb) -> io::Result<()> {
        queue!(
            self.out,
            style::SetForegroundColor(style::Color::from(color)),
        )
    }
    
    pub fn set_style(&mut self, style: Style) -> io::Result<()> {
        queue!(self.out, style::SetAttribute(style::Attribute::Reset))?;
        if style.ital {
            queue!(self.out, style::SetAttribute(style::Attribute::Italic))?;
        }
        if style.bold {
            queue!(self.out, style::SetAttribute(style::Attribute::Bold))?;
        }
        queue!(
            self.out,
            style::SetForegroundColor(style::Color::from(style.fg)),
            style::SetBackgroundColor(style::Color::from(style.bg)),
        )
    }
    
    pub fn write_uitile(&mut self, tile: &UITile) {
        self.set_style(tile.sty);
        self.write::<char>(&tile.ch);
    }
    
    pub fn move_to(&mut self, col: u16, row: u16) -> io::Result<()> {
        queue!(self.out, MoveTo(col, row))
    }
    
    pub fn finish(&mut self) -> io::Result<()> {
        self.out.flush()
    }
    
    pub fn stop(&mut self) -> io::Result<()> {
        if self.supports_keyboard_enhancement {
            queue!(self.out, PopKeyboardEnhancementFlags)?;
        }

        execute!(
            self.out,
            LeaveAlternateScreen,
            Show,
            DisableBracketedPaste,
            PopKeyboardEnhancementFlags,
            DisableFocusChange,
            DisableMouseCapture
        )?;

        disable_raw_mode()
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.stop().unwrap();
    }
}
