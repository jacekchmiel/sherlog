use tui::layout::Rect;
use tui::style::Style;
use tui::widgets::{Block, BorderType, Borders, StatefulWidget, Widget};

pub struct OverlayBlock<T> {
    inner: T,
    border: Option<(BorderType, Style)>,
}

impl<T> OverlayBlock<T> {
    pub fn new(inner: T) -> Self {
        OverlayBlock {
            inner,
            border: None,
        }
    }

    pub fn border(self, border_type: BorderType, border_style: Style) -> Self {
        OverlayBlock {
            inner: self.inner,
            border: Some((border_type, border_style)),
        }
    }

    fn render_base(&self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) -> Rect {
        let content_area = if let Some((border_type, border_style)) = self.border {
            let block = Block::default()
                .border_type(border_type)
                .border_style(border_style)
                .borders(Borders::all());
            let content_area = block.inner(area);
            block.render(area, buf);

            content_area
        } else {
            area
        };

        // clear content area to mimic overlay
        for x in content_area.left()..content_area.right() {
            for y in content_area.top()..content_area.bottom() {
                buf.get_mut(x, y).reset();
            }
        }

        content_area
    }
}

impl<T: Widget> Widget for OverlayBlock<T> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let content_area = self.render_base(area, buf);
        self.inner.render(content_area, buf)
    }
}

impl<T: StatefulWidget> StatefulWidget for OverlayBlock<T> {
    type State = T::State;

    fn render(
        self,
        area: tui::layout::Rect,
        buf: &mut tui::buffer::Buffer,
        state: &mut Self::State,
    ) {
        let content_area = self.render_base(area, buf);
        self.inner.render(content_area, buf, state);
    }
}
