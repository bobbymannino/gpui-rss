use gpui_kit::Window;
use gpui_kit::base::Side;
use gpui_kit::component::sidebar::Sidebar as SidebarElement;
use gpui_kit::component::sidebar::SidebarGroup;
use gpui_kit::component::sidebar::SidebarMenu;
use gpui_kit::component::sidebar::SidebarMenuItem;
use gpui_kit::prelude::*;

/// The workspace's left hand navigation.
pub struct Sidebar {
    collapsed: bool,
}

impl Sidebar {
    pub const fn new() -> Self {
        Self { collapsed: false }
    }

    /// Collapse the sidebar to icon width, or expand it again.
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        cx.notify();
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        SidebarElement::new("sidebar")
            .side(Side::Left)
            .collapsed(self.collapsed)
            .child(
                SidebarGroup::new("Library")
                    .child(SidebarMenu::new().child(SidebarMenuItem::new("All").active(true))),
            )
    }
}
