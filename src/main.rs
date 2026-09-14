mod config;
mod key_overlay;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use config::Config;
use gpui::{
    App, Application, AsyncApp, Context, FocusHandle, IntoElement, KeyDownEvent, KeyUpEvent,
    Modifiers, ModifiersChangedEvent, ParentElement, Styled, Task, WeakEntity, Window,
    WindowOptions, div, prelude::*, rgb,
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
    prev_modifiers: Modifiers,
}

impl HelloWorld {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            active_bars: HashMap::new(),
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

    fn press_key(&mut self, key: String, cx: &mut Context<Self>) {
        let bars = self.active_bars.entry(key).or_default();
        if !bars.iter().any(|b| b.release_time.is_none()) {
            bars.push(KeyBar::new());
            self._animation_task = Some(Self::start_animation(cx));
        }
    }

    fn release_key(&mut self, key: &str, cx: &mut Context<Self>) {
        if let Some(bars) = self.active_bars.get_mut(key)
            && let Some(bar) = bars.iter_mut().find(|b| b.release_time.is_none())
        {
            bar.release_time = Some(Instant::now());
        }
        cx.notify();
    }
}

/// Maps each modifier bool field to a stable key name.
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
                let is_pressed = self
                    .active_bars
                    .get(&key_lower)
                    .map(|bs| bs.iter().any(|b| b.release_time.is_none()))
                    .unwrap_or(false);

                let bars = self
                    .active_bars
                    .get(&key_lower)
                    .map(|bs| bs.iter().map(|b| b.geometry()).collect())
                    .unwrap_or_default();

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
                eprintln!("key: {key:?}");
                this.press_key(key, cx);
            }))
            .on_key_up(cx.listener(|this, event: &KeyUpEvent, _window, cx| {
                this.release_key(&event.keystroke.key, cx);
            }))
            .on_modifiers_changed(cx.listener(|this, event: &ModifiersChangedEvent, _window, cx| {
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
                    .children(overlays),
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
