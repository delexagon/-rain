
mod ui;
mod common;
mod game;

use common::DataBox;

use ui::{UI, Widget, Action, Lines};
use game::{game_start};

use std::any::Any;

#[derive(Clone)]
enum ActionOut {
    Start,
    Exit,
    None,
} crate::to_action!(ActionOut);

fn main() {
    let ui = UI::new();
    let lines = Lines::<ActionOut>::new();
    {
        let mut write_lines = lines.write();
        write_lines.add_line(String::from("Start"));
        write_lines.add_line(String::from("Continue"));
        write_lines.add_line(String::from("Options"));
        write_lines.add_line(String::from("Exit"));
        
        fn select(_: DataBox<Lines<ActionOut>>, _: DataBox<UI>, line: usize) -> ActionOut {
            match line {
                0 => ActionOut::Start,
                1 => ActionOut::None,
                2 => ActionOut::None,
                3 => ActionOut::Exit,
                _ => ActionOut::None,
            }
        }
        write_lines.select_func = Some(select);
    }
    let menu_screen;
    { menu_screen = ui.write().add_widget(Box::new(lines)); }
    loop {
        match UI::get_action(ui.clone()).any().downcast_ref::<ActionOut>().unwrap() {
            ActionOut::Start => {
                game_start();
                UI::force_draw(ui.clone());
            },
            ActionOut::Exit => break,
            _ => (),
        }
    }
    ui.write().stop();
}

