use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct Line {
    pub string: String
}

impl Widget for Line {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}
    
    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let _ = buffer.wstr(&self.string, Style::default());
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::Nothing
    }
}
