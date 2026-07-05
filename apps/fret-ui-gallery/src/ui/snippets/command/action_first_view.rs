pub const SOURCE: &str = include_str!("action_first_view.rs");

// region: example
use std::sync::Arc;

use fret::AppComponentCx;
use fret::app::prelude::*;
use fret::style::Space;
use fret_runtime::Model;
use fret_ui::CommandAvailability;
use fret_ui_kit::declarative::text as decl_text;
use fret_ui_shadcn::facade as shadcn;

mod act {
    fret::actions!([Ping = "ui-gallery.command.action_first.ping.v1"]);
}

#[derive(Default)]
struct ActionFirstViewRuntimeDemo {
    last_action: Option<Model<Arc<str>>>,
}

impl View for ActionFirstViewRuntimeDemo {
    fn init(_app: &mut App, _window: WindowId) -> Self {
        Self { last_action: None }
    }

    fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui {
        let last_action = self
            .last_action
            .clone()
            .expect("expected snippet to inject `last_action` model");

        let count_state = cx.state().local::<u32>();
        let count_value = count_state.watch(cx).layout().value_or(0);
        let last_action_value = last_action.watch(cx).layout().value_or_default();

        cx.actions().locals_with(&count_state).on::<act::Ping>({
            let last_action = last_action.clone();
            move |tx, count_state| {
                let count_updated = tx.update(&count_state, |v| *v = v.saturating_add(1));
                let last_action_updated =
                    tx.update_shared_model(&last_action, |v| *v = Arc::from("Ping (view runtime)"));
                count_updated || last_action_updated
            }
        });

        cx.actions()
            .availability::<act::Ping>(|_host, _acx| CommandAvailability::Available);

        ui::v_flex(|cx| {
            [
                shadcn::Label::new("Action-first (view runtime)").into_element(cx),
                decl_text::text_control_readout(cx, format!("Count: {count_value}")),
                decl_text::text_control_readout(cx, format!("Last action: {last_action_value}")),
                shadcn::Button::new("Ping")
                    .action(act::Ping)
                    .into_element(cx)
                    .test_id("ui-gallery-command-action-first-view-runtime.button-ping"),
                shadcn::Badge::new("Ping via activate sugar")
                    .action(act::Ping)
                    .test_id("ui-gallery-command-action-first-view-runtime.ping")
                    .into_element(cx),
            ]
        })
        .gap(Space::N2)
        .into_element_in(cx)
        .test_id("ui-gallery-command-action-first-view-runtime")
        .into()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let last_action = super::last_action_model(cx);
    fret::app::view_child_with(
        cx,
        "ui-gallery.command.action_first.view_runtime",
        move |view: &mut ActionFirstViewRuntimeDemo| {
            view.last_action = Some(last_action.clone());
        },
    )
}

#[cfg(target_arch = "wasm32")]
pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    cx.named("ui-gallery.command.action_first.view_runtime", |cx| {
        ui::v_flex(|cx| {
            [
                shadcn::Label::new("Action-first (view runtime)").into_element(cx),
                decl_text::text_paragraph(cx, "This demo is desktop-only in v1."),
            ]
        })
        .gap(Space::N2)
        .into_element(cx)
        .test_id("ui-gallery-command-action-first-view-runtime")
    })
}
// endregion: example
