use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct Border {
    // top, bottom, left, right
    bordered: [bool; 4],
    chars: [char; 8]
}

impl Default for Border {
    fn default() -> Self { 
        Self {
            bordered: [true,true,true,true],
            chars: ['▄','▀','▐','▌','▗','▖','▝','▘']
        }
    }
}

impl Border {
    pub const TOP: usize = 0;
    pub const BOTTOM: usize = 1;
    pub const LEFT: usize = 2;
    pub const RIGHT: usize = 3;
    pub const TOPLEFT: usize = 4;
    pub const TOPRIGHT: usize = 5;
    pub const BOTTOMLEFT: usize = 6;
    pub const BOTTOMRIGHT: usize = 7;
    
    pub fn without_side(mut self, side: usize) -> Self {
        self.bordered[side] = false;
        self
    }
    pub fn with_char(mut self, side: usize, ch: char) -> Self {
        self.chars[side] = ch;
        self
    }
}

impl Widget for Border {
    fn child_number(&mut self, desired: usize) -> usize {1}
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {
        if bound.width < 2 || bound.height < 2 {
            vec![WidgetBound {width:0,height:0}]
        } else {
            vec![
                WidgetBound {
                    width: bound.width - self.bordered[3] as u16 - self.bordered[2] as u16,
                    height: bound.height - self.bordered[1] as u16 - self.bordered[0] as u16
                }
            ]
        }
    }

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let bound = buffer.bound();
        let (width, height) = (bound.width as usize, bound.height as usize);
        if 2 > bound.width || 2 > bound.height {
            return;
        }
        for i in 1..width-1 {
            buffer.wchar_at((i,0), self.chars[0], Style::default());
            buffer.wchar_at((i,height-1), self.chars[1], Style::default());
        }
        for i in 1..height-1 {
            buffer.wchar_at((0,i), self.chars[2], Style::default());
            buffer.wchar_at((width-1,i), self.chars[3], Style::default());
        }
        buffer.wchar_at((0,0), self.chars[4], Style::default());
        buffer.wchar_at((width-1,0), self.chars[5], Style::default());
        buffer.wchar_at((0,height-1), self.chars[6], Style::default());
        buffer.wchar_at((width-1,height-1), self.chars[7], Style::default());


        children[0].copy_to(
            ((self.bordered[2] as u16,self.bordered[0] as u16),(0,0)),
            WidgetBound {
                width: bound.width - self.bordered[3] as u16 - self.bordered[2] as u16,
                height: bound.height - self.bordered[1] as u16 - self.bordered[0] as u16
            }, buffer
        );
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::PassToChild(0)
    }
}
