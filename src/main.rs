mod state_machine;

fn main() {
    if let Some(_) = option_env!("misctest") {
        // For testing miscellaneous functionality.
        //let args: Vec<u16> = std::env::args().skip(1).map(|x| x.parse().unwrap()).collect();
    } else if let Some(_) = option_env!("widgettest") {
        // For testing the functionality of widgets.
        // Please place the relevant UI manipulation code below.
        if let Err(e) = state_machine::widget_test(|ui, _res| {
            use crate::ui::widgets::*;
            let (_,v) = ui.new_context(ExtTree((true, {
                let mut l = LinesScroll::new(10); 
                l.push("holy fucking shit that was a little overkill but what do you expect from THE BEST??!?!?!?!?!?!"); l.into()}), vec![]));
            let [widget] = v[..] else {panic!()};
            return widget;
        }) {println!(crate::translate!(start_err), e)}
    } else if let Some(_) = option_env!("maptest") {
        // For testing the appearance and functionality of maps.
        // This will find the first available square in the map to place
        // the player on.
        let args: Vec<String> = std::env::args().skip(1).collect();
        let args2: Vec<&str> = args.iter().map(|arg| arg.as_str()).collect();
        if let Err(e) = state_machine::map_test(&args2) {
            println!(crate::translate!(start_err), e)
        }
    } else {
        let skip;
        if let Some(_) = option_env!("skipintro") {
            skip = true;
        } else {
            skip = false;
        }
        if let Err(e) = state_machine::normal_start(skip) {println!(crate::translate!(start_err), e)}
    }
}

mod ui;
mod common;
mod game;
mod filesystem;
mod translation_english;
//mod traducción_español;
pub use common::*;
