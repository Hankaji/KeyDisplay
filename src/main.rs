mod key_overlay;

use gpui::{
    App, Application, Context, IntoElement, ParentElement, Styled, Window, WindowOptions, div,
    prelude::*, rgb,
};
use key_overlay::KeyOverlay;

struct HelloWorld;

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
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
                    .child(KeyOverlay::new("A"))
                    .child(KeyOverlay::new("Space")),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                ..Default::default()
            },
            |_, cx| cx.new(|_| HelloWorld),
        )
        .unwrap();
    });
}
