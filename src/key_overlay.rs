use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px, rgb};

/// Each tuple is `(top_px, height_px)` relative to the square's top edge.
/// top_px can be negative (bar extending above the square, clipped by overflow_hidden).
#[derive(IntoElement)]
pub struct KeyOverlay {
    label: &'static str,
    bars: Vec<(f32, f32)>,
}

impl KeyOverlay {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            bars: vec![],
        }
    }

    pub fn bars(mut self, bars: Vec<(f32, f32)>) -> Self {
        self.bars = bars;
        self
    }
}

impl RenderOnce for KeyOverlay {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let square_size = 64.0_f32;

        let mut container = div()
            .size(px(square_size))
            .border_1()
            .border_color(rgb(0xffffff))
            .relative()
            .flex()
            .justify_center()
            .items_center();

        for (top, height) in self.bars {
            container = container.child(
                div()
                    .absolute()
                    .left_0()
                    .w_full()
                    .top(px(top))
                    .h(px(height))
                    .bg(rgb(0x808080)),
            );
        }

        // Label sits on top of all bars
        container.child(div().relative().child(self.label))
    }
}
