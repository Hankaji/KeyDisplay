mod key_overlay;

use std::collections::HashMap;
use std::time::{Duration, Instant};

use gpui::{
    App, Application, AsyncApp, Context, FocusHandle, IntoElement, KeyDownEvent, KeyUpEvent,
    ParentElement, Styled, Task, WeakEntity, Window, WindowOptions, div, prelude::*, rgb,
};
use key_overlay::KeyOverlay;

// ── Tuning constants ──────────────────────────────────────────────────────────
const SQUARE_SIZE: f32 = 64.0;
/// px per second the bar grows while the key is held
const PIXELS_PER_SEC: f32 = 300.0;
/// minimum bar height so even a quick tap is visible
const MIN_HEIGHT: f32 = 10.0;
/// px per second the bar floats upward after release
const FLOAT_SPEED: f32 = 200.0;

// ── KeyBar ────────────────────────────────────────────────────────────────────

struct KeyBar {
    press_time: Instant,
    /// None while the key is still held
    release_time: Option<Instant>,
}

impl KeyBar {
    fn new() -> Self {
        Self {
            press_time: Instant::now(),
            release_time: None,
        }
    }

    /// Height of the bar based on how long the key was (or is being) held.
    fn height(&self) -> f32 {
        let held = match self.release_time {
            None => self.press_time.elapsed(),
            Some(t) => t.duration_since(self.press_time),
        };
        (held.as_secs_f32() * PIXELS_PER_SEC).max(MIN_HEIGHT)
    }

    /// How far the bar has floated upward since release (0 while still held).
    fn float_offset(&self) -> f32 {
        match self.release_time {
            None => 0.0,
            Some(t) => t.elapsed().as_secs_f32() * FLOAT_SPEED,
        }
    }

    /// `top_px` is relative to the square's top edge; can be negative (above square).
    fn geometry(&self) -> (f32, f32) {
        let h = self.height();
        let offset = self.float_offset();
        // Bar bottom is at SQUARE_SIZE - offset; bar top is h pixels above that.
        let top = SQUARE_SIZE - h - offset;
        (top, h)
    }

    /// True once the bar's bottom edge has exited the top of the window.
    ///
    /// The bar's bottom in window-space = (window_height - SQUARE_SIZE) + (SQUARE_SIZE - float_offset)
    ///                                  = window_height - float_offset
    /// It exits when that value drops below 0, i.e. float_offset >= window_height.
    fn is_offscreen(&self, window_height: f32) -> bool {
        self.release_time.is_some() && self.float_offset() >= window_height
    }
}

// ── Root view ─────────────────────────────────────────────────────────────────

struct HelloWorld {
    focus_handle: FocusHandle,
    /// Per-key list of active bars (held + floating).
    active_bars: HashMap<String, Vec<KeyBar>>,
    /// Kept alive so the animation loop isn't cancelled.
    _animation_task: Option<Task<()>>,
    /// Updated every render; used by the animation task for precise cleanup.
    window_height: f32,
}

impl HelloWorld {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            active_bars: HashMap::new(),
            _animation_task: None,
            window_height: 1080.0, // reasonable default before first render
        }
    }

    fn start_animation(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |view: WeakEntity<HelloWorld>, cx: &mut AsyncApp| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(16))
                    .await;

                let still_running = view
                    .update(cx, |this, cx| {
                        let wh = this.window_height;
                        // Drop bars whose bottom edge has exited the top of the window
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
        })
    }
}

impl Render for HelloWorld {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Keep window_height in sync so the animation task can use it
        self.window_height = window.viewport_size().height.into();

        // Pre-compute bar geometries before borrowing cx for listeners
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

                // Ignore OS key-repeat: only add a bar on the initial press
                if !bars.iter().any(|b| b.release_time.is_none()) {
                    bars.push(KeyBar::new());
                    this._animation_task = Some(Self::start_animation(cx));
                }
            }))
            .on_key_up(cx.listener(|this, event: &KeyUpEvent, _window, cx| {
                let key = &event.keystroke.key;
                if let Some(bars) = this.active_bars.get_mut(key) {
                    if let Some(bar) = bars.iter_mut().find(|b| b.release_time.is_none()) {
                        bar.release_time = Some(Instant::now());
                    }
                }
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
                    .child(KeyOverlay::new("A").bars(bars_a)),
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
