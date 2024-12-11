use crate::common::{UIResources, ExtTree, RecTree, Tree, UITile, Array2D, Style, ItrResult, Id, ReverseExtract, RemoveVec, TakeBox};
use crate::ui::Terminal;
use crate::Rgb;
use std::collections::HashSet;
pub use crossterm::event::{Event, KeyEvent, MouseEventKind, KeyEventKind, MouseButton, MouseEvent, KeyCode, KeyModifiers};
use serde::{Deserialize,Serialize};
use super::widgets::{AddChild, IsWidget, Padding, RemoveChild};
use super::{WidgetEnum, Widget};
use std::time::{SystemTime, Duration};

#[derive(Eq, PartialEq, Clone, Copy, Hash)]
pub struct RawKey {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Eq, PartialEq, Clone, Debug, Copy, Hash)]
pub enum Candidate {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Exit,
    Wait,
    Debug,
    Interact,
    Get,
    Tab,
    // These are not defined keystrokes,
    // but are instead requests for special data from the widget
    Select,
    LeftClick,
    RightClick,
    Character,
    FinishAnimation,
}

// Couples a key with a window id
pub type PollCandidate = (Id, Candidate);
pub type Poll = HashSet<PollCandidate>;

#[derive(PartialEq,Eq,Hash,Debug)]
pub enum Match {
    Standard(Candidate),
    Selection1D(u8),
    Selection2D(u8,u8),
    LeftClick(u16,u16),
    RightClick(u16,u16),
    Character(char),
    Paste(String),
    FinishAnimation,
}
pub type PollResult = (Id, Match);

pub(super) enum EventResult {
    PassToChild(i32),
    PollResult(PollResult),
    Changed,
    Nothing
}
enum EventResultInternal {
    Changed(Id),
    PollResult(PollResult),
    Nothing
}

fn match_poll(poll: &Poll, event: &Event, translation: Option<Candidate>, my_id: Id) -> Option<PollResult> {
    if let Some(candidate) = translation {
        if poll.contains(&(my_id, translation.unwrap())) {
            return Some((my_id, Match::Standard(candidate)));
        }
    }
    // Handling special matches
    match event {
        Event::Mouse(MouseEvent {kind: MouseEventKind::Down(MouseButton::Left), row, column, ..}) => {
            if poll.contains(&(my_id, Candidate::LeftClick)) {
                return Some((my_id, Match::LeftClick(*column, *row)));
            }
            None
        },
        Event::Mouse(MouseEvent {kind: MouseEventKind::Down(MouseButton::Right), row, column, ..}) => {
            if poll.contains(&(my_id, Candidate::RightClick)) {
                return Some((my_id, Match::RightClick(*column, *row)));
            }
            None
        },
        Event::Key(KeyEvent {code: KeyCode::Char(ch), modifiers, ..}) => {
            if poll.contains(&(my_id, Candidate::Character)) {
                if modifiers.contains(KeyModifiers::SHIFT) {
                    let mut uppercase = ch.to_uppercase();
                    let upper_ch = uppercase.next();
                    let upper_ch2 = uppercase.next();
                    return match (upper_ch, upper_ch2) {
                        (Some(newch), None) => Some((my_id, Match::Character(newch))),
                        _ => Some((my_id, Match::Character(*ch)))
                    }
                } else {
                    return Some((my_id, Match::Character(*ch)));
                }
            }
            None
        }
        _ => None,
    }
}

#[derive(Eq, Debug, PartialEq, Clone, Copy,Deserialize,Serialize)]
/// Widgets no longer have a starting position;
/// the top left corner of the widget is (0,0).
pub struct WidgetBound {
    pub width: u16,
    pub height: u16
} impl WidgetBound {
    fn zero(&self) -> bool {self.width==0||self.height==0}
}

#[derive(Serialize,Deserialize)]
pub struct WidgetData {
    id: Id,
    // The size this widget was last suggested
    last_bound: WidgetBound,
    
    // Remembering where this was drawn to the parent (to send mouse clicks)
    parent_position: (u16,u16),
    child_position: (i32,i32),
    drawn_as: WidgetBound,

