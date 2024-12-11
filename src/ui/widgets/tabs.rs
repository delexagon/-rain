use super::widget_package::*;

/// Displays a list of child widgets vertically.
#[derive(Deserialize,Serialize,Default)]
pub struct Tabs {
    pub selected: usize,
    children: usize
}

impl AddChild for Tabs {
    fn add_child(&mut self, i: usize) -> bool {
        if i <= self.selected && self.children != 0 {
            self.selected += 1;
        }
        self.children += 1;
        true
    }
}

impl RemoveChild for Tabs {
    fn remove_child(&mut self, i: usize) -> bool {
        if i <= self.selected && self.selected != 0 {
            self.selected -= 1;
        }
        self.children -= 1;
        true
    }
}

impl Widget for Tabs {
    fn child_number(&mut self,desired:usize) -> usize {
        self.children = desired;
        desired
    }
    fn child_sizes(&self,bound:WidgetBound) -> Vec<WidgetBound> {
        let mut v = Vec::with_capacity(self.children);
        for i in 0..self.children {
            if i == self.selected {
                v.push(bound);
            } else {
                v.push(WidgetBound {width: 0, height: 0});
            }
        }
        return v;
    }

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let bound = buffer.bound();
        if children.len() <= self.selected {
            return;
        }
        children[self.selected].copy_to(((0,0),(0,0)),bound,buffer);
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        return EventResult::PassToChild(self.selected as i32);
    }
}
