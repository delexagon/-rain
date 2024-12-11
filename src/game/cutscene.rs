
fn intro_cinematic(ui: &mut UI, sound: &mut SoundManager, resources: &mut ResourceHandler) -> bool {
    sound.background(crate::translate!("rain_inside"), resources);
    let base_volume = resources.options.volume;
    let [split,text,input] = ui.append_to::<Tabs>(ui.root(),
        ExtTree((false, Aligned::new(WidgetBound {width: 27, height: 3}, (0.5,0.5)).into()), vec![
            ExtTree((true, Split::new(true, false, SplitType::AbsBelow(1)).into()), vec![
                ExtTree((true, LineScroll::new(crate::translate!(intro_text_1), resources.options.text_speed as usize).finished().into()), vec![]),
                ExtTree((true, TextInput::new(20).into()), vec![])
            ])
        ])
    )[..] else {panic!()};
    let top = ui.root();

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
    ui.remove_child::<Tabs>(ui.root(), -1);
    true
}
