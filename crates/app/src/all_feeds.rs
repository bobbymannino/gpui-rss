use chrono::Utc;
use feed::Post;
use gpui_kit::Entity;
use gpui_kit::SharedString;
use gpui_kit::Task;
use gpui_kit::Window;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::IconName;
use gpui_kit::component::spinner::Spinner;
use gpui_kit::div;
use gpui_kit::prelude::*;
use storage::JSONDatabase;

/// What the page has to show for the latest fetch.
enum State {
    /// Posts are still being fetched.
    Loading,
    /// Every source has been fetched. `errors` describes each source that failed.
    Loaded { errors: Vec<SharedString> },
}

/// Every post from every source, newest first. Posts are stored on their source, so the last fetch shows while the
/// next one loads.
pub struct AllFeeds {
    storage: Entity<JSONDatabase>,
    state: State,
    /// URLs of the sources the latest fetch covered, to tell a changed source list apart from saved posts.
    fetched_urls: Vec<String>,
    /// Why the last attempt to save a post as read failed, if it did.
    read_error: Option<SharedString>,
    /// The in flight fetch. Replacing it cancels the previous one.
    fetch: Task<()>,
}

impl AllFeeds {
    pub fn new(storage: Entity<JSONDatabase>, cx: &mut Context<Self>) -> Self {
        // Refetch whenever a source is added or removed. Saving fetched posts also notifies, so compare URLs to avoid
        // fetching in a loop.
        cx.observe(&storage, |this, storage, cx| {
            if source_urls(storage.read(cx)) != this.fetched_urls {
                this.reload(cx);
            }
        })
        .detach();

        let mut this = Self {
            storage,
            state: State::Loading,
            fetched_urls: Vec::new(),
            read_error: None,
            fetch: Task::ready(()),
        };
        this.reload(cx);
        this
    }

    /// Fetch every source in the background, then save each source's posts.
    fn reload(&mut self, cx: &mut Context<Self>) {
        self.fetched_urls = source_urls(self.storage.read(cx));

        // Spawn every fetch up front so the sources download concurrently.
        let pending: Vec<_> = self
            .storage
            .read(cx)
            .sources()
            .iter()
            .map(|source| {
                let name = source.name().to_owned();
                let url = source.url().to_owned();
                cx.background_spawn(async move {
                    let result = feed::fetch(&url);
                    (name, url, result)
                })
            })
            .collect();

        self.state = State::Loading;
        self.fetch = cx.spawn(async move |this, cx| {
            let mut fetched = Vec::new();
            let mut errors = Vec::new();

            for task in pending {
                match task.await {
                    (_, url, Ok(posts)) => fetched.push((url, posts)),
                    (name, _, Err(err)) => errors.push(SharedString::from(format!("{name}: {err:#}"))),
                }
            }

            this.update(cx, |this, cx| {
                this.storage.update(cx, |db, cx| {
                    for (url, posts) in fetched {
                        // The source may have been removed while it was being fetched.
                        if let Some(source) = db.sources_mut().iter_mut().find(|source| source.url() == url) {
                            source.set_posts(posts);
                        }
                    }

                    if let Err(err) = db.write() {
                        errors.push(SharedString::from(format!("Could not save posts: {err:#}")));
                    }

                    cx.notify();
                });

                this.state = State::Loaded { errors };
                cx.notify();
            })
            .ok();
        });
        cx.notify();
    }

    /// Mark the post with `post_id` from the source at `source_url` as read now and save, unless it was already read.
    fn mark_read(&mut self, source_url: &str, post_id: &str, cx: &mut Context<Self>) {
        let result = self.storage.update(cx, |db, cx| {
            let Some(post) = find_post(db, source_url, post_id).filter(|post| post.read_at().is_none()) else {
                return Ok(());
            };
            post.set_read_at(Some(Utc::now()));

            if let Err(err) = db.write() {
                if let Some(post) = find_post(db, source_url, post_id) {
                    post.set_read_at(None);
                }
                return Err(err);
            }

            cx.notify();
            Ok(())
        });

        self.read_error = result
            .err()
            .map(|err| SharedString::from(format!("Could not mark the post as read: {err:#}")));
        cx.notify();
    }

    /// One post as a row: its title with the source and date beneath. Clicking it marks it read and opens it in the
    /// browser.
    fn post_row(
        index: usize,
        source_name: &str,
        source_url: &str,
        post: &Post,
        cx: &Context<Self>,
    ) -> impl IntoElement {
        let meta = post.published_at().map_or_else(
            || source_name.to_owned(),
            |published_at| format!("{source_name} · {}", published_at.format("%-d %b %Y")),
        );
        let title = if post.title().is_empty() {
            "Untitled"
        } else {
            post.title()
        };

        let source_url = source_url.to_owned();
        let post_id = post.id().to_owned();
        let link = post.link().map(str::to_owned);

        div()
            .id(("post", index))
            .h_flex()
            .gap_1()
            .justify_between()
            .py_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .cursor_pointer()
            .hover(|style| style.bg(cx.theme().accent))
            .on_click(cx.listener(move |this, _, _, cx| {
                this.mark_read(&source_url, &post_id, cx);
                if let Some(link) = &link {
                    cx.open_url(link);
                }
            }))
            .child(
                div()
                    .child(div().font_medium().child(title.to_owned()))
                    .child(div().text_sm().text_color(cx.theme().muted_foreground).child(meta)),
            )
            .when(post.read_at().is_some(), |div| div.child(IconName::Eye))
    }
}

impl Render for AllFeeds {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let muted = cx.theme().muted_foreground;
        let db = self.storage.read(cx);

        let mut posts: Vec<_> = db
            .sources()
            .iter()
            .flat_map(|source| source.posts().iter().map(move |post| (source, post)))
            .collect();
        // Newest first. Posts without a date sort last, since `None` is less than any `Some`.
        posts.sort_by_key(|(_, post)| std::cmp::Reverse(post.published_at()));

        let status = match &self.state {
            State::Loading => Some(
                div()
                    .h_flex()
                    .gap_2()
                    .text_color(muted)
                    .child(Spinner::new())
                    .child("Loading posts…"),
            ),
            State::Loaded { .. } => None,
        };
        let errors = match &self.state {
            State::Loading => &[][..],
            State::Loaded { errors } => errors,
        };
        let empty = (posts.is_empty() && matches!(self.state, State::Loaded { .. })).then(|| {
            let message = if db.sources().is_empty() {
                "No sources yet. Add one from Sources → New."
            } else {
                "No posts yet."
            };
            div().text_color(muted).child(message)
        });

        let body = div()
            .v_flex()
            .children(status)
            .children(
                errors
                    .iter()
                    .chain(&self.read_error)
                    .map(|error| div().text_sm().text_color(cx.theme().danger).child(error.clone())),
            )
            .children(empty)
            .children(
                posts
                    .into_iter()
                    .enumerate()
                    .map(|(index, (source, post))| Self::post_row(index, source.name(), source.url(), post, cx)),
            );

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

/// The post with `post_id` from the source at `source_url`, if both still exist.
fn find_post<'a>(db: &'a mut JSONDatabase, source_url: &str, post_id: &str) -> Option<&'a mut Post> {
    db.sources_mut()
        .iter_mut()
        .find(|source| source.url() == source_url)
        .and_then(|source| source.post_mut(post_id))
}

/// URLs of every source in `db`, in order.
fn source_urls(db: &JSONDatabase) -> Vec<String> {
    db.sources().iter().map(|source| source.url().to_owned()).collect()
}
