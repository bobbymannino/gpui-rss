use crate::page::Page;
use gpui_kit::EventEmitter;
use gpui_kit::Window;
use gpui_kit::base::Side;
use gpui_kit::component::sidebar::Sidebar as SidebarElement;
use gpui_kit::component::sidebar::SidebarGroup;
use gpui_kit::component::sidebar::SidebarMenu;
use gpui_kit::component::sidebar::SidebarMenuItem;
use gpui_kit::prelude::*;

/// The workspace's left hand navigation. Emits the [`Page`] the user picks.
pub struct Sidebar {
    collapsed: bool,
    active: Page,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            collapsed: false,
            active: Page::default(),
        }
    }

    /// Collapse the sidebar to icon width, or expand it again.
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        cx.notify();
    }

    /// Mark `page` as active and tell subscribers about it.
    fn select(&mut self, page: Page, cx: &mut Context<Self>) {
        if self.active == page {
            return;
        }

        self.active = page;
        cx.emit(page);
        cx.notify();
    }

    fn item(&self, label: &'static str, page: Page, cx: &Context<Self>) -> SidebarMenuItem {
        let this = cx.entity();

        SidebarMenuItem::new(label)
            .active(self.active == page)
            .on_click(move |_, _, cx| this.update(cx, |this, cx| this.select(page, cx)))
    }
}

impl Default for Sidebar {
    fn default() -> Self {
        Self::new()
    }
}

impl EventEmitter<Page> for Sidebar {}

impl Render for Sidebar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        SidebarElement::new("sidebar")
            .side(Side::Left)
            .collapsed(self.collapsed)
            .child(SidebarGroup::new("Feeds").child(SidebarMenu::new().child(self.item("All", Page::AllFeeds, cx))))
            .child(
                SidebarGroup::new("Sources").child(
                    SidebarMenu::new()
                        .child(self.item("All", Page::AllSources, cx))
                        .child(self.item("New", Page::NewSource, cx)),
                ),
            )
    }
}