    widget: WidgetEnum,
    buffer: WidgetBuffer
} impl WidgetData {
    pub(super) fn copy_to(&mut self, loc: ((u16,u16),(i32,i32)), draw_size: WidgetBound, other: &mut WidgetBuffer) {
        other.copy_from(loc.0, draw_size, loc.1, self.id, &self.buffer);
        self.drawn_as = draw_size;
        self.child_position = loc.1;
        self.parent_position = loc.0;
    }
    pub(super) fn actual_size(&self) -> WidgetBound {self.buffer.bound()}
}

/// Frequently recalculated
struct TemporaryData {
    last_frame: SystemTime,
    animating: Vec<Id>
}

fn fix_node(val: WidgetEnum) -> WidgetData {
    WidgetData {
        id: Id::default(),
        last_bound: WidgetBound {width: 0,height: 0},
        parent_position: (0,0),
        child_position: (0,0),
        drawn_as: WidgetBound { width: 0, height: 0 },
        widget: val,
        buffer: WidgetBuffer::new(0,0)
    }
}

fn fix_tree(tree: RecTree<WidgetEnum>) -> RecTree<WidgetData> {
    tree.recursive_map(&|(b, val)| (b, fix_node(val)))
}

/******************************************************************************/
/******************************************************************************/
/*********************************** Main UI **********************************/
/******************************************************************************/
/******************************************************************************/

/// UI.
/// Widgets must be in a proper tree structure when drawn,
/// or else errors will occur.
pub struct UI {
    // pub(super) items and methods are meant only to be used by widgets.
    pub(super) term: Terminal,
    widgets: Vec<Tree<WidgetData>>,
    context: usize,
    width: u16,
    height: u16,
    temp: TemporaryData
}

impl UI {
    /// Creates a new UI.
    /// This also changes the content of the terminal screen;
    /// self.stop() must be called to correctly restore it.
    pub fn new(res: &mut UIResources) -> Self {
        let mut ui = Self {
            widgets: Vec::new(),
            context: 0,
            width: 0,
            height: 0,
            term: Terminal::start(res),
            temp: TemporaryData {
                last_frame: SystemTime::now(),
                animating: Vec::new()
            }
        };
        let (width, height) = ui.term.size(res).unwrap_or_else(|| (30,30));
        ui.height = height;
        ui.width = width;
        return ui;
    }

    fn initialize_descend(&mut self, id: Id) {
        let widgets = &mut self.widgets[self.context];
        let children = widgets.children(id);
        widgets[id].id = id;
        if widgets[id].widget.child_number(children) != children {
            panic!()
        }

        for i in 0..children {
            let child = self.widgets[self.context].child(id, i as i32);
            self.initialize_descend(child);
        }
    }

    /// Creates a new context with the given widgets.  
    /// Automatically changes to the new context.  
    pub fn new_context(&mut self, widgets: RecTree<WidgetEnum>) -> (usize, Vec<Id>) {
        let tree = fix_tree(widgets);
        let mut vars = Vec::new();
        self.widgets.push(Tree::from((&mut vars, tree)));
        let old_context = self.context;
        let context = self.widgets.len()-1;
        self.context = context;
        let root = self.widgets[self.context].root();
        self.initialize_descend(root);
        self.change_descend(root, self.fullscreen());
        self.context = old_context;
        return (context,vars);
    }

    pub fn root(&self) -> Id {
        return self.widgets[self.context].root();
    }

    fn fullscreen(&self) -> WidgetBound {
        WidgetBound {width:self.width,height:self.height}
    }
    
    pub fn get_context_widgets(&self, context: usize) -> &Tree<WidgetData> {
        return &self.widgets[context];
    }

    pub fn remove_context(&mut self, context: usize) {
        self.widgets.remove(context);
    }

    pub fn context(&self) -> usize {self.context}
    
    pub fn set_context(&mut self, context: usize) {
        self.context = context;
    }

    pub fn replace_context(&mut self, context: usize, widgets: RecTree<WidgetEnum>) -> Vec<Id> {
        let tree = fix_tree(widgets);
        let mut vars = Vec::new();
        self.widgets[context] = Tree::from((&mut vars, tree));
        let old_context = self.context;
        self.context = context;
        let root = self.widgets[self.context].root();
        self.initialize_descend(root);
        self.change_descend(root, self.fullscreen());
        self.context = old_context;
        return vars;
    }

