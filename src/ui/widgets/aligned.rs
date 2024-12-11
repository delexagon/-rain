use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct Aligned {
    pub size: WidgetBound,
    pub alignment: (f64, f64)
}

impl Aligned {
    pub fn new(size: WidgetBound, alignment: (f64,f64)) -> Self {
        Self {
            size,
            alignment
        }
    }
}

impl Widget for Aligned {
    fn child_number(&mut self, desired: usize) -> usize {1}
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {
        vec![WidgetBound {
            width: self.size.width,
            height: self.size.height
        }]
    }

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        buffer.clear();
        let bound = buffer.bound();
        let start_x = inner_cut(bound.width, self.size.width, self.alignment.0);
        let start_y = inner_cut(bound.height, self.size.height, self.alignment.1);

        children[0].copy_to(
            ((start_x,start_y),(0,0)),
            WidgetBound {
                width: self.size.width,
                height: self.size.height
            }, buffer
        );
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::PassToChild(0)
    }
}
