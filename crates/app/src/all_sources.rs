use anyhow::Context as _;
use gpui_kit::Entity;
use gpui_kit::PromptLevel;
use gpui_kit::Window;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::component::IconName;
use gpui_kit::component::button::Button;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::div;
use gpui_kit::prelude::*;
use storage::JSONDatabase;
use storage::Source;

/// Label of the prompt button that confirms a delete. It is the first answer, so the prompt returns `0` for it.
const DELETE_ANSWER: &str = "Delete";

/// Every source the user has subscribed to.
pub struct AllSources {
    storage: Entity<JSONDatabase>,
}

impl AllSources {
    pub fn new(storage: Entity<JSONDatabase>, cx: &mut Context<Self>) -> Self {
        // Redraw whenever a source is added, removed, or changed.
        cx.observe(&storage, |_, _, cx| cx.notify()).detach();

        Self { storage }
    }

    /// Ask the user, with a system prompt, whether to delete `source`, and delete it if they agree.
    fn confirm_delete(source: Source, window: &mut Window, cx: &mut Context<Self>) {
        let message = format!("Delete “{}”?", source.name());
        let answer = window.prompt(
            PromptLevel::Warning,
            &message,
            Some("This source will be removed. You can add it again later."),
            &[DELETE_ANSWER, "Cancel"],
            cx,
        );

        cx.spawn_in(window, async move |this, cx| {
            if answer.await != Ok(0) {
                return anyhow::Ok(());
            }

            this.update(cx, |this, cx| this.delete(&source, cx))?
        })
        .detach_and_log_err(cx);
    }

    /// Remove `source` and save. The prompt is async, so the source is found by value rather than by its old index.
    fn delete(&self, source: &Source, cx: &mut Context<Self>) -> anyhow::Result<()> {
        self.storage.update(cx, |db, cx| {
            let Some(index) = db.sources().iter().position(|s| s == source) else {
                return Ok(());
            };

            let removed = db.sources_mut().remove(index);

            if let Err(err) = db.write() {
                db.sources_mut().insert(index, removed);
                return Err(err).context("failed to delete source");
            }

            cx.notify();
            Ok(())
        })
    }

    /// One source as a row: its name with the feed URL beneath, and a delete button.
    fn source_row(index: usize, source: &Source, cx: &Context<Self>) -> impl IntoElement {
        let delete = {
            let source = source.clone();
            cx.listener(move |_, _, window, cx| Self::confirm_delete(source.clone(), window, cx))
        };

        div()
            .id(("source", index))
            .h_flex()
            .gap_3()
            .py_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .v_flex()
                    .flex_1()
                    .gap_1()
                    .child(div().font_medium().child(source.name().to_owned()))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(source.url().to_owned()),
                    ),
            )
            .child(
                Button::new(("delete-source", index))
                    .danger()
                    .icon(IconName::Delete)
                    .tooltip("Delete source")
                    .on_click(delete),
            )
    }
}

impl Render for AllSources {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sources = self.storage.read(cx).sources();

        let body = if sources.is_empty() {
            div()
                .text_color(cx.theme().muted_foreground)
                .child("No sources yet. Add one from Sources → New.")
                .into_any_element()
        } else {
            div()
                .v_flex()
                .children(
                    sources
                        .iter()
                        .enumerate()
                        .map(|(index, source)| Self::source_row(index, source, cx)),
                )
                .into_any_element()
        };

        div()
            .id("page-all-sources")
            .v_flex()
            .flex_1()
            .size_full()
            .gap_4()
            .p_6()
            .overflow_y_scroll()
            .child(div().text_xl().font_semibold().child("Sources"))
            .child(body)
    }
}
