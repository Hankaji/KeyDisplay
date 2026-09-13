mod key_overlay;

use gpui::{
    App, Application, Context, FocusHandle, IntoElement, KeyDownEvent, KeyUpEvent, ParentElement,
    Styled, Window, WindowOptions, div, prelude::*, rgb,
};
use key_overlay::KeyOverlay;
use std::collections::HashSet;

struct HelloWorld {
    focus_handle: FocusHandle,
    pressed_keys: HashSet<String>,
}

impl HelloWorld {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            pressed_keys: HashSet::new(),
        }
    }

    fn is_pressed(&self, key: &str) -> bool {
        self.pressed_keys.contains(key)
    }
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                this.pressed_keys.insert(event.keystroke.key.clone());
                cx.notify();
            }))
            .on_key_up(cx.listener(|this, event: &KeyUpEvent, _window, cx| {
                this.pressed_keys.remove(&event.keystroke.key);
                cx.notify();
            }))
            .flex()
            .flex_col()
            .bg(rgb(0x505050))
            .size_full()
            .justify_end()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .w_full()
                    .gap_2()
                    .p_2()
                    .child(KeyOverlay::new("A").pressed(self.is_pressed("a"))),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                ..Default::default()
            },
            |window, cx| {
                cx.new(|cx| {
                    let view = HelloWorld::new(cx);
                    window.focus(&view.focus_handle);
                    view
                })
            },
        )
        .unwrap();
    });
}
