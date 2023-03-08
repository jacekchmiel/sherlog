use tui::layout::Rect;
use tui::widgets::{List, StatefulWidget};

use crate::ty::{Cursor, RenderCursor};

pub(crate) struct ListWithCursor<'a> {
    pub list: List<'a>,
    //TODO: need to create an EditableList to properly handle cursor
    // This one will fail miserably if this list starts from bottom
    pub cursor: Option<Cursor>,
}

impl<'a> StatefulWidget for ListWithCursor<'a> {
    type State = <List<'a> as StatefulWidget>::State;

    fn render(
        self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        self.list.render(area, buf, state)
    }
}

impl RenderCursor for ListWithCursor<'_> {
    fn cursor(&self, area: tui::layout::Rect) -> Option<Cursor> {
        // Cannot query if the list inside has borders. For now we assume that it has.
        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };
        self.cursor.map(|c| c.inside(inner))
    }
}
