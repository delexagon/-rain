mod lines; pub use lines::Lines;
mod lines_children; pub use lines_children::LinesChildren;
mod chararea; pub use chararea::CharArea;
mod window_manager; pub use window_manager::WindowManager;
mod split; pub use split::{Split, SplitType};
mod lines_horizontal; pub use lines_horizontal::LinesHorizontal;
mod progressbar; pub use progressbar::ProgressBar;
mod padding; pub use padding::Padding;
mod nothing; pub use nothing::Nothing;
mod line; pub use line::Line;
mod rain_title; pub use rain_title::{TitleTop, TitleBottom};
mod choice; pub use choice::Choice;
mod aligned; pub use aligned::Aligned;
mod sized; pub use sized::{Size, Sized};
mod ascii_image; pub use ascii_image::ASCIIImage;
mod line_scroll; pub use line_scroll::LineScroll;
mod lines_scroll; pub use lines_scroll::LinesScroll;
mod text_input; pub use text_input::TextInput;
mod border; pub use border::Border;
mod los_area; pub use los_area::LOSArea;
mod tabs; pub use tabs::Tabs;

mod widget_package {
    pub(super) use serde::{Deserialize,Serialize};
    pub(super) use crate::common::{Rgb, Id, Style, UIResources};
    pub(super) use super::{cut, inner_cut, next_word_break, line_breaks, Widget, AddChild, RemoveChild};
    pub(super) use super::super::ui::{UI, WidgetBound, EventResult, Match, Candidate, KeyEvent, KeyCode, WidgetData, KeyModifiers, Event, WidgetBuffer, PollResult, Poll};
    pub(super) use crossterm::event::{MouseButton, MouseEventKind};
}

macro_rules! arrange {
    ($($used:ident),*) => {
        #[enum_dispatch]
        #[derive(Serialize,Deserialize)]
        pub enum WidgetEnum {
            $($used),*
        }

        // Type -> variant
        pub trait IsWidget {
            fn is_widget(my_enum: &WidgetEnum) -> bool;
        }
        $(
        impl IsWidget for $used {
            fn is_widget(my_enum: &WidgetEnum) -> bool {
                match my_enum {
                    WidgetEnum:: $used(_) => true,
                    _ => false
                }
            }
        }
        impl<'a> TryFrom<&'a WidgetEnum> for &'a $used {
            type Error = ();

            fn try_from(value: &'a WidgetEnum) -> Result<Self, Self::Error> {
                match value {
                    WidgetEnum:: $used(widget) => {
                        Ok(widget)
                    },
                    _ => Err(())
                }
            }
        }
        impl<'a> TryFrom<&'a mut  WidgetEnum> for &'a mut $used {
            type Error = ();

            fn try_from(value: &'a mut WidgetEnum) -> Result<Self, Self::Error> {
                match value {
                    WidgetEnum:: $used(widget) => {
                        Ok(widget)
                    },
                    _ => Err(())
                }
            }
        }
        )*
    }
} arrange! {
    Lines, CharArea, WindowManager, Split, LinesHorizontal, ProgressBar, Sized,
    Padding, Nothing, TitleTop, TitleBottom, LinesChildren, Line, Choice, Aligned, ASCIIImage,
    LineScroll, TextInput, Border, LinesScroll, LOSArea, Tabs
}

use enum_dispatch::enum_dispatch;
use super::WidgetBuffer;
use super::{Poll, Candidate, PollResult, Match, Event, UI};
pub use super::WidgetBound;
use crate::common::Id;
use super::ui::{MouseEvent, EventResult, WidgetData, KeyCode, KeyEvent, KeyModifiers, MouseEventKind, MouseButton};
use crate::UIResources;
use serde::{Serialize,Deserialize};

pub trait AddChild {
    /// For widgets which can have children added to them
    /// i represents where the child is added
    /// return whether the operation is possible
    fn add_child(&mut self, i: usize) -> bool;
}
pub trait RemoveChild {
    fn remove_child(&mut self, i: usize) -> bool;
}

