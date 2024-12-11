use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct ProgressBar {
    pub amt: u8
}

impl Widget for ProgressBar {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        buffer.clear();
        let bound = buffer.bound();
        let split_point = cut(bound.width, self.amt as f64/255.);
        for j in 0..bound.width {
            if j < split_point {
                buffer.wchar('█', Style::default());
            } else {
                buffer.wchar('░', Style::default());
            }
        }
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::Nothing
    }
}
