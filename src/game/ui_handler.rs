use crate::common::{ResourceHandler, Id, ExtTree, UITile,TakeBox, NONETILE};
use crate::main;
use crate::ui::{UI, Poll, PollResult, Match, PollCandidate, Candidate};
use crate::ui::widgets::*;
use super::{EntityHandler, GameData, Traverser, UniqTile, MapHandler};
use serde::{Deserialize,Serialize};

pub const PLAYER_VIEW_SIZE: (usize,usize) = (60,60);

#[derive(Serialize,Deserialize)]
pub struct UIHandler {
    #[serde(skip)]
    pub writable: TakeBox<UI>,
    static_base: Id,
    log_screen: Id,
    pub main_character_view: Id,
}

impl UIHandler {
    pub fn new(mut ui: Box<UI>) -> Self {
        ui.set_context(1);
        Self {
            writable: TakeBox::newb(ui),
            static_base: Id::default(),
            main_character_view: Id::default(),
            log_screen: Id::default(),
        }
    }

    pub fn area_size(&self, widget: Id) -> Option<(usize,usize)> {
        let ui = self.writable.as_ref();
        Some(self.writable.as_ref().widget::<LOSArea>(widget).unwrap().dim())
    }

    pub fn update_los(id: Id, trav: Traverser, data: &mut GameData) {
        let mut ui = data.ui.writable.take();
        ui.mut_widget::<LOSArea>(id).unwrap().set(trav, data);
        data.ui.writable.replace(ui);
    }

    /*
    pub fn show_desc(trav: Traverser, data: &mut GameData) {
        if let WidgetEnum::
    }
    */

    pub fn log(&mut self, string: &str) {
        self.writable.as_mut().mut_widget::<LineScroll>(self.log_screen).unwrap().extend(string);
    }

    pub fn poll(&mut self, poll: &Poll, resources: &mut ResourceHandler) -> PollResult {
        let ui = self.writable.as_mut();
        return ui.poll_from(poll, resources);
    }

    pub fn initial_setup(&mut self) {
        let ui = self.writable.as_mut();
        let mut logs = LinesScroll::new(15);
        logs.push("You are the '@'.\nUse the arrow keys to move.");
        let vars = ui.replace_context(1, 
            ExtTree((false, Tabs::default().into()), vec![
            ExtTree((true,
            Sized {
                width: Size::Minimum(40),
                height: Size::Maximum(PLAYER_VIEW_SIZE.1 as u16),
            }.into()), vec![
                ExtTree((false,
                Split::new(
                    false, true,
                    SplitType::MaxAboveMinBelow(PLAYER_VIEW_SIZE.0 as u16, 30)).into()), vec![
                    ExtTree((false,
                    Border::default()
                        .with_char(Border::TOPRIGHT, '▄')
                        .with_char(Border::BOTTOMRIGHT, '▀').into()), vec![
                        ExtTree((true,
                            LOSArea::new(
                                PLAYER_VIEW_SIZE.0, PLAYER_VIEW_SIZE.1
                            ).into()
                        ), vec![])
                    ]),
                    ExtTree((false,
                    Border::default().without_side(Border::LEFT)
                        .with_char(Border::TOPLEFT, '▄')
                        .with_char(Border::BOTTOMLEFT, '▀').into()), vec![
                        ExtTree((true, logs.into()), vec![])
                    ])
                ])
            ]
        )]));
        let [static_base,main_character_view,log_screen] = vars[..] else {panic!()};
        self.static_base = static_base;
        self.main_character_view = main_character_view;
        self.log_screen = log_screen;
    }
    
    pub fn ui_tile(&self, tile: UniqTile, world: &MapHandler, actors: &EntityHandler) -> UITile {
        let objects = world.objects_on(tile);
        if objects.len() == 0 {
            return world.background(tile).extract();
        }
        let mut style = NONETILE;
        for obj in objects.iter().rev() {
            style.mod_style(actors.template(obj.template_id).style);
        }
        style.mod_style(world.background(tile));
        style.extract()
    }
    
    pub fn exit_key(&self) -> PollCandidate {
        return (self.writable.as_ref().root(), Candidate::Exit);
    }
    pub fn exit_returned(&self) -> PollResult {
        return (self.writable.as_ref().root(), Match::Standard(Candidate::Exit));
    }
}
