use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px, rgb};

#[derive(IntoElement)]
pub struct KeyOverlay {
    label: &'static str,
}

impl KeyOverlay {
    pub fn new(label: &'static str) -> Self {
        Self { label }
    }
}

impl RenderOnce for KeyOverlay {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .size(px(64.))
            .border_2()
            .border_color(rgb(0xffffff))
            .flex()
            .justify_center()
            .items_center()
            .child(self.label)
    }
}
