use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, prelude::*, px, rgb};

#[derive(IntoElement)]
pub struct KeyOverlay {
    label: &'static str,
    is_pressed: bool,
}

impl KeyOverlay {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            is_pressed: false,
        }
    }

    pub fn pressed(mut self, is_pressed: bool) -> Self {
        self.is_pressed = is_pressed;
        self
    }
}

impl RenderOnce for KeyOverlay {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div()
            .size(px(64.))
            .border_1()
            .border_color(rgb(0xffffff))
            .relative()
            .flex()
            .justify_center()
            .items_center()
            // Gray fill layer — sits behind the label
            .when(self.is_pressed, |d| {
                d.child(div().absolute().inset_0().bg(rgb(0x808080)))
            })
            .child(self.label)
    }
}