    pub fn replace_context_2(&mut self, context: usize, widgets: Tree<WidgetData>) {
        self.widgets[context] = widgets;
        let old_context = self.context;
        self.context = context;
        let root = self.widgets[context].root();
        self.initialize_descend(root);
        self.change_descend(root, self.fullscreen());
        self.context = old_context;
    }

    /****************** **********************/
    pub fn children(&self, id: Id) -> usize {self.widgets[self.context].children(id)}
    pub fn child_num(&self, parent: Id, child: Id) -> usize {
        return self.widgets[self.context].child_num(parent,child).unwrap();
    }

    pub fn flat_replace(&mut self, old_id: Id, x: WidgetEnum) -> Id {
        let id = self.widgets[self.context].cut(old_id).purge().add_node(fix_node(x));
        self.initialize_descend(id);
        match self.widgets[self.context].parent(id) {
            Some(parent) => {
                self.apply_change(parent);
            },
            None => self.change_descend(id, self.fullscreen()),
        }
        return id;
    }

    pub fn replace(&mut self, id: Id, tree: RecTree<WidgetEnum>) -> Vec<Id> {
        let tree = fix_tree(tree);
        let mut vars = Vec::new();
        let id = self.widgets[self.context].cut(id).purge().add_rectree((&mut vars, tree));
        self.initialize_descend(id);
        match self.widgets[self.context].parent(id) {
            Some(parent) => {
                self.apply_change(parent);
            },
            None => self.change_descend(id, self.fullscreen()),
        }
        return vars;
    }

