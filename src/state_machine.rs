
use crate::ui::{Candidate, Match, Poll, UI};
use crate::ui::widgets::*;
use crate::filesystem::get_resources;
use crate::game::GameData;
use crate::common::{Style, Rgb, Id, ExtTree, ResourceHandler, transition_length, transition_default, RemoveVec, SoundManager, TakeBox};
use crate::{errstr, err};
use std::error::Error;

/********************************************************/
// START TYPES
/********************************************************/

pub fn normal_start(skip_intro: bool) -> Result<(), String> {
    let (paths, options, debug) = get_resources()?;
    let mut handler = ResourceHandler::new(paths, options, debug);
    handler.init();
    println!(crate::translate!(loading_text));
    let sound = SoundManager::new(handler.static_sounds(), &mut handler);
    let mut ui = UI::new(&mut handler);
    ui.new_context(ExtTree((false, Tabs::default().into()), vec![]));
    ui.new_context(ExtTree((false, Tabs::default().into()), vec![]));
    ui.set_context(0);
    err!(spin(handler, ui, sound, skip_intro))?;
    Ok(())
}

pub fn map_test(maps: &[&str]) -> Result<(), String> {
    let (paths, options, debug) = get_resources()?;
    let mut handler = ResourceHandler::new(paths, options, debug);
    handler.init();
    println!(crate::translate!(loading_text));
    let sound = SoundManager::new(handler.static_sounds(), &mut handler);
    let mut ui = UI::new(&mut handler);
    ui.new_context(ExtTree((false, Tabs::default().into()), vec![]));
    ui.new_context(ExtTree((false, Tabs::default().into()), vec![]));
    let mut game = GameData::from_map(maps, Box::new(handler), Box::new(ui), Box::new(sound));
    loop {
        let interrupt = game.next_update();
        match interrupt {
            Ok(()) => (),
            Err(Interrupt::AbortError) => (),
            // TODO: Consolidate this with the resource_handler error system
            Err(Interrupt::CriticalError(err)) => {
                return Err(errstr!(err));
            },
            Err(Interrupt::MainMenu) => break,
            Err(Interrupt::ForcedExit) => break,
        };
    }
    let (mut handler, mut ui, _sound) = game.take();
    ui.stop(&mut handler);
    Ok(())
}

pub fn widget_test<Func>(f: Func) -> Result<(), String> where 
Func: FnOnce(&mut UI, &mut ResourceHandler) -> Id {
    let (paths, options, debug) = get_resources()?;
    let mut handler = ResourceHandler::new(paths, options, debug);
    handler.init();
    let mut ui = UI::new(&mut handler);
    let base_widget = f(&mut ui, &mut handler);
    let poll = Poll::from([
        (base_widget, Candidate::Exit),
    ]);
    loop {
        match ui.poll_from(&poll, &mut handler) {
            (widget, Match::Standard(Candidate::Exit)) if widget == base_widget => break,
            _ => (),
        }
    }
    ui.stop(&mut handler);
    Ok(())
}

/********************************************************/
// MAIN LOOP
/********************************************************/

// These are cases in which control will be kicked back up into the state machine
pub enum Interrupt {
    CriticalError(Box<dyn Error>),
    AbortError,
    MainMenu, // 
    ForcedExit // In case of there being no additional events (for example, the player dies)
}

struct PersistentComponents {
    game: Option<GameData>,
    pub resources: TakeBox<ResourceHandler>,
    pub ui: TakeBox<UI>,
    pub sound: TakeBox<SoundManager>,
    game_has_components: bool
}

impl PersistentComponents {
    pub fn new(res: ResourceHandler, ui: UI, sound: SoundManager) -> Self {
        Self {
            game: None,
            resources: TakeBox::new(res),
            ui: TakeBox::new(ui),
            sound: TakeBox::new(sound),
            game_has_components: false
        }
    }

    pub fn game_new(&mut self) {
        if self.game.is_some() {
            self.game_destroy();
        }
        self.game = Some(GameData::from_the_beginning(self.resources.take(), self.ui.take(), self.sound.take()));
        self.game_has_components = true;
    }

    pub fn game_from_save(&mut self, save_name: &str) {
        if self.game.is_some() {
            self.game_destroy();
        }
        self.game = Some(GameData::from_save(uuid::Uuid::parse_str(save_name).unwrap(), self.resources.take(), self.ui.take(), self.sound.take()));
        self.game_has_components = true;
    }

    pub fn from_game(&mut self) {
        if self.game.is_none() || !self.game_has_components {return;}
        let (r,u,s) = self.game.as_mut().unwrap().take();
        self.resources.replace(r);
        self.ui.replace(u);
        self.sound.replace(s);
        self.game_has_components = false;
    }

