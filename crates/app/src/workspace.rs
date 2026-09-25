use crate::page::Page;
use crate::sidebar::Sidebar;
use gpui_kit::Entity;
use gpui_kit::Window;
use gpui_kit::base::StyledExt as _;
use gpui_kit::div;
use gpui_kit::prelude::*;

/// The top level view hosted by the window's `Root`. Everything the user sees
/// lives inside this.
pub struct Workspace {
    sidebar: Entity<Sidebar>,
    page: Page,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let sidebar = cx.new(|_| Sidebar::new());

        cx.subscribe(&sidebar, |this, _, page: &Page, cx| {
            this.page = *page;
            cx.notify();
        })
        .detach();

        Self {
            sidebar,
            page: Page::default(),
        }
    }
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_flex()
            .size_full()
            .child(self.sidebar.clone())
            .child(self.page.render())
    }
}