    pub fn remove_child<T>(&mut self, parent: Id, child: i32)
    where T: RemoveChild, for<'a> &'a mut T: TryFrom<&'a mut WidgetEnum> {
        let children = self.widgets[self.context].children(parent);
        let child = if child < 0 {(children as i32+child) as usize} else {child as usize};
        let if2;
        if let Ok(widget) = <&mut T>::try_from(&mut self.widgets[self.context][parent].widget) {
            if widget.remove_child(child) {
                if2 = true;
            } else {
                if2 = false;
            }
        } else {
            if2 = false;
        }
        if if2 {
            let widgets = &mut self.widgets[self.context];
            widgets.cut(widgets.child(parent,child as i32)).purge().collapse();
            self.apply_change(parent);
        }
    }

    pub fn append_to<T>(&mut self, to: Id, tree: RecTree<WidgetEnum>) -> Vec<Id>
    where T: AddChild, for<'a> &'a mut T: TryFrom<&'a mut WidgetEnum> {
        let children = self.widgets[self.context].children(to);
        let if2;
        if let Ok(widget) = <&mut T>::try_from(&mut self.widgets[self.context][to].widget) {
            if widget.add_child(children) {
                if2 = true;
            } else {
                if2 = false;
            }
        } else {
            if2 = false;
        }
        if if2 {
            let tree = fix_tree(tree);
            let mut vars = Vec::new();
            let widgets = &mut self.widgets[self.context];
            widgets.append(to).add_rectree((&mut vars, tree));
            let id = widgets.child(to,-1);
            self.initialize_descend(id);
            // Changes to children must be applied to the PARENT, so that
            // its bound can be fixed.
            self.apply_change(to);
            return vars;
        }
        return Vec::with_capacity(0);
    }

    pub fn insert_new<T>(&mut self, to: (Id, usize), tree: RecTree<WidgetEnum>) -> Vec<Id>
    where T: AddChild, for<'a> &'a mut T: TryFrom<&'a mut WidgetEnum> {
        let if2;
        if let Ok(widget) = <&mut T>::try_from(&mut self.widgets[self.context][to.0].widget) {
            if widget.add_child(to.1) {
                if2 = true;
            } else {
                if2 = false;
            }
        } else {
            if2 = false;
        }
        if if2 {
            let tree = tree.recursive_map(&|(b, val)| (b, WidgetData {
                id: Id::default(),
                last_bound: WidgetBound {width: 0,height: 0},
                parent_position: (0,0),
                child_position: (0,0),
                drawn_as: WidgetBound { width: 0, height: 0 },
                widget: val,
                buffer: WidgetBuffer::new(0,0)
            }));
            let mut vars = Vec::new();
            let widgets = &mut self.widgets[self.context];
            widgets.insert(to.0, to.1).add_rectree((&mut vars, tree));
            let id = widgets.child(to.0,to.1 as i32);
            self.initialize_descend(id);
            self.apply_change(to.0);
            return vars;
        }
        return Vec::with_capacity(0);
    }

    /**************** ******************/

    fn change_descend(&mut self, id: Id, bound: WidgetBound) {
        if bound.zero() {return;}
        let widgets = &mut self.widgets[self.context];
        let old_bound = widgets[id].last_bound;
        if bound != old_bound {
            widgets[id].last_bound = bound;
            let WidgetData {ref mut widget, ref mut buffer, ..} = widgets[id];
            widget.update_size(bound, buffer);
            let mut children = widgets[id].widget.child_sizes(bound);
            assert!(children.len() == widgets.children(id));
            for (i,next_bound) in children.drain(..).enumerate() {
                let child = self.widgets[self.context].child(id,i as i32);
                self.change_descend(child, next_bound);
            }
        }
        let (WidgetData {
            ref mut widget, ref mut buffer, ..
        }, mut children) = self.widgets[self.context].write_children(id);
        widget.draw(&mut children, buffer);
    }

    fn apply_change(&mut self, id: Id) {
        let widgets = &mut self.widgets[self.context];
        let bound = widgets[id].last_bound;
        let mut children = widgets[id].widget.child_sizes(bound);
        for (i,next_bound) in children.drain(..).enumerate() {
            let child = self.widgets[self.context].child(id,i as i32);
            self.change_descend(child, next_bound);
        }
        let mut next = Some(id);
        let widgets = &mut self.widgets[self.context];
        while let Some(parent) = next {
            let (WidgetData {
                ref mut widget, ref mut buffer, ..
            }, mut children) = widgets.write_children(parent);
            widget.draw(&mut children, buffer);
            next = widgets.parent(parent);
        }
    }

    fn check_animate(&mut self, start: Id) {
        self.temp.animating.clear();
        let widgets = &mut self.widgets[self.context];
        let mut df_store = widgets.df_reverse(start);
        while let ItrResult::Continue((ReverseExtract {id, ..}, itr)) = df_store.next(widgets) {
            if widgets[id].widget.animates() {
                self.temp.animating.push(id);
            }
            df_store = itr.provide(());
        }
    }

    fn event_result(&mut self, start: Id, poll: &Poll, mut event: Event, res: &mut UIResources) -> EventResultInternal {
        let event_translation = if let Event::Key(KeyEvent {code, modifiers, kind: KeyEventKind::Press, ..}) = event {
            let raw_key = RawKey {
                code,
                modifiers,
            };
            res.map_key(&raw_key)
        } else {None};
        let mut id = start;
        
        loop {
            if let Some(thing) = match_poll(poll, &event, event_translation, id) {
                return EventResultInternal::PollResult(thing);
            }
            match self.widgets[self.context][id].widget.poll(id, event.clone(), event_translation, poll) {
                EventResult::Nothing => return EventResultInternal::Nothing,
                EventResult::Changed => return EventResultInternal::Changed(id),
                EventResult::PollResult(result) => return EventResultInternal::PollResult(result),
                EventResult::PassToChild(child_num) => {
                    let child_id = self.widgets[self.context].child(id, child_num);
                    if let Event::Mouse(MouseEvent { kind, column, row, modifiers }) = &event {
                        let WidgetData {
                            parent_position, child_position, drawn_as, ..
                        } = self.widgets[self.context][child_id];
                        let mod_x = *column as i32-parent_position.0 as i32+child_position.0;
                        let mod_x = if mod_x < 0 {0 as u16} else if mod_x >= drawn_as.width as i32 {drawn_as.width} else {mod_x as u16};
                        let mod_y = *row as i32-parent_position.1 as i32+child_position.1;
                        let mod_y = if mod_y < 0 {0 as u16} else if mod_y >= drawn_as.height as i32 {drawn_as.height} else {mod_y as u16};
                        event = Event::Mouse(MouseEvent {kind: *kind, modifiers: *modifiers, column: mod_x, row: mod_y});
                    }
                    id = child_id;
                }
            }
        }
    }

    // Event -> Action logic:
    // Events come in, and get consumed by the UI and any widgets they pass through.
    // When an event is consumed, consume_action returns None.
    // This get_action will continue to run until an event is not consumed, and the non-consuming
    // widget produces Some(Action) instead.
    const FRAME_LENGTH: Duration = Duration::from_millis(30);
    pub fn poll_from(&mut self, poll: &Poll, res: &mut UIResources) -> PollResult {
        let root = self.widgets[self.context].root();
        self.check_animate(root);
        self.to_terminal(res);
        loop {
            let event;
            if self.temp.animating.len() > 0 {
                let wait = match SystemTime::now().duration_since(self.temp.last_frame) {
                    Ok(time_since_last_frame) if Self::FRAME_LENGTH > time_since_last_frame => Self::FRAME_LENGTH - time_since_last_frame,
                    _ => Duration::ZERO
                };
                let maybe_event = self.term.event_hang_for(wait, res);
                match maybe_event {
                    Some(e) => event = e,
                    None => {
                        self.temp.last_frame=SystemTime::now();
                        let mut i = 0;
                        while i < self.temp.animating.len() {
                            let ani_id = self.temp.animating[i];
                            let WidgetData {
                                ref mut buffer, ref mut widget, ..
                            } = self.widgets[self.context][ani_id];
                            let continued = widget.next_frame(buffer);
                            if !continued {
                                self.temp.animating.swap_remove(i);
                                if poll.contains(&(ani_id, Candidate::FinishAnimation)) {
                                    return (ani_id, Match::FinishAnimation);
                                }
                                // Need to reevaluate whatever this was replaced with
                            } else {
                                i += 1;
                            }
                        }
                        self.to_terminal(res);
                        continue;
                    }
                };
            } else {
                match self.term.event_hang(res) {
                    Some(e) => event = e,
                    None => continue
                }
            }
            if let Event::Resize(x, y) = event {
                self.width = x;
                self.height = y;
                self.change_descend(root, self.fullscreen());
                self.check_animate(root);
                self.to_terminal(res);
                continue;
            }
            let result = self.event_result(root, poll, event, res);
            match result {
                EventResultInternal::Nothing => continue,
                EventResultInternal::Changed(id) => {
                    self.apply_change(id);
                    self.check_animate(root);
                    self.to_terminal(res);
                },
                EventResultInternal::PollResult(res) => return res
            }
        }
    }
    
    fn to_terminal(&mut self, res: &mut UIResources) {
        let widgets = &self.widgets[self.context];
        let WidgetData{ref buffer, ..} = widgets[widgets.root()];
        for y in 0..buffer.area.height().min(self.height as usize) {
            self.term.move_to((0,y as u16), res);
            for x in 0..buffer.area.width().min(self.width as usize) {
                match buffer.area[(x,y)] {
                    BufferItem::Is(ch, style) => {
                        self.term.set_style(style, res);
                        self.term.wchar(ch, res);
                    },
                    BufferItem::References(id, x, y) => {
                        let x = x as usize;
                        let y = y as usize;
                        if widgets.has(id) {
                            match widgets[id].buffer.area.get((x,y)) {
                                Some(BufferItem::Is(ch, style)) => {
                                    self.term.set_style(*style, res);
                                    self.term.wchar(*ch, res);
                                },
                                Some(BufferItem::References(..)) => {
                                    unreachable!();
                                },
                                None => {
                                    self.term.set_style(Style {
                                        bg: Some(Rgb(255,0,255)),
                                        fg: Some(Rgb(0,0,0)),
                                        bold: false, ital: false, reverse: false
                                    }, res);
                                    self.term.wchar(' ', res);
                                }
                            }
                        } else {
                            self.term.set_style(Style {
                                bg: Some(Rgb(255,0,0)),
                                fg: Some(Rgb(0,0,0)),
                                bold: false, ital: false, reverse: false
                            }, res);
                            self.term.wchar(' ', res);
                        }
                        
                    }
                }
            }
        }
        // Windows terminals unhide the cursor whenever the terminal
        // is resized. Fun!
        self.term.hide_cursor(res);
        self.term.finish(res);
    }
    
    pub fn mut_widget<'a,T>(&'a mut self, widget: Id) -> Option<ChangeTracker<'a,T>>
    where T: IsWidget, 
    for<'b> &'b mut T: TryFrom<&'b mut WidgetEnum>,
    for<'b> &'b T: TryFrom<&'b WidgetEnum> {
        if T::is_widget(&self.widgets[self.context][widget].widget) {
            Some(ChangeTracker {
                ui: self,
                id: widget,
                changed: false,
                typ: std::marker::PhantomData
            })
        } else {None}
    }

    pub fn widget<'a,T>(&'a self, widget: Id) -> Option<&'a T>
    where T: IsWidget, 
    for<'b> &'b T: TryFrom<&'b WidgetEnum> {
        if let Ok(w) = <&T>::try_from(&self.widgets[self.context][widget].widget) {
            Some(w)
        } else {None}
    }
    
    pub fn stop(&mut self, res: &mut UIResources) {
        self.term.stop(res);
    }
}

