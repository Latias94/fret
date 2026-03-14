pub const SOURCE: &str = include_str!("action_first_view.rs");

// region: example
#[cfg(not(target_arch = "wasm32"))]
use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::rc::Rc;
use std::sync::Arc;

use fret::advanced::prelude::*;
use fret::app::App;
use fret::{UiChild, UiCx};
use fret_ui::CommandAvailability;
use fret_ui_shadcn::facade as shadcn;

mod act {
    fret::actions!([Ping = "ui-gallery.command.action_first.ping.v1"]);
}

#[derive(Default)]
struct ActionFirstViewRuntimeDemo {
    last_action: Option<Model<Arc<str>>>,
}

impl View for ActionFirstViewRuntimeDemo {
    fn init(_app: &mut App, _window: AppWindowId) -> Self {
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

        cx.actions().models::<act::Ping>({
            let count_state = count_state.clone();
            let last_action = last_action.clone();
            move |models| {
                let count_updated = count_state.update_in(models, |v| *v = v.saturating_add(1));
                let last_action_updated = models
                    .update(&last_action, |v| {
                        *v = Arc::from("Ping (view runtime)");
                    })
                    .is_ok();
                count_updated && last_action_updated
            }
        });

        cx.actions()
            .availability::<act::Ping>(|_host, _acx| CommandAvailability::Available);

        ui::v_flex(|cx| {
            [
                shadcn::Label::new("Action-first (view runtime)").into_element(cx),
                cx.text(format!("Count: {count_value}")),
                cx.text(format!("Last action: {last_action_value}")),
                shadcn::Button::new("Ping")
                    .action(act::Ping)
                    .into_element(cx)
                    .test_id("ui-gallery-command-action-first-view-runtime.ping"),
            ]
        })
        .gap(Space::N2)
        .into_element(cx)
        .test_id("ui-gallery-command-action-first-view-runtime")
        .into()
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    let last_action = super::last_action_model(cx);
    cx.named("ui-gallery.command.action_first.view_runtime", move |cx| {
        let view_state_slot = cx.slot_id();
        let view_state = cx.state_for(
            view_state_slot,
            || None::<Rc<RefCell<fret::view::ViewWindowState<ActionFirstViewRuntimeDemo>>>>,
            |slot| slot.clone(),
        );
        let view_state = match view_state {
            Some(state) => state,
            None => {
                let state = Rc::new(RefCell::new(fret::view::view_init_window::<
                    ActionFirstViewRuntimeDemo,
                >(&mut *cx.app, cx.window)));
                cx.state_for(
                    view_state_slot,
                    || None::<Rc<RefCell<fret::view::ViewWindowState<ActionFirstViewRuntimeDemo>>>>,
                    |slot| {
                        if slot.is_none() {
                            *slot = Some(state.clone());
                        }
                        slot.clone()
                            .expect("view runtime slot must contain state after init")
                    },
                )
            }
        };

        let mut st = view_state.borrow_mut();
        st.view.last_action = Some(last_action.clone());

        let elements = fret::view::view_view(cx, &mut *st);
        elements
            .into_vec()
            .into_iter()
            .next()
            .expect("view runtime must produce a root element")
    })
}

#[cfg(target_arch = "wasm32")]
pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<> {
    cx.named("ui-gallery.command.action_first.view_runtime", |cx| {
        ui::v_flex(|cx| {
            [
                shadcn::Label::new("Action-first (view runtime)").into_element(cx),
                cx.text("This demo is desktop-only in v1."),
            ]
        })
        .gap(Space::N2)
        .into_element(cx)
        .test_id("ui-gallery-command-action-first-view-runtime")
    })
}
// endregion: example
