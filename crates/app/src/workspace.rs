use crate::all_sources::AllSources;
use crate::new_source::NewSource;
use crate::page::Page;
use crate::sidebar::Sidebar;
use gpui_kit::AnyElement;
use gpui_kit::Entity;
use gpui_kit::Window;
use gpui_kit::base::StyledExt as _;
use gpui_kit::div;
use gpui_kit::prelude::*;
use storage::JSONDatabase;

/// The top level view hosted by the window's `Root`. Everything the user sees
/// lives inside this.
pub struct Workspace {
    sidebar: Entity<Sidebar>,
    page: Page,
    /// The user's sources, shared by every page that needs them.
    storage: Entity<JSONDatabase>,
    all_sources: Entity<AllSources>,
    new_source: Entity<NewSource>,
}

impl Workspace {
    pub fn new(storage: Entity<JSONDatabase>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let sidebar = cx.new(|_| Sidebar::new());
        let all_sources = cx.new(|cx| AllSources::new(storage.clone(), cx));
        let new_source = cx.new(|cx| NewSource::new(storage.clone(), window, cx));

        cx.subscribe(&sidebar, |this, _, page: &Page, cx| {
            this.page = *page;
            cx.notify();
        })
        .detach();

        Self {
            sidebar,
            page: Page::default(),
            storage,
            all_sources,
            new_source,
        }
    }

    /// The content area for the current page.
    fn render_page(&self) -> AnyElement {
        match self.page {
            Page::AllFeeds => empty_page("page-all-feeds"),
            Page::AllSources => self.all_sources.clone().into_any_element(),
            Page::NewSource => self.new_source.clone().into_any_element(),
        }
    }
}

/// A placeholder for a page that has no content yet.
fn empty_page(id: &'static str) -> AnyElement {
    div().id(id).v_flex().flex_1().size_full().into_any_element()
}

impl Render for Workspace {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .h_flex()
            .size_full()
            .child(self.sidebar.clone())
            .child(self.render_page())
    }
}
