use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub enum Size {
    Exact(u16),
    Minimum(u16),
    Maximum(u16)
}

#[derive(Deserialize,Serialize)]
pub struct Sized {
    pub width: Size,
    pub height: Size,
}

impl Widget for Sized {
    fn child_number(&mut self, desired: usize) -> usize {1}
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {
        let width = match self.width {
            Size::Exact(x) => x,
            Size::Minimum(x) => x.max(bound.width),
            Size::Maximum(x) => x.min(bound.width),
        };
        let height = match self.height {
            Size::Exact(x) => x,
            Size::Minimum(x) => x.max(bound.height),
            Size::Maximum(x) => x.min(bound.height),
        };
        vec![WidgetBound {width,height}]
    }

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let width = match self.width {
            Size::Exact(x) => x,
            Size::Minimum(x) => x.max(buffer.width()),
            Size::Maximum(x) => x.min(buffer.width()),
        };
        let height = match self.height {
            Size::Exact(x) => x,
            Size::Minimum(x) => x.max(buffer.height()),
            Size::Maximum(x) => x.min(buffer.height()),
        };
        let bound = WidgetBound {
            width,
            height
        };
        
        children[0].copy_to(((0,0),(0,0)), bound, buffer);
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::PassToChild(0)
    }
}
