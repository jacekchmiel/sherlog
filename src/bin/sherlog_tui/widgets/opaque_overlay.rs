use tui::layout::Rect;
use tui::widgets::{StatefulWidget, Widget};

use crate::ty::{Cursor, RenderCursor};

pub struct OpaqueOverlay<T>(pub T);

impl<T> OpaqueOverlay<T> {
    fn clear(&self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) -> Rect {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf.get_mut(x, y).reset();
            }
        }

        area
    }
}

impl<T: Widget> Widget for OpaqueOverlay<T> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        self.clear(area, buf);
        self.0.render(area, buf);
    }
}

impl<T: StatefulWidget> StatefulWidget for OpaqueOverlay<T> {
    type State = T::State;

    fn render(
        self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        self.clear(area, buf);
        self.0.render(area, buf, state);
    }
}

impl<T: RenderCursor> RenderCursor for OpaqueOverlay<T> {
    fn cursor(&self, area: Rect) -> Option<Cursor> {
        self.0.cursor(area)
    }
}
