use gpui_kit::Entity;
use gpui_kit::Window;
use gpui_kit::base::StyledExt as _;
use gpui_kit::div;
use gpui_kit::prelude::*;

use crate::sidebar::Sidebar;

/// The top level view hosted by the window's `Root`. Everything the user sees
/// lives inside this.
pub struct Workspace {
    sidebar: Entity<Sidebar>,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            sidebar: cx.new(|_| Sidebar::new()),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_flex()
            .size_full()
            .child(self.sidebar.clone())
            .child(div().v_flex().flex_1())
    }
}
