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