    fn to_game(&mut self) {
        if self.game.is_none() || self.game_has_components {return;}
        self.game.as_mut().unwrap().replace(self.resources.take(), self.ui.take(), self.sound.take());
        self.game_has_components = true;
    }

    fn game_loop(&mut self) -> Result<bool, Box<dyn Error>> {
        if self.game.is_none() {
            return Ok(false);
        }
        if !self.game_has_components {
            self.to_game();
        }
        let game = self.game.as_mut().unwrap();
        game.restore_ui_context();
        let keep_this;
        loop {
            let interrupt = game.next_update();
            match interrupt {
                Ok(()) => (),
                Err(Interrupt::AbortError) => (),
                // TODO: Consolidate this with the resource_handler error system
                Err(Interrupt::CriticalError(err)) => {
                    return Err(err);
                },
                Err(Interrupt::MainMenu) => {
                    keep_this = true;
                    break;
                },
                Err(Interrupt::ForcedExit) => {
                    keep_this = false;
                    break;
                },
            };
        }
        Ok(keep_this)
    }

    pub fn to_main_menu_screen(&mut self) {
        if self.game_has_components {self.from_game();}
        self.ui.as_mut().set_context(0);
    }

    fn game_destroy(&mut self) {
        if self.game.is_none() {return;}
        if !self.game_has_components {self.to_game();}
        self.game.as_mut().unwrap().save_state();
        self.from_game();
        self.ui.as_mut().replace_context(1, ExtTree((false, Tabs::default().into()), vec![]));
        self.game = None;
    }
}

fn change_options(options_widgets: [Id; 3], ui: &mut UI, resources: &mut ResourceHandler, sound: &mut SoundManager) {
    let [options_base, volume, text_speed] = options_widgets;
    let poll = Poll::from([
        (volume, Candidate::Left),
        (volume, Candidate::Right),
        (text_speed, Candidate::Left),
        (text_speed, Candidate::Right),
        (options_base, Candidate::Exit),
    ]);
    let prev_selected = ui.widget::<Tabs>(ui.root()).unwrap().selected;
    ui.mut_widget::<Tabs>(ui.root()).unwrap().selected = ui.child_num(ui.root(), options_base);
    loop {
        match ui.poll_from(&poll, resources) {
            (widget, Match::Standard(Candidate::Left)) if widget == volume =>
            if let Some(mut widget) = ui.mut_widget::<ProgressBar>(volume) {
                if widget.amt > 5 {widget.amt -= 5;}
                else {widget.amt = 0;}
                sound.set_background_volume(widget.amt, resources, transition_default());
                resources.options.volume = widget.amt;
            },
            (widget, Match::Standard(Candidate::Right)) if widget == volume =>
            if let Some(mut widget) = ui.mut_widget::<ProgressBar>(volume) {
                if widget.amt < 250 {widget.amt += 5;}
                else {widget.amt = 255;}
                sound.set_background_volume(widget.amt, resources, transition_default());
                resources.options.volume = widget.amt;
            },
            (widget, Match::Standard(Candidate::Left)) if widget == text_speed =>
            if let Some(mut widget) = ui.mut_widget::<Choice>(text_speed) {
                widget.shift_left();
                resources.options.text_speed = widget.selected as u8;
            },
            (widget, Match::Standard(Candidate::Right)) if widget == text_speed => 
            if let Some(mut widget) = ui.mut_widget::<Choice>(text_speed) {
                widget.shift_right();
                resources.options.text_speed = widget.selected as u8;
            },
            (widget, Match::Standard(Candidate::Exit)) if widget == options_base => break,
            _ => resources.err(&errstr!(crate::translate!(options_bad_key)))
        }
    }
    resources.save_options();
    ui.mut_widget::<Tabs>(ui.root()).unwrap().selected = prev_selected;
}

