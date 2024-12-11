use crate::common::{Array2D, UITile, BLANKTILE, FILLEDTILE};
use crate::game::{GameData, Traverser, los_scan, transform_uitile};
use super::widget_package::*;

#[derive(Deserialize,Serialize)]
pub struct LOSArea {
    t_arr: Array2D<Option<Traverser>>,
    ui_arr: Array2D<UITile>
}

impl LOSArea {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            t_arr: Array2D::new_sized(width, height, None),
            ui_arr: Array2D::new_sized(width, height, BLANKTILE)
        }
    }

    pub fn set(&mut self, center: Traverser, data: &mut GameData) {
        los_scan(&mut self.t_arr, center,data);
        transform_uitile(&mut self.ui_arr, &self.t_arr, data);
    }
    
    pub fn dim(&self) -> (usize,usize) {self.t_arr.dim()}
}

impl Widget for LOSArea {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}

    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        let bound = buffer.bound();
        let (arr_width, arr_height) = self.ui_arr.dim();
        let (outer_width, outer_height) = (bound.width as usize, bound.height as usize);
        for row in 0..bound.height {
            let inner_row = if arr_height/2>=outer_height/2 {arr_height/2-outer_height/2+row as usize} else {arr_height};
            for col in 0..bound.width {
                let inner_col = if arr_width/2>=outer_width/2 {arr_width/2-outer_width/2+col as usize} else {arr_width};
                if inner_row >= arr_height || inner_col >= arr_width {
                    buffer.wtile_at((col as usize, row as usize), &BLANKTILE);
                } else {
                    buffer.wtile_at((col as usize, row as usize), &self.ui_arr[(inner_col, inner_row)]);
                }
            }
        }
    }

    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::Nothing
    }
}
