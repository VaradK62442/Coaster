use std::io::Error;

use super::terminal::Size;

pub trait UIComponent {
    fn mark_redrawn(&mut self, value: bool);
    fn needs_redrawn(&self) -> bool;

    fn resize(&mut self, size: Size) {
        self.set_size(size);
        self.mark_redrawn(true);
    }

    fn set_size(&mut self, size: Size);

    fn render(&mut self, origin_y: usize) {
        if self.needs_redrawn() {
            match self.draw(origin_y) {
                Ok(()) => self.mark_redrawn(false),
                Err(err) => {
                    #[cfg(debug_assertions)]
                    {
                        panic!("Could not render component: {err:?}");
                    }
                }
            }
        }
    }

    fn draw(&mut self, origin_y: usize) -> Result<(), Error>;
}
