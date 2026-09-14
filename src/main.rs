mod config;
mod key_overlay;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use config::Config;
use gpui::{
    App, Application, AsyncApp, Context, FocusHandle, IntoElement, KeyDownEvent, KeyUpEvent,
    ParentElement, Styled, Task, WeakEntity, Window, WindowOptions, div, prelude::*, rgb,
};
use key_overlay::KeyOverlay;

const SQUARE_SIZE: f32 = 64.0;
const SPEED: f32 = 600.0;

struct KeyBar {
    press_time: Instant,
    release_time: Option<Instant>,
}

impl KeyBar {
    fn new() -> Self {
        Self {
            press_time: Instant::now(),
            release_time: None,
        }
    }

    fn held_secs(&self) -> f32 {
        match self.release_time {
            None => self.press_time.elapsed().as_secs_f32(),
            Some(t) => t.duration_since(self.press_time).as_secs_f32(),
        }
    }

    fn float_secs(&self) -> f32 {
        match self.release_time {
            None => 0.0,
            Some(t) => t.elapsed().as_secs_f32(),
        }
    }

    /// Bar geometry as (top_px, height_px) relative to the square's top edge.
    ///
    /// Bottom is anchored at the square's top edge (0) while held, then floats up.
    /// Height starts at 0 and grows at SPEED px/s while the key is held.
    ///
    ///   height = held_secs * SPEED
    ///   top    = -(held_secs + float_secs) * SPEED   ← always moves at SPEED
    ///   bottom = top + height = -float_secs * SPEED   ← 0 while held, rises after
    fn geometry(&self) -> (f32, f32) {
        let held = self.held_secs();
        let floating = self.float_secs();
        let height = held * SPEED;
        let top = -(held + floating) * SPEED;
        (top, height)
    }

    fn is_offscreen(&self, window_height: f32) -> bool {
        self.release_time.is_some() && self.float_secs() * SPEED >= window_height - SQUARE_SIZE
    }
}

struct HelloWorld {
    focus_handle: FocusHandle,
    active_bars: HashMap<String, Vec<KeyBar>>,
    /// Kept alive so the animation loop isn't cancelled.
    _animation_task: Option<Task<()>>,
    window_height: f32,
    config: Config,
}

impl HelloWorld {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            active_bars: HashMap::new(),
            _animation_task: None,
            window_height: 1080.0,
            config: Config::load(),
        }
    }

    fn start_animation(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(
            async move |view: WeakEntity<HelloWorld>, cx: &mut AsyncApp| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(16))
                        .await;

                    let still_running = view
                        .update(cx, |this, cx| {
                            let wh = this.window_height;
                            this.active_bars.retain(|_, bars| {
                                bars.retain(|b| !b.is_offscreen(wh));
                                !bars.is_empty()
                            });

                            if this.active_bars.is_empty() {
                                false
                            } else {
                                cx.notify();
                                true
                            }
                        })
                        .unwrap_or(false);

                    if !still_running {
                        break;
                    }
                }
            },
        )
    }
}

impl Render for HelloWorld {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.window_height = window.viewport_size().height.into();

        let is_a_pressed = self
            .active_bars
            .get("a")
            .map(|bs| bs.iter().any(|b| b.release_time.is_none()))
            .unwrap_or(false);

        let bars_a: Vec<(f32, f32)> = self
            .active_bars
            .get("a")
            .map(|bs| bs.iter().map(|b| b.geometry()).collect())
            .unwrap_or_default();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let key = event.keystroke.key.clone();
                let bars = this.active_bars.entry(key).or_default();

                if !bars.iter().any(|b| b.release_time.is_none()) {
                    bars.push(KeyBar::new());
                    this._animation_task = Some(Self::start_animation(cx));
                }
            }))
            .on_key_up(cx.listener(|this, event: &KeyUpEvent, _window, cx| {
                let key = &event.keystroke.key;
                if let Some(bars) = this.active_bars.get_mut(key)
                    && let Some(bar) = bars.iter_mut().find(|b| b.release_time.is_none())
                {
                    bar.release_time = Some(Instant::now());
                }
                cx.notify();
            }))
            .flex()
            .flex_col()
            .bg(self.config.bg_color)
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
                    .child(KeyOverlay::new("A").bars(bars_a).pressed(is_a_pressed)),
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
