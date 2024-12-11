use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct Padding {
    top: u16,
    left: u16,
    right: u16,
    bottom: u16
}

impl Padding {
    pub fn none() -> Self {
        Self {
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
        }
    }

    pub fn sides(horizontal: u16, vertical: u16) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }

    pub fn horizontal(amt: u16) -> Self {
        Self {
            top: 0,
            bottom: 0,
            left: amt,
            right: amt,
        }
    }

    pub fn vertical(amt: u16) -> Self {
        Self {
            top: amt,
            bottom: amt,
            left: 0, 
            right: 0,
        }
    }
}

impl Widget for Padding {
    fn child_number(&mut self, desired: usize) -> usize {1}
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {
        if self.left+self.right >= bound.width || self.top+self.bottom >= bound.height {
            vec![WidgetBound {width:0,height:0}]
        } else {
            vec![
                WidgetBound {
                    width: bound.width-self.left-self.right,
                    height: bound.height-self.top-self.bottom,
                }
            ]
        }
    }

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let bound = buffer.bound();
        if self.left+self.right >= bound.width || self.top+self.bottom >= bound.height {
            return;
        }

        children[0].copy_to(
            ((self.left,self.top),(0,0)),
            WidgetBound {
                width: bound.width-self.left-self.right,
                height: bound.height-self.top-self.bottom,
            }, buffer
        );
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::PassToChild(0)
    }
}