pub struct ChangeTracker<'a,T> where
for<'b> &'b mut T: TryFrom<&'b mut WidgetEnum>,
for<'b> &'b T: TryFrom<&'b WidgetEnum> {
    ui: &'a mut UI,
    changed: bool,
    id: Id,
    typ: std::marker::PhantomData<T>
}

use std::ops::{Deref,DerefMut};

impl<'a,T> Deref for ChangeTracker<'a,T> where
for<'b> &'b mut T: TryFrom<&'b mut WidgetEnum>,
for<'b> &'b T: TryFrom<&'b WidgetEnum> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match <&T>::try_from(&self.ui.widgets[self.ui.context][self.id].widget) {
            Ok(w) => w,
            // Checked during the creation of the struct;
            // this should not occur
            _ => unreachable!()
        }
    }
}

impl<'a,T> DerefMut for ChangeTracker<'a,T> where
for<'b> &'b mut T: TryFrom<&'b mut WidgetEnum>,
for<'b> &'b T: TryFrom<&'b WidgetEnum> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.changed = true;
        match <&mut T>::try_from(&mut self.ui.widgets[self.ui.context][self.id].widget) {
            Ok(w) => w,
            // Checked during the creation of the struct;
            // this should not occur
            _ => unreachable!()
        }
    }
}

