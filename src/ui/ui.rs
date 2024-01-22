use crate::common::DataBox;
use crate::ui::Terminal;
use std::any::Any;
use std::io::Write;
use std::fs::File;
use crossterm::event::{Event, KeyEvent, KeyCode, KeyModifiers};

#[derive(Eq, PartialEq, Clone, Copy, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers
}

pub trait Action: 'static {
    fn any(&self) -> &dyn Any;
    fn bad_clone(&self) -> Box<Self> where Self: Sized + Clone {
        Box::new(self.clone())
    }
    fn awful_clone(&self) -> Box<dyn Action>;
}

#[macro_export]
macro_rules! to_action {
    // Dumb
    ($x:ident) => {
        impl Action for $x {
            fn any(&self) -> &dyn Any {self}
            fn awful_clone(&self) -> Box<dyn Action> {Box::new(self.clone())}
        }
    };
}

#[derive(Clone)]
pub enum NoAction {}
to_action!(NoAction);

impl DataBox<UI> {
    pub fn debug(&self, thing: &str) {
        self.write().error_output.write(thing.as_bytes()).expect("Debug debug");
    }
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub struct DrawBound {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl DrawBound {
    pub fn end_x(&self) -> u16 { self.x + self.width }
    pub fn end_y(&self) -> u16 { self.y + self.height }
}

// A simple UI system.
// It was very, very hard to finagle Rust to do what I wanted it to, so this is weird,
// but hopefully it can work albiet slowly.
// I considered using ECS, but that would almost certainly be even worse than this.
pub struct UI {
    pub term: Terminal,
    pub error_output: File,
    self_reference: Option<DataBox<UI>>,
    root: usize,
    // Should all be databoxed widgets, thank you.
    // TODO: Find a way to remove the Box, all DataBoxes are the same size.
    widgets: Vec<Box<dyn WidgetIntermediate>>,
    width: u16,
    height: u16,
}

impl UI {
    pub fn new() -> DataBox<Self> {
        let ui = DataBox::new(Self {
            self_reference: None,
            root: 0,
            widgets: Vec::new(),
            width: 0,
            height: 0,
            error_output: File::create("ui_error.txt").unwrap(),
            term: Terminal::start().unwrap(),
        });
        let (width, height) = ui.read().term.size().unwrap();
        {
            let mut i = ui.write();
            i.self_reference = Some(ui.clone());
            i.height = height;
            i.width = width;
        }
        return ui.clone();
    }
    
    pub fn swap_root(&mut self, new_root: usize) {
        self.root = new_root;
    }
    
    pub fn goto(&mut self, col: u16, row: u16) {
        self.term.move_to(col, row);
    }
    
    pub fn set_fg(&mut self, r: u8, g: u8, b: u8) {
        self.term.set_fg((r,g,b));
    }
    
    fn next_event(&self) -> Event {
        return self.term.get_event().unwrap();
    }
    
    pub fn add_widget(&mut self, widget: Box<dyn WidgetIntermediate>) -> usize {
        self.widgets.push(widget);
        return self.widgets.len()-1;
    }

    pub fn draw(me: DataBox<Self>) {
        let bound;
        let root_widget;
        {
            let inside = me.read();
            bound = DrawBound { x: 0, y: 0, width: inside.width, height: inside.height };
            root_widget = inside.widgets[inside.root].bad_clone();
        }
        root_widget.draw(me.clone(), bound, false);
        me.write().term.finish();
    }
    
    pub fn force_draw(me: DataBox<Self>) {
        let bound;
        let root_widget;
        {
            let mut inside = me.write();
            let (width, height) = inside.term.size().unwrap();
            inside.height = height;
            inside.width = width;
            bound = DrawBound { x: 0, y: 0, width: inside.width, height: inside.height };
            root_widget = inside.widgets[inside.root].bad_clone();
        }
        root_widget.draw(me.clone(), bound, true);
        me.write().term.finish();
    }
    
    pub fn draw_one(me: DataBox<Self>, widget_id: usize, bound: DrawBound, force: bool) -> bool {
        let widget;
        {
            let inside = me.read();
            widget = inside.widgets[widget_id].bad_clone();
        }
        return widget.draw(me.clone(), bound, force);
    }
    
    // Event -> Action logic:
    // Events come in, and get consumed by the UI and any widgets they pass through.
    // When an event is consumed, consume_action returns None.
    // This get_action will continue to run until an event is not consumed, and the non-consuming
    // widget produces Some(Action) instead.
    // When this occurs, this function returns Some(Action) to the external function.
    // Borrowing items is generally deferred as much as possible, because this locks the object in place.
    // Rc<RefCell< boxes are used for copyable data which is mutably borrowable at any point along the
    // process.
    pub fn get_action(me: DataBox<Self>) -> Box<dyn Action> {
        loop {
            Self::draw(me.clone());
            // TODO: Put a loop in here to draw every 1/60 second or something until an action appears.
            let mut event;
            {
                let a = me.read();
                event = a.next_event();
            }
            let possible_action = Self::consume_action(me.clone(), event);
            match possible_action {
                Some(action) => return action,
                None => continue,
            }
        }
    }
    
    pub fn consume_action(me: DataBox<Self>, event: Event) -> Option<Box<dyn Action>> {
        match event {
            Event::Resize(x, y) => {
                let m = me.clone();
                // Need to make a write to persistently change our width/height
                {
                    let mut e = m.write();
                    e.width = x;
                    e.height = y;
                }
                Self::draw(me);
                return None;
            },
            _ => (),
        }
        let root_widget;
        {
            let inside = me.read();
            root_widget = inside.widgets[inside.root].bad_clone();
        }
        return root_widget.consume_action(me.clone(), event);
    }
    
    pub fn consume_action_one(me: DataBox<Self>, widget_id: usize, event: Event) -> Option<Box<dyn Action>> {
        let widget;
        {
            let inside = me.read();
            widget = inside.widgets[widget_id].bad_clone();
        }
        return widget.consume_action(me.clone(), event);
    }
    
    pub fn widget<T: 'static>(&self, widget: usize) -> Option<DataBox<T>> {
        let x = self.widgets[widget].any().downcast_ref::<DataBox<T>>();
        match x {
            Some(thing) => return Some(thing.clone()),
            None => return None,
        }
    }
    
    pub fn stop(&mut self) {
        self.term.stop();
    }
}

/*******************************************/

pub trait WidgetIntermediate {
    fn draw(&self, ui: DataBox<UI>, bound: DrawBound, force: bool) -> bool;
    fn consume_action(&self, ui: DataBox<UI>, event: Event) -> Option<Box<dyn Action>> ;
    fn bad_clone(&self) -> Box<dyn WidgetIntermediate>;
    fn any(&self) -> &dyn Any;
}

impl<T: Widget + 'static> WidgetIntermediate for DataBox<T> {
    fn draw(&self, ui: DataBox<UI>, bound: DrawBound, force: bool) -> bool {
        return T::draw(self.clone(), ui, bound, force);
    }
    fn consume_action(&self, ui: DataBox<UI>, event: Event) -> Option<Box<dyn Action>> {
        return T::consume_action(self.clone(), ui, event);
    }
    fn bad_clone(&self) -> Box<dyn WidgetIntermediate> {
        return Box::new(self.clone());
    }
    fn any(&self) -> &dyn Any {self}
}

/*******************************************/

pub trait Widget: Sized {
    fn new_unboxed() -> Self;
    
    fn boxed(me: Self) -> DataBox<Self> {
        return DataBox::new(me);
    }

    fn new() -> DataBox<Self> {
        return DataBox::new(Self::new_unboxed());
    }
    
    // There are no inherent guards against double borrows here;
    // make sure whenever you borrow one of these boxes you give it back
    // before sending them off to someone else!
    // Any read or write calls should be in their own scope.
    fn draw(me: DataBox<Self>, ui: DataBox<UI>, bound: DrawBound, force: bool) -> bool;
    fn consume_action(me: DataBox<Self>, ui: DataBox<UI>, event: Event) -> Option<Box<dyn Action>>;
}
