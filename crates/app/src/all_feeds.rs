use feed::Post;
use gpui_kit::Entity;
use gpui_kit::SharedString;
use gpui_kit::Task;
use gpui_kit::Window;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::div;
use gpui_kit::prelude::*;
use storage::JSONDatabase;

/// A post along with the name of the source it came from.
struct FeedPost {
    source: String,
    post: Post,
}

/// What the page has to show for the latest fetch.
enum State {
    /// Posts are still being fetched.
    Loading,
    /// Every source has been fetched. `errors` describes each source that failed.
    Loaded {
        posts: Vec<FeedPost>,
        errors: Vec<SharedString>,
    },
}

/// Every post from every source, newest first.
pub struct AllFeeds {
    storage: Entity<JSONDatabase>,
    state: State,
    /// The in flight fetch. Replacing it cancels the previous one.
    fetch: Task<()>,
}

impl AllFeeds {
    pub fn new(storage: Entity<JSONDatabase>, cx: &mut Context<Self>) -> Self {
        // Refetch whenever a source is added, removed, or changed.
        cx.observe(&storage, |this, _, cx| this.reload(cx)).detach();

        let mut this = Self {
            storage,
            state: State::Loading,
            fetch: Task::ready(()),
        };
        this.reload(cx);
        this
    }

    /// Fetch every source in the background, then show the combined posts.
    fn reload(&mut self, cx: &mut Context<Self>) {
        // Spawn every fetch up front so the sources download concurrently.
        let pending: Vec<_> = self
            .storage
            .read(cx)
            .sources()
            .iter()
            .map(|source| {
                let name = source.name().to_owned();
                let url = source.url().to_owned();
                cx.background_spawn(async move { (name, feed::fetch(&url)) })
            })
            .collect();

        self.state = State::Loading;
        self.fetch = cx.spawn(async move |this, cx| {
            let mut posts = Vec::new();
            let mut errors = Vec::new();

            for task in pending {
                match task.await {
                    (source, Ok(fetched)) => posts.extend(fetched.into_iter().map(|post| FeedPost {
                        source: source.clone(),
                        post,
                    })),
                    (source, Err(err)) => errors.push(SharedString::from(format!("{source}: {err:#}"))),
                }
            }

            // Newest first. Posts without a date sort last, since `None` is less than any `Some`.
            posts.sort_by_key(|feed_post| std::cmp::Reverse(feed_post.post.published_at()));

            this.update(cx, |this, cx| {
                this.state = State::Loaded { posts, errors };
                cx.notify();
            })
            .ok();
        });
        cx.notify();
    }

    /// One post as a row: its title with the source and date beneath. Clicking it opens the post in the browser.
    fn post_row(index: usize, feed_post: &FeedPost, cx: &Context<Self>) -> impl IntoElement {
        let post = &feed_post.post;
        let meta = post.published_at().map_or_else(
            || feed_post.source.clone(),
            |published_at| format!("{} · {}", feed_post.source, published_at.format("%-d %b %Y")),
        );
        let title = if post.title().is_empty() {
            "Untitled"
        } else {
            post.title()
        };

        let row = div()
            .id(("post", index))
            .v_flex()
            .gap_1()
            .py_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(div().font_medium().child(title.to_owned()))
            .child(div().text_sm().text_color(cx.theme().muted_foreground).child(meta));

        match post.link() {
            Some(link) => {
                let link = link.to_owned();
                row.cursor_pointer()
                    .hover(|style| style.bg(cx.theme().accent))
                    .on_click(move |_, _, cx| cx.open_url(&link))
            }
            None => row,
        }
    }
}

impl Render for AllFeeds {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;

        let body = match &self.state {
            State::Loading => div()
                .h_flex()
                .gap_2()
                .text_color(muted)
                .child(Spinner::new())
                .child("Loading posts…"),
            State::Loaded { posts, errors } => {
                let empty = posts.is_empty().then(|| {
                    let message = if self.storage.read(cx).sources().is_empty() {
                        "No sources yet. Add one from Sources → New."
                    } else {
                        "No posts yet."
                    };
                    div().text_color(muted).child(message)
                });

                div()
                    .v_flex()
                    .children(
                        errors
                            .iter()
                            .map(|error| div().text_sm().text_color(cx.theme().danger).child(error.clone())),
                    )
                    .children(empty)
                    .children(
                        posts
                            .iter()
                            .enumerate()
                            .map(|(index, feed_post)| Self::post_row(index, feed_post, cx)),
                    )
            }
        };

        div()
            .id("page-all-feeds")
            .v_flex()
            .flex_1()
            .size_full()
            .gap_4()
            .p_6()
            .overflow_y_scroll()
            .child(div().text_xl().font_semibold().child("Feeds"))
            .child(body)
    }
}
