use gpui::{
    App, DefiniteLength, IntoElement, ParentElement, RenderOnce, Rgba, SharedString, Styled,
    Window, div, prelude::FluentBuilder, px, rgb,
};

/// Each tuple is `(top_px, height_px)` relative to the square's top edge.
/// top_px can be negative (bar extending above the square, clipped by overflow_hidden).
#[derive(IntoElement)]
pub struct KeyOverlay {
    label: SharedString,
    bars: Vec<(f32, f32)>,
    is_pressed: bool,
    width: DefiniteLength,
    bar_color: Rgba,
}

impl KeyOverlay {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            bars: vec![],
            is_pressed: false,
            width: px(64.0).into(),
            bar_color: rgb(0x808080),
        }
    }

    pub fn bars(mut self, bars: Vec<(f32, f32)>) -> Self {
        self.bars = bars;
        self
    }

    pub fn pressed(mut self, is_pressed: bool) -> Self {
        self.is_pressed = is_pressed;
        self
    }

    pub fn width(mut self, width: DefiniteLength) -> Self {
        self.width = width;
        self
    }

    pub fn bar_color(mut self, color: Rgba) -> Self {
        self.bar_color = color;
        self
    }
}

impl RenderOnce for KeyOverlay {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let square_size = 64.0_f32;
        let bar_color = self.bar_color;

        let mut container = div()
            .w(self.width)
            .h(px(square_size))
            .border_2()
            .border_color(rgb(0xffffff))
            .when(self.is_pressed, |div| div.bg(bar_color))
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
                    .bg(bar_color),
            );
        }

        container.child(div().relative().child(self.label))
    }
}
