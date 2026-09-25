use gpui_kit::base::StyledExt as _;
use gpui_kit::div;
use gpui_kit::prelude::*;

/// A page the workspace can show next to the sidebar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Page {
    /// Every item from every feed.
    #[default]
    AllFeeds,
    /// Every source the user has added.
    AllSources,
    /// Add a new source.
    NewSource,
}

impl Page {
    /// The page's content area. Every page is empty for now.
    pub fn render(self) -> impl IntoElement {
        let id = match self {
            Self::AllFeeds => "page-all-feeds",
            Self::AllSources => "page-all-sources",
            Self::NewSource => "page-new-source",
        };

        div().id(id).v_flex().flex_1().size_full()
    }
}