#[enum_dispatch(WidgetEnum)]
pub(super) trait Widget {
    /// Draws the content of this widget to a buffer.  
    /// This function may draw content from the buffers of other widgets;
    /// which is why it requires the UI.
    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer);
    /// Sends a poll request recursively down. If the event is ever intercepted,
    /// this function returns.  
    /// Events relevant to the poll should return Some(thing),
    /// events consumed for the widget's internals should return None.  
    /// It is common that you will want to have three levels of interception of a poll:
    /// - First, check for keys used internally by the widget
    /// - Second, check for keys present in the poll
    /// - Third, pass the event to the relevant child to poll from.  
    /// Returns whether a widget has been modified, and thus
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult;
    /// Used to REQUEST the widget to have a particular size.  
    /// Some widgets may ignore this request, in which case 
    fn update_size(&mut self, bound: WidgetBound, buffer: &mut WidgetBuffer) -> WidgetBound {
        buffer.resize(bound.into());
        buffer.clear();
        return bound;
    }
    /// The outside may potentially provide any number of widgets as children.  
    /// Please return whether this number of children is acceptable or not by returning a number this
    /// widget supports.  
    /// Widget
    fn child_number(&mut self, desired: usize) -> usize;
    /// Provide a vector of the sizes of the children of this widget, in order.
    /// Your child widgets will be fed this size, and may choose to respect it or not.
    /// Later, in the draw() function, you may use these prepared widgets 
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound>;
    /// Whether this widget could ever potentially animate;
    /// should only directly return true or not.
    fn animates(&self) -> bool {false}
    fn next_frame(&mut self, _buffer: &mut WidgetBuffer) -> bool {false}
}

// Helper functions

/// Assumes 0<=loc<=1  
/// Finds the location along the length
pub fn cut(length: u16, loc: f64) -> u16 {
    return ((length as f64)*loc).round() as u16;
}

/// Determines where an inner length needs to be start along an outer length
/// such that it gets placed at some percentage location.
/// Assumes: outer_len >= inner_len, 0<=loc<=1
pub fn inner_cut(outer_len: u16, inner_len: u16, loc: f64) -> u16 {
    if outer_len < inner_len {
        return 0;
    }
    let cut_outer = cut(outer_len, loc);
    let cut_inner = cut(inner_len, loc);
    return cut_outer-cut_inner;
}

fn next_word_break(chars: &[char], mut i: usize) -> usize {
    while i < chars.len() {
        match chars[i] {
            ' '|'\n'|'\t' => return i,
            _ => i+=1
        }
    }
    chars.len()
}

fn line_breaks(breaks: &mut Vec<usize>, chars: &[char], width: usize) {
    if chars.len() == 0 || width == 0 {
        return;
    }
    let mut along = 0;
    fn width_remaining(along: usize, width: usize) -> usize {if along>=width {0} else {width-along}}

    let mut word_start = 0;
    let mut word_break = next_word_break(chars, word_start);
    if word_break-word_start <= width && word_break-word_start > width_remaining(along, width) {
        breaks.push(word_start);
        along = word_break-word_start;
        word_start = word_break;
    } else if word_break-word_start > width {
        word_start += width-along;
        while word_start < word_break {
            breaks.push(word_start);
            word_start += width;
        }
        along = word_break-(word_start-width);
        word_start = word_break;
    } else {
        along += word_break-word_start;
        word_start = word_break;
    }
    while word_start < chars.len() {
        match chars[word_start] {
            '\n' => {breaks.push(word_start); along = 0;},
            '\t' => {
                along += 4;
            }
            ' ' => {
                along += 1;
            }
            _ => unreachable!()
        };
        word_start = word_break+1;
        word_break = next_word_break(chars, word_start);
        if word_break-word_start <= width && word_break-word_start > width_remaining(along, width) {
            breaks.push(word_start);
            along = word_break-word_start;
            word_start = word_break;
        } else if word_break-word_start > width {
            word_start += width_remaining(along, width);
            while word_start < word_break {
                breaks.push(word_start);
                word_start += width;
            }
            along = word_break-(word_start-width);
            word_start = word_break;
        } else {
            along += word_break-word_start;
            word_start = word_break;
        }
    }
}
