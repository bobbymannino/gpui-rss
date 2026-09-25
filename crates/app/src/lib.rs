mod sidebar;
mod workspace;

use gpui_kit::App;
use gpui_kit::Bounds;
use gpui_kit::Pixels;
use gpui_kit::QuitMode;
use gpui_kit::WindowBounds;
use gpui_kit::WindowOptions;
use gpui_kit::component::Root;
use gpui_kit::prelude::*;
use gpui_kit::px;
use gpui_kit::size;

use crate::workspace::Workspace;

/// Width to height ratio of the initial window.
const OPENING_WINDOW_ASPECT_RATIO: f32 = 1.4;
/// How much of the display the initial window takes up.
const OPENING_WINDOW_DISPLAY_FRACTION: f32 = 0.65;
/// The narrowest the initial window may open at.
const OPENING_WINDOW_MIN_WIDTH: f32 = 900.0;
/// The widest the initial window may open at.
const OPENING_WINDOW_MAX_WIDTH: f32 = 1400.0;

/// The bounds the first window opens at: a [`WINDOW_ASPECT_RATIO`] rectangle
/// scaled to the primary display, clamped to a min/max width, and centered.
fn initial_window_bounds(cx: &App) -> Bounds<Pixels> {
    let width = cx
        .primary_display()
        .map_or(OPENING_WINDOW_MIN_WIDTH, |display| {
            let display = display.bounds().size;
            let by_width = f32::from(display.width) * OPENING_WINDOW_DISPLAY_FRACTION;
            let by_height = f32::from(display.height) * OPENING_WINDOW_DISPLAY_FRACTION * OPENING_WINDOW_ASPECT_RATIO;

            by_width.min(by_height)
        })
        .clamp(OPENING_WINDOW_MIN_WIDTH, OPENING_WINDOW_MAX_WIDTH);

    Bounds::centered(None, size(px(width), px(width / OPENING_WINDOW_ASPECT_RATIO)), cx)
}

/// Run the application. This is the root most function that sets up the app and runs it.
pub fn run() {
    let app = gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .with_quit_mode(QuitMode::LastWindowClosed);

    app.run(move |cx| {
        gpui_kit::init(cx);
        cx.activate(true);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(initial_window_bounds(cx))),
            ..WindowOptions::default()
        };

        let open = cx.spawn(async move |cx| {
            cx.open_window(options, |window, cx| {
                window.activate_window();
                let view = cx.new(Workspace::new);
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            anyhow::Ok(())
        });

        open.detach_and_log_err(cx);
    });
}