impl<'a,T> AsRef<T> for ChangeTracker<'a,T> where
for<'b> &'b mut T: TryFrom<&'b mut WidgetEnum>,
for<'b> &'b T: TryFrom<&'b WidgetEnum> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

impl<'a,T> AsMut<T> for ChangeTracker<'a,T> where
for<'b> &'b mut T: TryFrom<&'b mut WidgetEnum>,
for<'b> &'b T: TryFrom<&'b WidgetEnum> {
    fn as_mut(&mut self) -> &mut T {
        self.deref_mut()
    }
}

impl<'a,T> Drop for ChangeTracker<'a,T> where
for<'b> &'b mut T: TryFrom<&'b mut WidgetEnum>,
for<'b> &'b T: TryFrom<&'b WidgetEnum> {
    fn drop(&mut self) {
        if self.changed {
            self.ui.apply_change(self.id);
        }
    }
}


/******************************************************************************/
/******************************************************************************/
/******************************* Widget buffers *******************************/
/******************************************************************************/
/******************************************************************************/

#[derive(Serialize,Deserialize,PartialEq,Eq,Copy,Clone)]
enum BufferItem {
    References(Id,u16,u16),
    Is(char,Style)
}

#[derive(Serialize,Deserialize)]
/// A 'terminal' based drawing thing. This keeps an internal cursor,
/// and characters are written from left to right.  
/// The internal buffer here will later be written out to the output medium,
/// which is handled in terminal_base.
pub struct WidgetBuffer {
    area: Array2D<BufferItem>,
    cursor_pos: (usize,usize),
}

