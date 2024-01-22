use std::any::Any;
use std::collections::HashMap;
use crate::common::{KeyAction, TileStyle, NORMALSTYLE, DataBox};
use crate::ui::{UI, CharArea, Action, Widget, Key, KeyCode, KeyModifiers, NoAction};
use super::identifiers::UniqObj;
use super::vision::traverser_view_msg;
use super::gamedata::{GameData};
use super::traverser::Traverser;
use super::object::{Object, ObjTraverser};

// Maybe this shouldn't be an enum?
pub enum ActionType {
    Damage(u32),
    Temperature(u32),
    None,
}

// Keep entities small! They have to be copied!!
// Data for your entity should be kept in GameData, which will always be available!
pub trait Entity {
    // General methods
    fn get_id(&self) -> u32;
    fn as_any(&self) -> &dyn Any;
    fn react(&self) -> fn(this: UniqObj, action: ActionType, data: &mut GameData);
    fn act(&self) -> fn(this: UniqObj, data: &mut GameData);
}

#[derive(Clone)]
enum YouAction {
    Left,
    Right,
    Up,
    Down,
    Wait,
    MakeWatcher,
} crate::to_action!(YouAction);

#[derive(Clone)]
pub struct You {
    id: u32,
    my_window: usize,
    objt: ObjTraverser,
}

impl Entity for You {    
    fn get_id(&self) -> u32 {
        return self.id;
    }
    fn as_any(&self) -> &dyn Any { self }
    // Function redirection has to happen so GameData can extract the correct function
    // without mutably borrowing the object.
    fn react(&self) -> fn(this: UniqObj, action: ActionType, data: &mut GameData) { return You::react; }
    fn act(&self) -> fn(this: UniqObj, data: &mut GameData) { return You::act; }
}

impl You {
    pub fn new(id: u32, on: Traverser, data: &mut GameData) -> You {
        let obj = Box::new(
            Object {entity_id: data.next_ent_id(), id: 0,
                style: TileStyle {
                    ch: Some('@'),
                    sty: NORMALSTYLE,
                }
            }
        );
        let obj_trav = ObjTraverser::new(on, obj, data);
        let ui = &data.ui;
        let mut my_widget = CharArea::<YouAction>::new();
        my_widget.write().keymap(HashMap::from([
            (Key { code: KeyCode::Left, modifiers: KeyModifiers::empty(), }, YouAction::Left),
            (Key { code: KeyCode::Right, modifiers: KeyModifiers::empty(), }, YouAction::Right),
            (Key { code: KeyCode::Down, modifiers: KeyModifiers::empty(), }, YouAction::Down),
            (Key { code: KeyCode::Up, modifiers: KeyModifiers::empty(), }, YouAction::Up),
            (Key { code: KeyCode::Char('j'), modifiers: KeyModifiers::empty(), }, YouAction::MakeWatcher),
            (Key { code: KeyCode::Char('.'), modifiers: KeyModifiers::empty(), }, YouAction::Wait),
        ]));
        let msg = traverser_view_msg(on, 40, 20, &data);
        my_widget.write().set(msg);
        let my_window = ui.write().add_widget(Box::new(my_widget));
        data.add_child_widget(my_window);
        data.add_update(50, UniqObj {entity: id, obj: 0});
        return You { id: id, my_window: my_window, objt: obj_trav, };
    }
    
    fn react(this: UniqObj, action: ActionType, data: &mut GameData) {
    }
    
    fn act(this: UniqObj, data: &mut GameData) {
        let act_maybe = data.action();
        if let None = act_maybe {
            return;
        }
        let act_any = act_maybe.unwrap();
        let mut still_ent = data.ent_clone::<You>(this.entity).unwrap();
        let act = act_any.any().downcast_ref::<YouAction>();
        
        match act {                
            Some(YouAction::Up) => {still_ent.objt.move_obj(0, data)},
            Some(YouAction::Down) => {still_ent.objt.move_obj(1, data)},
            Some(YouAction::Left) => {still_ent.objt.move_obj(2, data)},
            Some(YouAction::Right) => {still_ent.objt.move_obj(3, data)},
            Some(YouAction::MakeWatcher) => {
                let ent = Box::new(Watcher::new(data.next_ent_id(), still_ent.objt.traverser(), data));
                data.add_boxed_ent(ent);
            },
            Some(YouAction::Wait) => (),
            None => return,
        };
        let my_widget = data.ui.read().widget::<CharArea<YouAction>>(still_ent.my_window);
        let msg = traverser_view_msg(still_ent.objt.traverser(), 40, 20, &data);
        my_widget.unwrap().write().set(msg);
        data.replace_ent(this.entity, still_ent);
        data.add_update(50, UniqObj {entity: this.entity, obj: 0});
    }
}

#[derive(Clone)]
pub struct Watcher {
    id: u32,
    my_window: usize,
    objt: ObjTraverser,
}

impl Entity for Watcher {    
    fn get_id(&self) -> u32 {
        return self.id;
    }
    fn as_any(&self) -> &dyn Any { self }
    // Function redirection has to happen so GameData can extract the correct function
    // without mutably borrowing the object.
    fn react(&self) -> fn(this: UniqObj, action: ActionType, data: &mut GameData) { return Self::react; }
    fn act(&self) -> fn(this: UniqObj, data: &mut GameData) { return Self::act; }
}

impl Watcher {
    pub fn new(id: u32, on: Traverser, data: &mut GameData) -> Self {
        let obj = Box::new(
            Object {entity_id: data.next_ent_id(), id: 0,
                style: TileStyle {
                    ch: Some('t'),
                    sty: NORMALSTYLE,
                }
            }
        );
        let obj_trav = ObjTraverser::new(on, obj, data);
        let ui = &data.ui;
        let mut my_widget = CharArea::<NoAction>::new();
        
        let msg = traverser_view_msg(on, 40, 20, &data);
        my_widget.write().set(msg);
        let my_window = ui.write().add_widget(Box::new(my_widget));
        data.add_child_widget_keep_context(my_window);
        data.add_update(50, UniqObj {entity: id, obj: 0});
        return Self { id: id, my_window: my_window, objt: obj_trav, };
    }
    
    fn react(this: UniqObj, action: ActionType, data: &mut GameData) {
    }
    
    fn act(this: UniqObj, data: &mut GameData) {
        let mut still_ent = data.ent_clone::<Watcher>(this.entity).unwrap();
        
        let my_widget = data.ui.read().widget::<CharArea<NoAction>>(still_ent.my_window);
        let msg = traverser_view_msg(still_ent.objt.traverser(), 40, 20, &data);
        my_widget.unwrap().write().set(msg);
        data.replace_ent(this.entity, still_ent);
        data.add_update(49, UniqObj {entity: this.entity, obj: 0});
    }
}