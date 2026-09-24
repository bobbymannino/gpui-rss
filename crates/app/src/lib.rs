mod sidebar;
mod workspace;

use gpui_kit::QuitMode;
use gpui_kit::WindowOptions;
use gpui_kit::component::Root;
use gpui_kit::prelude::*;

use crate::workspace::Workspace;

/// Run the application. This is the root most function that sets up the app and runs it.
pub fn run() {
    let app = gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .with_quit_mode(QuitMode::LastWindowClosed);

    app.run(move |cx| {
        gpui_kit::init(cx);
        cx.activate(true);

        let open = cx.spawn(async move |cx| {
            cx.open_window(WindowOptions::default(), |window, cx| {
                window.activate_window();
                let view = cx.new(Workspace::new);
                cx.new(|cx| Root::new(view, window, cx))
            })?;

            anyhow::Ok(())
        });

        open.detach_and_log_err(cx);
    });
}