pub fn spin(res: ResourceHandler, ui: UI, sound: SoundManager, skip_intro: bool) -> Result<(), Box<dyn Error>> {
    let mut components = PersistentComponents::new(res,ui,sound);
    let [menu_base, menu_choice] = add_main_menu(components.ui.as_mut(), components.resources.as_mut());
    let options_widgets = add_options_menu(components.ui.as_mut(), components.resources.as_ref());

    let menu_poll = Poll::from([
        (menu_choice, Candidate::Select),
    ]);

    components.from_game();
    components.sound.as_mut().background(&crate::translate!("rain"), components.resources.as_mut());
    
    // State loop (main menu->game->back and such, NOT framerate loop (in UI) or game loop
    // (in GameData))
    loop {
        components.from_game();
        match components.ui.as_mut().mut_widget::<TitleBottom>(menu_choice) {
            Some(mut widget) => {
                widget.set(0, match components.game {
                    Some(_) => crate::translate!("Continue").to_string(),
                    None => crate::translate!("Start").to_string()
                });
            }, _=>components.resources.as_mut().err(&errstr!(crate::translate!(main_menu_no_match)))
        };
        match components.ui.as_mut().poll_from(&menu_poll, components.resources.as_mut()) {
            (_, Match::Selection1D(0)) => {
                if components.game.is_none() {
                    if skip_intro {
                        components.game_new();
                    } else {
                        if intro_cinematic(components.ui.as_mut(), components.sound.as_mut(), components.resources.as_mut()) {
                            components.game_new();
                        } else {
                            continue;
                        }
                    }
                }
                let keep_game = components.game_loop()?;
                if !keep_game {
                   components.game_destroy();
                }
                components.to_main_menu_screen();
            },
            (_, Match::Selection1D(1)) => {
                let ui = components.ui.as_mut();
                let root = ui.root();
                let [load_screen] = ui.append_to::<Tabs>(root, 
                    ExtTree((true, load_screen(components.resources.as_mut())), vec![])
                )[..] else {panic!()};
                ui.mut_widget::<Tabs>(ui.root()).unwrap().selected = ui.child_num(ui.root(), load_screen);
                let pkey = ui.poll_from(&Poll::from([
                    (load_screen, Candidate::Select),
                    (load_screen, Candidate::Exit)
                ]), components.resources.as_mut());
                match pkey {
                    (_, Match::Selection1D(n)) => {
                        let widget = ui.widget::<Lines>(load_screen).unwrap();
                        let s = widget.get(n as usize).to_string();
                        components.game_from_save(&s);
                        components.game_loop()?;
                    }, (_, Match::Standard(Candidate::Exit)) => {
                    }, _=>()
                }
                let ui = components.ui.as_mut();
                ui.mut_widget::<Tabs>(ui.root()).unwrap().selected = ui.child_num(ui.root(), menu_base);
                ui.remove_child::<Tabs>(root, -1);
            },
            (_, Match::Selection1D(2)) => {
                change_options(options_widgets, components.ui.as_mut(), components.resources.as_mut(), components.sound.as_mut())
            },
            (_, Match::Selection1D(3)) => break,
            _ => components.resources.as_mut().err(&errstr!(crate::translate!(main_menu_bad_key))),
        }
    }
    
    components.game_destroy();

    // Cleanup
    components.from_game();
    components.ui.as_mut().stop(components.resources.as_mut());
    Ok(())
}

/********************************************************/
// COMPONENT GENERATION
/********************************************************/

/// Returns (id of base display window, id of lines widget that receives input)
fn add_main_menu(ui: &mut UI, resources: &mut ResourceHandler) -> [Id; 2] {
    let title_top = TitleTop::from_file(concat!(crate::translate!("title"),".txt"), (0.5, 0.3), resources);
    let (hash,step) = (title_top.hash(), title_top.step());
    let vars = ui.append_to::<Tabs>(ui.root(), 
        ExtTree((true, 
        Sized {
            width: Size::Minimum(40),
            height: Size::Minimum(20)
        }.into()),
        vec![
            ExtTree((false, 
            Split::new(
                true,
                false,
                SplitType::AbsBelow(9),
            ).into()),
            vec![
                ExtTree((false, title_top.into()), vec![]),
                ExtTree((true, 
                TitleBottom::from(vec!(
                    crate::translate!("Start").to_string(),
                    crate::translate!("Load").to_string(),
                    crate::translate!("Options").to_string(),
                    crate::translate!("Exit").to_string(),
                ), hash, step, 
                crate::translate!("city_background.txt"), resources).into()), vec![]),
            ])
        ])
    );
    let [base,input] = vars[..] else {panic!()};
    return [base,input];
}

pub fn load_screen(resources: &mut ResourceHandler) -> WidgetEnum {
    return Lines::from_vec(resources.list_saves().unwrap()).into();
}

fn add_options_menu(ui: &mut UI, resources: &ResourceHandler) -> [Id; 3] {
    let vars = ui.append_to::<Tabs>(ui.root(),
        ExtTree((true, LinesChildren::from_vec(vec![1, 1]).into()), vec![
            ExtTree((false,
            Split::new(
                false,
                false,
                SplitType::AbsBelow(30),
            ).into()), vec![
                ExtTree((false, Line {string: crate::translate!("Volume:").to_string()}.into()), vec![]),
                ExtTree((true, ProgressBar {amt: resources.options.volume}.into()), vec![])
            ]),
            ExtTree((false,
            Split::new(
                false,
                false,
                SplitType::AbsBelow(30),
            ).into()), vec![
                ExtTree((false, Line {string: crate::translate!("Text Speed:").to_string()}.into()), vec![]),
                ExtTree((true,
                Choice::from_vec(vec![
                    // If you add more of these, make sure to release the restriction in reading in options.rs
                    // I don't feel like fixing this rn
                    // TODO: fix this
                    crate::translate!("Slow").to_string(),
                    crate::translate!("Medium").to_string(),
                    crate::translate!("Fast").to_string(),
                    crate::translate!("Instant").to_string()
                ], resources.options.text_speed.into(), false).into()), vec![])
            ])
        ])
    );
    let [screen, volume, text_speed] = vars[..] else {panic!()};
    return [screen, volume, text_speed];
}