impl WidgetBuffer {
    pub fn new(width: usize,height: usize) -> Self {
        Self {
            area: Array2D::new_sized(width, height, BufferItem::Is(' ',Style::default())),
            cursor_pos: (0,0),
        }
    }

    pub fn clear(&mut self) {
        self.area.set_all(BufferItem::Is(' ',Style::default()));
        self.cursor_pos = (0,0);
    }

    /// Copies data from another buffer to some location in this buffer.  
    /// This is used by UI to write child widgets to the parent's buffer,
    /// and it is unlikely to need to be used elsewhere.
    fn copy_from(&mut self, topleft: (u16,u16), bound: WidgetBound, inner_offset: (i32,i32), other_id: Id, other: &Self) {
        // Fix bound: it cannot be larger than the width/height of this buffer.
        let copy_width = if (bound.width+topleft.0) as usize > self.area.width() {
            self.area.width() as u16 - topleft.1
        } else {bound.width};
        let copy_height = if (bound.height+topleft.1) as usize > self.area.height() {
            self.area.height() as u16 - topleft.1
        } else {bound.height};

        let minimum_viable_row = if inner_offset.1 < 0 {(-inner_offset.1) as u16} else {0};
        let minimum_viable_col = if inner_offset.0 < 0 {(-inner_offset.0) as u16} else {0};
        let maximum_viable_col = if inner_offset.0+other.area.width() as i32 <= 0 || inner_offset.0+copy_width as i32 <= 0 {0} else {
            (inner_offset.0 + (other.area.width() as i32).min(copy_width as i32)) as u16
        };
        let maximum_viable_row = if inner_offset.1+other.area.height() as i32 <= 0 || inner_offset.1+copy_height as i32 <= 0 {0} else {
            (inner_offset.1 + (other.area.height() as i32).min(copy_height as i32)) as u16
        };

        for copy_row in minimum_viable_row..maximum_viable_row {
            let outer_y = copy_row+topleft.1;
            let inner_y = (inner_offset.0+copy_row as i32) as u16;
            for copy_col in minimum_viable_col..maximum_viable_col {
                let outer_x = copy_col+topleft.0;
                let inner_x = (inner_offset.0+copy_col as i32) as u16;
                self.area[(outer_x as usize,outer_y as usize)] = match other.area[(inner_x as usize,inner_y as usize)] {
                    BufferItem::Is(..) => BufferItem::References(other_id, inner_x,inner_y),
                    reference => reference
                };
            }
        }
    }

    /// Provides how much the cursor can move until reaching the end of the current bound.
    pub fn width_remaining(&self) -> usize {
        if self.cursor_pos.0 >= self.area.width() || self.cursor_pos.1 >= self.area.height() {
            0
        } else {
            self.area.width() - self.cursor_pos.0
        }
    }

    /// The width and height this buffer is expected to be filled with
    pub fn bound(&self) -> WidgetBound {
        let x = self.area.dim();
        WidgetBound {width:x.0 as u16,height:x.1 as u16}
    }

    /// Resizes and clears the data in this buffer.
    pub fn resize(&mut self, bound: WidgetBound) {
        if bound == self.bound() {
            return;
        }
        self.area.resize(bound.width as usize,bound.height as usize,BufferItem::Is(' ',Style::default()));
    }

    pub fn char(&self, coord: (usize,usize)) -> Option<char> {
        if self.area.within(coord) {
            if let BufferItem::Is(ch, _) = self.area[coord] {
                return Some(ch);
            }
        }
        return None;
    }

    pub fn set_char(&mut self, loc: (usize,usize), ch: char) {
        if loc.0 >= self.area.width() || loc.1 >= self.area.height() {
            return;
        }
        match &mut self.area[loc] {
            BufferItem::Is(old,_) => *old = ch,
            _ => (),
        }
    }

    pub fn style(&mut self, coord: (usize,usize)) -> Option<&mut Style> {
        if self.area.within(coord) {
            if let BufferItem::Is(_, sty) = &mut self.area[coord] {
                return Some(sty);
            }
        }
        return None;
    }

