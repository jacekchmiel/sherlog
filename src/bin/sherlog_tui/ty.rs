use crossterm::event::{KeyEvent, MouseEvent};

pub(crate) trait RenderWithState<'a> {
    type Widget: tui::widgets::StatefulWidget;

    fn widget(
        &'a mut self,
    ) -> (
        Self::Widget,
        &mut <Self::Widget as tui::widgets::StatefulWidget>::State,
    );
}

pub(crate) trait Render<'a> {
    type Widget: tui::widgets::Widget;
    fn widget(&'a self) -> Self::Widget;
}

pub(crate) trait RenderCursor {
    fn cursor(&self, area: tui::layout::Rect) -> Option<Cursor>;
}

pub(crate) trait React<'a> {
    type Reaction;

    /// Returns None if event was not consumed, a Reaction otherwise
    fn on_key(&'a mut self, _key: KeyEvent) -> Self::Reaction;

    /// Returns None if event was not consumed, a Reaction otherwise
    fn on_mouse(&'a mut self, _mouse: MouseEvent) -> Self::Reaction;
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Cursor {
    pub x: u16,
    pub y: u16,
}
impl Cursor {
    pub fn new(x: u16, y: u16) -> Self {
        Cursor { x, y }
    }

    pub fn inside(self, area: tui::layout::Rect) -> Self {
        Cursor {
            x: self.x + area.x,
            y: self.y + area.y,
        }
    }
}
