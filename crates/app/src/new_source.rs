use gpui_kit::Entity;
use gpui_kit::SharedString;
use gpui_kit::Window;
use gpui_kit::base::StyledExt as _;
use gpui_kit::component::button::Button;
use gpui_kit::component::button::ButtonVariants as _;
use gpui_kit::component::form::field;
use gpui_kit::component::form::v_form;
use gpui_kit::component::input::Input;
use gpui_kit::component::input::InputEvent;
use gpui_kit::component::input::InputState;
use gpui_kit::div;
use gpui_kit::prelude::*;
use gpui_kit::px;
use storage::JSONDatabase;
use storage::Source;

/// The widest the form grows, so fields stay readable on large windows.
const FORM_MAX_WIDTH: f32 = 480.0;

/// A form for subscribing to a new source.
pub struct NewSource {
    storage: Entity<JSONDatabase>,
    name: Entity<InputState>,
    url: Entity<InputState>,
    /// Why the last submit failed, shown under the form.
    error: Option<SharedString>,
}

impl NewSource {
    pub fn new(storage: Entity<JSONDatabase>, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let name = cx.new(|cx| InputState::new(window, cx).placeholder("My favourite blog"));
        let url = cx.new(|cx| InputState::new(window, cx).placeholder("https://example.com/feed.xml"));

        for input in [&name, &url] {
            cx.subscribe_in(input, window, |this, _, event, window, cx| {
                if matches!(event, InputEvent::PressEnter { .. }) {
                    this.submit(window, cx);
                }
            })
            .detach();
        }

        Self {
            storage,
            name,
            url,
            error: None,
        }
    }

    /// Validate the fields, save the source, and clear the form.
    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let name = self.name.read(cx).value().trim().to_owned();
        let url = self.url.read(cx).value().trim().to_owned();

        let result = validate(&name, &url, self.storage.read(cx).sources()).and_then(|()| {
            self.storage
                .update(cx, |db, cx| {
                    db.sources_mut().push(Source::new(name, url));

                    if let Err(err) = db.write() {
                        db.sources_mut().pop();
                        return Err(err);
                    }

                    cx.notify();
                    Ok(())
                })
                .map_err(|err| SharedString::from(format!("Could not save the source: {err:#}")))
        });

        match result {
            Ok(()) => {
                self.error = None;
                self.name.update(cx, |input, cx| input.set_value("", window, cx));
                self.url.update(cx, |input, cx| input.set_value("", window, cx));
            }
            Err(err) => self.error = Some(err),
        }

        cx.notify();
    }
}

/// Check the form's values against the `existing` sources, returning a message for the user if they are unusable.
fn validate(name: &str, url: &str, existing: &[Source]) -> Result<(), SharedString> {
    if name.is_empty() {
        return Err("Give the source a name.".into());
    }

    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("The URL must start with http:// or https://.".into());
    }

    if let Some(duplicate) = existing.iter().find(|source| same_url(source.url(), url)) {
        return Err(format!("“{}” already uses this URL.", duplicate.name()).into());
    }

    Ok(())
}

/// Whether two feed URLs point at the same place, ignoring a trailing slash.
fn same_url(a: &str, b: &str) -> bool {
    a.trim_end_matches('/') == b.trim_end_matches('/')
}

impl Render for NewSource {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().id("page-new-source").v_flex().flex_1().size_full().p_6().child(
            div()
                .v_flex()
                .gap_4()
                .w_full()
                .max_w(px(FORM_MAX_WIDTH))
                .child(div().text_xl().font_semibold().child("New source"))
                .child(
                    v_form()
                        .child(field().label("Name").child(Input::new(&self.name)))
                        .child(field().label("Feed URL").child(Input::new(&self.url))),
                )
                .children(
                    self.error
                        .clone()
                        .map(|error| div().text_sm().text_color(gpui_kit::red()).child(error)),
                )
                .child(
                    div().h_flex().justify_end().child(
                        Button::new("add-source")
                            .primary()
                            .label("Add source")
                            .on_click(cx.listener(|this, _, window, cx| this.submit(window, cx))),
                    ),
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URL: &str = "https://example.com/feed.xml";

    #[test]
    fn accepts_a_new_url() {
        let existing = [Source::new("Other", "https://other.com/feed.xml")];

        assert!(validate("Example", URL, &existing).is_ok());
    }

    #[test]
    fn rejects_an_existing_url() {
        let existing = [Source::new("Example", URL)];

        assert!(validate("Again", URL, &existing).is_err());
    }

    #[test]
    fn trailing_slash_is_the_same_url() {
        let existing = [Source::new("Example", "https://example.com/")];

        assert!(validate("Again", "https://example.com", &existing).is_err());
    }
}
