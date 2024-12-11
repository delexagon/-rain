use super::widget_package::*;
use crate::common::{Array2D,UITile,BLANKTILE,FILLEDTILE};

/// Calculates the length of a string with a width drawn inside
fn calc_start(align: f64, outer_width: u16, inner_width: u16) -> i32 {
    let inner_alignment_point = (align*(inner_width as f64).round()) as i32;
    let outer_alignment_point = (align*(outer_width as f64).round()) as i32;
    let inner_alignment_offset = outer_alignment_point - inner_alignment_point;
    return inner_alignment_offset;
}

#[derive(Deserialize,Serialize)]
pub struct CharArea {
    chars: Array2D<UITile>,
    // Of the inner chars
    row_alignment: f64,
    col_alignment: f64,
    default_ch: char,
}


impl CharArea {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            chars: Array2D::new_sized(width, height, FILLEDTILE),
            row_alignment: 0.5,
            col_alignment: 0.5,
            default_ch: ' ',
        }
    }
    
    pub fn dim(&self) -> (usize,usize) {self.chars.dim()}

    pub fn set(&mut self, ch: Array2D<UITile>) {
        if self.chars.dim() == ch.dim() {
            self.chars = ch;
        }
    }
}

impl Widget for CharArea {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}
    
    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let bound = buffer.bound();
        let area_y_offset_from_window = calc_start(self.row_alignment, bound.height, self.chars.height() as u16);
        let area_x_offset_from_window = calc_start(self.col_alignment, bound.width, self.chars.width() as u16);
        for row in 0..bound.height {
            let inner_row = row as i32 - area_y_offset_from_window;
            for col in 0..bound.width {
                let inner_col = col as i32 - area_x_offset_from_window;
                if inner_row < 0 || inner_col < 0 ||
                   inner_col >= self.chars.width() as i32 || inner_row >= self.chars.height() as i32 {
                    buffer.wtile(&BLANKTILE);
                } else {
                    buffer.wtile(
                        &self.chars[(inner_col as usize, inner_row as usize)]);
                }
            }
            buffer.next_line();
        }
    }

    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::Nothing
    }
}