    /// Moves the cursor to the next line.
    pub fn next_line(&mut self) {
        self.cursor_pos = (0, self.cursor_pos.1+1);
    }
    
    /// Outputs ' ', in the last used style, until the end of the line.
    /// Additionally, moves the cursor to the next line (under wherever it was last moved)
    pub fn blank_till_end(&mut self, style: Style) {
        self.blank(style);
        self.next_line();
    }

    pub fn blank(&mut self, style: Style) {
        if !self.area.within(self.cursor_pos) {
            return;
        }
        for x in self.cursor_pos.0..self.area.width() {
            self.area[(x, self.cursor_pos.1)] = BufferItem::Is(' ',style);
        }
        self.cursor_pos = (self.area.width(), self.cursor_pos.1);
    }
    
    /// Writes a string to the terminal, bounded by the writable area.  
    /// Please only write strings with single character UTF8 bits;
    /// languages like Arabic are, unfortunately, probably not supported.  
    /// Returns an iterator of the remaining characters of the string that did not
    /// fit in the buffer, which may be empty.
    pub fn wstr<'a>(&mut self, thing: &'a str, style: Style) -> std::str::Chars<'a> {
        let mut iter = thing.chars();
        self.wchars(&mut iter, None, style);
        return iter;
    }

    /// Writes a Chars<'a>.  
    /// Not to be confused with wchar.
    /// If it reaches the end of the buffer, not all of the iterator will be consumed.
    pub fn wchars(&mut self, chars: &mut std::str::Chars<'_>, num_write: Option<usize>, style: Style) -> usize {
        if self.width_remaining() == 0 {
            return 0;
        }
        let mut written = 0;
        let max = if let Some(num) = num_write {num} else {usize::MAX};
        let mut next = chars.next();
        while next != None && written < max && self.area.within(self.cursor_pos) {
            let ch = next.unwrap();
            written += 1;
            match ch {
                '\n' => {
                    self.blank(style);
                    return written;
                },
                '\t' => {
                    for _ in 0..self.width_remaining().min(4) {
                        self.area[(self.cursor_pos.0, self.cursor_pos.1)] = BufferItem::Is(' ', style);
                        self.cursor_pos.0 += 1;
                    }
                },
                c => {
                    self.area[(self.cursor_pos.0, self.cursor_pos.1)] = BufferItem::Is(c, style);
                    self.cursor_pos.0 += 1;
                }
            }
            next = chars.next();
        }
        return written;
    }

    pub fn wchar(&mut self, thing: char, style: Style) {
        if self.width_remaining() >= 1 {
            self.area[(self.cursor_pos.0, self.cursor_pos.1)] = BufferItem::Is(thing, style);
            self.cursor_pos.0 += 1;
        }
    }

    pub fn wchar_at(&mut self, loc: (usize, usize), thing: char, style: Style) {
        if loc.0 >= self.area.width() || loc.1 >= self.area.height() {
            return;
        }
        self.area[loc] = BufferItem::Is(thing, style);
    }
    
    pub fn wtile(&mut self, tile: &UITile) {
        if self.width_remaining() > 0 {
            self.wtile_at(self.cursor_pos, tile);
            self.cursor_pos.0 += 1;
        }
    }

    pub fn wtile_at(&mut self, loc: (usize, usize), tile: &UITile) {
        if loc.0 >= self.area.width() || loc.1 >= self.area.height() {
            return;
        }
        self.area[loc] =
            BufferItem::Is(tile.fg.ch, Style {
                fg: Some(tile.fg.color),
                bg: Some(tile.bg),
                ital: tile.fg.ital,
                bold: tile.fg.bold,
                reverse: false,
            });
    }

    pub fn width(&self) -> u16 {self.area.width() as u16}
    pub fn height(&self) -> u16 {self.area.height() as u16}
    
    /// This function should never be used interally by terminal_base.
    /// Moves the cursor with respect to the current writable area.
    pub fn move_to(&mut self, col: u16, row: u16) {
        self.cursor_pos = (col.into(),row.into());
    }
}
