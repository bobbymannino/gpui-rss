use gpui_kit::App;
use gpui_kit::Entity;
use gpui_kit::Window;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::ActiveTheme as _;
use gpui_kit::div;
use gpui_kit::prelude::*;
use storage::JSONDatabase;
use storage::Source;

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
}

/// One source as a row: its name, with the feed URL beneath.
fn source_row(index: usize, source: &Source, cx: &App) -> impl IntoElement {
    div()
        .id(("source", index))
        .v_flex()
        .gap_1()
        .py_3()
        .border_b_1()
        .border_color(cx.theme().border)
        .child(div().font_medium().child(source.name().to_owned()))
        .child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(source.url().to_owned()),
        )
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
                        .map(|(index, source)| source_row(index, source, cx)),
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
