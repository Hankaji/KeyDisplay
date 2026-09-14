mod config;
mod key_observer;
mod key_overlay;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use config::Config;
use gpui::{
    App, Application, AsyncApp, Context, Hsla, IntoElement, ParentElement, Styled, Task,
    WeakEntity, Window, WindowOptions, div, linear_color_stop, linear_gradient, prelude::*, px,
    rgb,
};
use key_overlay::KeyOverlay;

const SQUARE_SIZE: f32 = 64.0;
const SPEED: f32 = 1000.0;

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
    held_keys: HashMap<String, Instant>,
    floating_bars: HashMap<String, Vec<FloatingBar>>,
    _animation_task: Option<Task<()>>,
    _key_observer_task: Option<Task<()>>,
    window_height: f32,
    config: Config,
}

impl HelloWorld {
    fn new(cx: &mut Context<Self>, key_observer: key_observer::KeyObserver) -> Self {
        let mut this = Self {
            held_keys: HashMap::new(),
            floating_bars: HashMap::new(),
            _animation_task: None,
            _key_observer_task: None,
            window_height: 1080.0,
            config: Config::load(),
        };
        this._key_observer_task = Some(Self::start_key_observer(cx, key_observer));
        this
    }

    fn start_key_observer(cx: &mut Context<Self>, observer: key_observer::KeyObserver) -> Task<()> {
        // Offload the blocking recv() to a dedicated OS thread.
        // The async task drains the channel non-blockingly every frame.
        let (tx, rx) = std::sync::mpsc::channel::<key_observer::KeyEvent>();
        std::thread::spawn(move || {
            while let Some(ev) = observer.recv() {
                if tx.send(ev).is_err() {
                    break; // view dropped, stop forwarding
                }
            }
        });

        cx.spawn(
            async move |view: WeakEntity<HelloWorld>, cx: &mut AsyncApp| {
                loop {
                    cx.background_executor()
                        .timer(Duration::from_millis(16))
                        .await;

                    let still_alive = view.update(cx, |this, cx| {
                        for ev in rx.try_iter() {
                            let key_name = key_observer::key_code_to_name(ev.key);
                            match ev.state {
                                key_observer::KeyState::Down | key_observer::KeyState::Repeat => {
                                    this.press_key(key_name, cx);
                                }
                                key_observer::KeyState::Up => {
                                    this.release_key(&key_name, cx);
                                }
                            }
                        }
                    });

                    if still_alive.is_err() {
                        break;
                    }
                }
            },
        )
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
        self.held_keys.insert(key, Instant::now());
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

impl Render for HelloWorld {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.window_height = window.viewport_size().height.into();

        let overlays: Vec<KeyOverlay> = self
            .config
            .keys
            .iter()
            .map(|kc| {
                let key_lower = kc.key.to_lowercase();
                let is_pressed = self.held_keys.contains_key(&key_lower);

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

        let bg_opaque = Hsla::from(self.config.bg_color);
        let bg_fade = Hsla {
            a: 0.0,
            ..bg_opaque
        };

        div()
            .flex()
            .flex_col()
            .relative()
            .bg(self.config.bg_color)
            .size_full()
            .justify_end()
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
            .child(
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
                    )),
            )
    }
}

fn main() {
    let Some(key_observer) = key_observer::KeyObserver::spawn() else {
        eprintln!("Failed to start key observer – check /dev/input permissions.");
        return;
    };

    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_window, cx| {
            cx.new(|cx| HelloWorld::new(cx, key_observer))
        })
        .unwrap();
    });
}
