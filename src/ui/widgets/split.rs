use super::widget_package::*;

#[derive(Clone,Copy,Deserialize,Serialize)]
pub enum SplitType {
    AbsAbove(u16),
    AbsBelow(u16),
    PercentAbove(f64),
    /// Above expands to a certain 'min' size, then continues to expad.
    /// when below hits its 'max' size, above begins to increase again
    MinAboveMaxBelow(u16,u16),
    MaxAboveMinBelow(u16,u16)
}

#[derive(Deserialize,Serialize)]
pub struct Split {
    split: SplitType,

    // Determines which side keyboard strokes will be routed to
    above_active: bool,
    is_vertical: bool
}

impl Split {
    pub fn new(is_vertical: bool, above_active: bool, split: SplitType) -> Self {
        Self {
            above_active,
            split,
            is_vertical,
        }
    }

    pub fn set_active(&mut self, set_above_as_active: bool) {
        self.above_active = set_above_as_active;
    }

    fn bounds(&self, bound: WidgetBound) -> ((u16,u16), [WidgetBound; 2]) {
        if self.is_vertical {
            let height = calc_split(self.split, bound.height);
            ((0,height), 
            [
                WidgetBound {
                    width: bound.width,
                    height: height
                },
                WidgetBound {
                    width: bound.width,
                    height: bound.height-height
                }
            ])
        } else {
            let width = calc_split(self.split, bound.width);
            ((width,0), 
            [
                WidgetBound {
                    width: width,
                    height: bound.height
                },
                WidgetBound {
                    width: bound.width-width,
                    height: bound.height
                }
            ])
        }
    }
}

fn calc_split(split: SplitType, length: u16) -> u16 {
    match split {
        SplitType::AbsAbove(size) => {
            if size > length {
                length
            } else {
                size
            }
        },
        SplitType::AbsBelow(size) => {
            if size > length {
                0
            } else {
                length-size
            }
        },
        SplitType::PercentAbove(per) => {
            cut(length, per)
        },
        SplitType::MinAboveMaxBelow(min, max) => {
            if min > length {
                length
            } else if length > min+max {
                length-max
            } else {
                min
            }
        },
        SplitType::MaxAboveMinBelow(max, min) => {
            if min > length {
                0
            } else if length > min+max {
                max
            } else {
                length-min
            }
        }
    }
}

impl Widget for Split {
    fn child_number(&mut self,desired:usize) -> usize {2}

    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {
        Vec::from(self.bounds(bound).1)
    }
    
    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let bound = buffer.bound();
        let (second_position, [first_bound, second_bound]) = self.bounds(bound);
        
        children[0].copy_to(((0,0),(0,0)), first_bound, buffer);
        children[1].copy_to((second_position,(0,0)), second_bound, buffer);
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        if self.above_active {
            EventResult::PassToChild(0)
        } else {
            EventResult::PassToChild(1)
        }
    }
}
