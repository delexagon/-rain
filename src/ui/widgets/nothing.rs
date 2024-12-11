use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct Nothing;

impl Widget for Nothing {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {}
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::Nothing
    }
}