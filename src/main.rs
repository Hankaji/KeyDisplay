mod config;
mod key_overlay;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use config::Config;
use gpui::{
    App, Application, AsyncApp, Context, FocusHandle, Hsla, IntoElement, KeyDownEvent, KeyUpEvent,
    Modifiers, ModifiersChangedEvent, ParentElement, Styled, Task, WeakEntity, Window,
    WindowOptions, div, linear_color_stop, linear_gradient, prelude::*, px, rgb,
};
use key_overlay::KeyOverlay;

const SQUARE_SIZE: f32 = 64.0;
const SPEED: f32 = 600.0;
const KEY_REPEAT_WINDOW: Duration = Duration::from_millis(100);

/// A bar that has been released and is animating upward off screen.
struct FloatingBar {
    press_time: Instant,
    released_at: Instant,
}

impl FloatingBar {
    fn geometry(&self) -> (f32, f32) {
        let held = self
            .released_at
            .duration_since(self.press_time)
            .as_secs_f32();
        let floating = self.released_at.elapsed().as_secs_f32();
        let height = held * SPEED;
        let top = -(held + floating) * SPEED;
        (top, height)
    }

    fn is_offscreen(&self, window_height: f32) -> bool {
        self.released_at.elapsed().as_secs_f32() * SPEED >= window_height - SQUARE_SIZE
    }
}

struct HelloWorld {
    focus_handle: FocusHandle,
    /// Keys currently held down, mapped to the time they were pressed.
    held_keys: HashMap<String, Instant>,
    /// Bars that have been released and are floating upward.
    floating_bars: HashMap<String, Vec<FloatingBar>>,
    _animation_task: Option<Task<()>>,
    window_height: f32,
    config: Config,
    prev_modifiers: Modifiers,
}

impl HelloWorld {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            held_keys: HashMap::new(),
            floating_bars: HashMap::new(),
            _animation_task: None,
            window_height: 1080.0,
            config: Config::load(),
            prev_modifiers: Modifiers::default(),
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
                            this.floating_bars.retain(|_, bars| {
                                bars.retain(|b| !b.is_offscreen(wh));
                                !bars.is_empty()
                            });

                            if this.held_keys.is_empty() && this.floating_bars.is_empty() {
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

    fn press_key(&mut self, key: String, cx: &mut Context<Self>) {
        if self.held_keys.contains_key(&key) {
            return;
        }

        // On Linux the OS fires real KeyUp+KeyDown pairs for key repeat. If a bar
        // was released very recently, pull it back out of floating_bars and restore
        // its original press_time so the geometry continues seamlessly.
        let press_time = self
            .floating_bars
            .get_mut(&key)
            .and_then(|bars| {
                let pos = bars
                    .iter()
                    .rposition(|b| b.released_at.elapsed() < KEY_REPEAT_WINDOW)?;
                Some(bars.remove(pos).press_time)
            })
            .unwrap_or_else(Instant::now);

        self.held_keys.insert(key, press_time);
        self._animation_task = Some(Self::start_animation(cx));
    }

    fn release_key(&mut self, key: &str, cx: &mut Context<Self>) {
        if let Some(press_time) = self.held_keys.remove(key) {
            self.floating_bars
                .entry(key.to_string())
                .or_default()
                .push(FloatingBar {
                    press_time,
                    released_at: Instant::now(),
                });
        }
        cx.notify();
    }
}

const MODIFIERS: &[(&str, fn(&Modifiers) -> bool)] = &[
    ("shift", |m| m.shift),
    ("ctrl", |m| m.control),
    ("alt", |m| m.alt),
    ("super", |m| m.platform),
];

impl Render for HelloWorld {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.window_height = window.viewport_size().height.into();

        let overlays: Vec<KeyOverlay> = self
            .config
            .keys
            .iter()
            .map(|kc| {
                let key_lower = kc.key.to_lowercase();

                let is_pressed = self.held_keys.contains_key(&key_lower)
                    || self
                        .floating_bars
                        .get(&key_lower)
                        .map(|bars| {
                            bars.iter()
                                .any(|b| b.released_at.elapsed() < KEY_REPEAT_WINDOW)
                        })
                        .unwrap_or(false);

                let mut bars: Vec<(f32, f32)> = self
                    .floating_bars
                    .get(&key_lower)
                    .map(|bs| bs.iter().map(|b| b.geometry()).collect())
                    .unwrap_or_default();

                if let Some(&press_time) = self.held_keys.get(&key_lower) {
                    let held = press_time.elapsed().as_secs_f32();
                    bars.push((-held * SPEED, held * SPEED));
                }

                KeyOverlay::new(kc.key.clone())
                    .bars(bars)
                    .pressed(is_pressed)
                    .width(kc.width.to_definite_length())
                    .bar_color(kc.bar_color)
            })
            .collect();

        div()
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let key = event.keystroke.key.clone();
                // eprintln!("key: {key:?}"); // TODO: Use for debug later
                this.press_key(key, cx);
            }))
            .on_key_up(cx.listener(|this, event: &KeyUpEvent, _window, cx| {
                this.release_key(&event.keystroke.key, cx);
            }))
            .on_modifiers_changed(cx.listener(
                |this, event: &ModifiersChangedEvent, _window, cx| {
                    let new = event.modifiers;
                    for (name, get) in MODIFIERS {
                        let was = get(&this.prev_modifiers);
                        let is = get(&new);
                        if !was && is {
                            eprintln!("modifier down: {name}");
                            this.press_key(name.to_string(), cx);
                        } else if was && !is {
                            this.release_key(name, cx);
                        }
                    }
                    this.prev_modifiers = new;
                },
            ))
            .flex()
            .flex_col()
            .relative()
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
                    .children(overlays),
            )
            .child({
                let bg_opaque = Hsla::from(self.config.bg_color);
                let bg_fade = Hsla {
                    a: 0.0,
                    ..bg_opaque
                };
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h(px(200.0))
                    .bg(linear_gradient(
                        180.0,
                        linear_color_stop(bg_opaque, 0.0),
                        linear_color_stop(bg_fade, 1.0),
                    ))
            })
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
