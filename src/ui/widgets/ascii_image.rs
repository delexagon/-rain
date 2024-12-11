use super::widget_package::*;
use crate::{ResourceHandler, Array2D};

#[derive(Deserialize,Serialize)]
pub struct ASCIIImage {
    // TODO: this shouldn't have to ever be redrawn
    image: Array2D<char>
}

impl ASCIIImage {
    pub fn from_file(file: &str, res: &mut ResourceHandler) -> Self {
        match res.file_str(&res.path.misc.join(file)) {
            Some(x) => Self {image: Array2D::from_str(&x)},
            None => Self {image: Array2D::new()},
        }
    }
    pub fn size(&self) -> WidgetBound {
        return WidgetBound { width: self.image.width() as u16, height: self.image.height() as u16 }
    }
}

impl Widget for ASCIIImage {
    fn child_sizes(&self, bound: WidgetBound) -> Vec<WidgetBound> {Vec::with_capacity(0)}
    fn child_number(&mut self,desired:usize) -> usize {0}
    
    fn draw(&self, children: &mut [&mut WidgetData], buffer: &mut WidgetBuffer) {
        for y in 0..self.image.height() {
            for x in 0..self.image.width() {
                buffer.wchar_at((x,y), self.image[(x,y)], Style::default())
            }
        }
    }
    
    fn poll(&mut self, my_id: Id, event: Event, event_translation: Option<Candidate>, poll: &Poll) -> EventResult {
        EventResult::Nothing
    }
}