fn intro_cinematic(ui: &mut UI, sound: &mut SoundManager, resources: &mut ResourceHandler) -> bool {
    sound.background(crate::translate!("rain_inside"), resources);
    let base_volume = resources.options.volume;
    let [top, split,text,input] = ui.append_to::<Tabs>(ui.root(),
        ExtTree((true, Aligned::new(WidgetBound {width: 27, height: 3}, (0.5,0.5)).into()), vec![
            ExtTree((true, Split::new(true, false, SplitType::AbsBelow(1)).into()), vec![
                ExtTree((true, LineScroll::new(crate::translate!(intro_text_1), resources.options.text_speed as usize).finished().into()), vec![]),
                ExtTree((true, TextInput::new(20).into()), vec![])
            ])
        ])
    )[..] else {panic!()};
    let prev_selected = ui.widget::<Tabs>(ui.root()).unwrap().selected;
    ui.mut_widget::<Tabs>(ui.root()).unwrap().selected = ui.children(ui.root())-1;

    let input_send_or_return = Poll::from([
        (input, Candidate::Enter),
        (input, Candidate::Exit),
    ]);
    let input_send = Poll::from([
        (input, Candidate::Enter)
    ]);
    let text_finish = Poll::from([
        (text, Candidate::FinishAnimation)
    ]);
    loop {
        let result = ui.poll_from(&input_send_or_return, resources);
        if result == (input, Match::Standard(Candidate::Exit)) {
            sound.background(crate::translate!("rain"), resources);
            ui.mut_widget::<Tabs>(ui.root()).unwrap().selected = prev_selected;
            ui.remove_child::<Tabs>(ui.root(), -1);
            return false;
        }
        if ui.widget::<TextInput>(input).unwrap().string().len() > 0 {
            break;
        }
    }
    sound.set_background_volume(base_volume/6*5, resources, transition_length(1000));
    ui.mut_widget::<TextInput>(input).unwrap().set("");
    ui.mut_widget::<Split>(split).unwrap().set_active(true);
    ui.mut_widget::<LineScroll>(text).unwrap().change(crate::translate!(intro_text_2));
    ui.poll_from(&text_finish, resources);
    ui.mut_widget::<Split>(split).unwrap().set_active(false);
    loop {
        ui.poll_from(&input_send, resources);
        if ui.widget::<TextInput>(input).unwrap().string().len() > 0 {
            break;
        }
    }
    sound.set_background_volume(base_volume/6*4, resources, transition_length(1000));
    let input = ui.flat_replace(input, Nothing {}.into());
    let input_send = Poll::from([
        (input, Candidate::Enter)
    ]);
    ui.mut_widget::<LineScroll>(text).unwrap().change(crate::translate!(intro_text_3));
    sound.set_background_volume(base_volume/6*3, resources, transition_length(2000));
    ui.poll_from(&text_finish, resources);
    ui.poll_from(&input_send, resources);
    ui.mut_widget::<LineScroll>(text).unwrap().change(crate::translate!(intro_text_4));
    ui.poll_from(&text_finish, resources);
    ui.poll_from(&input_send, resources);
    sound.set_background_volume(0, resources, transition_default());
    let text2 = LineScroll::new(
        crate::translate!(intro_text_5), 
        resources.options.text_speed as usize).with_style(Style::from_fg(Rgb(255,0,0)).bold()
    );
    let len = text2.len();
    let split = ui.flat_replace(split, text2.into());
    ui.mut_widget::<Aligned>(top).unwrap().size = WidgetBound {width: len as u16, height: 1};
    let text = split;
    let text_finish = Poll::from([
        (text, Candidate::FinishAnimation)
    ]);
    let input_send = Poll::from([
        (text, Candidate::Enter)
    ]);
    ui.poll_from(&text_finish, resources);
    ui.poll_from(&input_send, resources);
    sound.set_background_volume(base_volume/6*3, resources, transition_default());
    ui.mut_widget::<Aligned>(top).unwrap().size = WidgetBound {width: 27, height: 3};
    {
    let mut widget = ui.mut_widget::<LineScroll>(text).unwrap();
    widget.set_style(Style::default());
    widget.change(crate::translate!(intro_text_6));
    }
    ui.poll_from(&text_finish, resources);
    ui.poll_from(&input_send, resources);
    ui.mut_widget::<Tabs>(ui.root()).unwrap().selected = prev_selected;
    ui.remove_child::<Tabs>(ui.root(), -1);
    true
}
