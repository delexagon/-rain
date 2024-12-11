use super::{Interrupt, ObjTile, UIHandler, TemplateID, EntityID, Object, Traverser, UniqTile, GameData};
use crate::ui::{WidgetEnum, Match, Candidate, Poll};
use crate::{errstr, Id};
use macros::func_enum;

fn weird_key(data: &mut GameData) -> Interrupt {
    data.resources.as_mut().err(&errstr!(crate::translate!(behaviors_bad_key)));
    return Interrupt::AbortError;
}

fn move_object(from: Traverser, to: Traverser, obj: Object, data: &mut GameData) -> bool {
    if !data.world.passable(to.tile) || !data.object_passable(to.tile) {
        return false;
    }
    data.world.move_object(from.tile, to.tile, obj);
    return true;
}

/// A general interaction function called by all entities.
fn interact(id: usize, tile: UniqTile, data: &mut GameData) -> bool {
    let mut interaction = None;
    for obj in data.world.objects_on(tile) {
        let template = data.entities.template(obj.template_id);
        if let Some(func) = &template.on_interact {
            interaction = Some((*obj, template.on_interact.clone().unwrap()));
        }
    }
    if let Some((obj, func)) = interaction {
        func.call((obj, tile), id, data);
        return true;
    }
    false
}

// Returns whether the entity moved successfully.
fn move_entity(id: usize, to: Traverser, data: &mut GameData) -> bool {
    if let Some((obj, from)) = data.entities[id].loc {
        if move_object(from, to, obj, data) {
            data.entities[id].loc = Some((obj, to));
            return true;
        }
    }
    return false;
}

fn pick_up_object(id: usize, tile: UniqTile, data: &mut GameData) -> Option<Object> {
    let entity = &mut data.entities[id];
    if entity.contains.is_none() {
        return None;
    }
    let objects = data.world.objects_mut(tile)?;

    for i in (0..objects.len()).rev() {
        // For now, entities cannot pick up a component of themselves
        if objects[i].entity_id.is_none() || objects[i].entity_id.unwrap() != id {
            entity.contains.as_mut().unwrap().push(objects.swap_remove(i));
            break;
        }
    }
    None
}

fn redraw_los(id: usize, (window, dim): (Id, (usize,usize)), data: &mut GameData) -> Result<bool, Interrupt> {
    let trav;
    {
        let entity = &data.entities[id];
        if entity.loc.is_some() {
            trav = entity.loc.as_ref().unwrap().1;
        } else {
            return Ok(false)
        }
    }
    UIHandler::update_los(window, trav, data);
    Ok(true)
}

func_enum! {
#[derive(Clone,Debug,Eq,PartialEq,Hash,serde::Serialize,serde::Deserialize)]
pub enum OnInteract: fn(object: ObjTile, interactor: EntityID, data: &mut GameData) {
    fn ChangeTemplate(template_name: &String, object: ObjTile, _interactor: EntityID, data: &mut GameData) {
        let maybe_obj = data.world.find_object(object.1, object.0);
        if let Some(obj) = maybe_obj {
            if let Some(template) = data.gen.template_names.get(template_name) {
                obj.template_id = *template;
            }
        }
    }
}
}

func_enum! {
#[derive(Copy,Clone,Debug,Eq,PartialEq,Hash,serde::Serialize,serde::Deserialize)]
pub enum Behavior: fn(entity: EntityID, data: &mut GameData) -> Result<(), Interrupt> {
    // Drawing windows cannot be done during the entity creation function,
    // thus a separate update must happen immediately after they are created.
    fn PlayerStartingDraw(id: usize, data: &mut GameData) -> Result<(), Interrupt> {
        let widget = data.ui.main_character_view;
        redraw_los(id, (widget, data.ui.area_size(widget).ok_or(Interrupt::AbortError)?), data)?;
        Ok(())
    }

    fn Player(id: usize, data: &mut GameData) -> Result<(), Interrupt> {
        let entity = &data.entities[id];
        // Adding update must be first (so an error during this update does not softlock)
        // Redrawing must occur before moving
        data.updates.add_update(entity.speed, id, Behavior::Player);
        let here;
        match entity.loc {
            None => {
                data.resources.as_mut().err(&errstr!("The player no location! What?"));
                return Err(Interrupt::AbortError);
            },
            Some(loc) => here = loc
        }
        let widget = data.ui.main_character_view;
        let poll_direction = Poll::from([
            (widget, Candidate::Up),
            (widget, Candidate::Down),
            (widget, Candidate::Left),
            (widget, Candidate::Right),
            data.ui.exit_key(),
        ]);
        let poll = Poll::from([
            (widget, Candidate::Up),
            (widget, Candidate::Down),
            (widget, Candidate::Left),
            (widget, Candidate::Right),
            (widget, Candidate::Debug),
            (widget, Candidate::Interact),
            (widget, Candidate::Wait),
            (widget, Candidate::Get),
            data.ui.exit_key(),
        ]);
        enum Todo {
            Move {dir: u8},
            Interact {dir: u8},
            Nothing,
        }
        use Todo::{Move, Interact, Nothing};
        loop {
            let what_todo;
            match data.ui.poll(&poll, data.resources.as_mut()) {
                x if x == data.ui.exit_returned() => return Err(Interrupt::MainMenu),
                (_, Match::Standard(Candidate::Up))    => {what_todo = Move {dir:0};},
                (_, Match::Standard(Candidate::Down))  => {what_todo = Move {dir:1};},
                (_, Match::Standard(Candidate::Left))  => {what_todo = Move {dir:2};},
                (_, Match::Standard(Candidate::Right)) => {what_todo = Move {dir:3};},
                (_, Match::Standard(Candidate::Wait))  => {what_todo = Nothing;},
                (_, Match::Standard(Candidate::Interact)) => {
                    match data.ui.poll(&poll_direction, data.resources.as_mut()) {
                        x if x == data.ui.exit_returned() => return Err(Interrupt::MainMenu),
                        (_, Match::Standard(Candidate::Up))    => {what_todo = Interact {dir:0};},
                        (_, Match::Standard(Candidate::Down))  => {what_todo = Interact {dir:1};},
                        (_, Match::Standard(Candidate::Left))  => {what_todo = Interact {dir:2};},
                        (_, Match::Standard(Candidate::Right)) => {what_todo = Interact {dir:3};},
                        _ => return Err(weird_key(data))
                    }
                },
                _ => return Err(weird_key(data))
            };
            match what_todo {
                Move {dir} => {
                    if let Some(to) = data.travel(here.1, dir) {
                        if move_entity(id, to, data) {
                            break;
                        }
                        if interact(id, to.tile, data) {
                            break;
                        }
                    }
                },
                Interact {dir} => {
                    if let Some(to) = data.travel(here.1, dir) {
                        if interact(id, to.tile, data) {
                            break;
                        }
                    }
                },
                Nothing => break,
            };
        }
        
        //
        redraw_los(id, (widget, data.ui.area_size(widget).ok_or(Interrupt::AbortError)?), data)?;
        Ok(())
    }
}
}
