pub const SOURCE: &str = include_str!("action_first_view.rs");

// region: example
#[cfg(not(target_arch = "wasm32"))]
use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::rc::Rc;
use std::sync::Arc;

use fret::prelude::*;
use fret_ui::CommandAvailability;
use fret_ui_shadcn::{self as shadcn};

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

    fn render(&mut self, cx: &mut ViewCx<'_, '_, App>) -> Elements {
        let last_action = self
            .last_action
            .clone()
            .expect("expected snippet to inject `last_action` model");

        let count = cx.use_state::<u32>();
        let count_value = cx.watch_model(&count).layout().copied_or(0);
        let last_action_value = cx.watch_model(&last_action).layout().cloned_or_default();

        cx.on_action::<act::Ping>({
            let count = count.clone();
            let last_action = last_action.clone();
            move |host, acx| {
                let _ = host
                    .models_mut()
                    .update(&count, |v| *v = v.saturating_add(1));
                let _ = host.models_mut().update(&last_action, |v| {
                    *v = Arc::from("Ping (view runtime)");
                });
                host.request_redraw(acx.window);
                host.notify(acx);
                true
            }
        });

        cx.on_action_availability::<act::Ping>(|_host, _acx| CommandAvailability::Available);

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
pub fn render(cx: &mut ElementContext<'_, App>, last_action: Model<Arc<str>>) -> AnyElement {
    cx.named("ui-gallery.command.action_first.view_runtime", |cx| {
        struct ViewSlot {
            state: Option<Rc<RefCell<fret::view::ViewWindowState<ActionFirstViewRuntimeDemo>>>>,
        }

        impl Default for ViewSlot {
            fn default() -> Self {
                Self { state: None }
            }
        }

        let existing = cx.with_state(ViewSlot::default, |slot| slot.state.clone());
        let view_state = match existing {
            Some(state) => state,
            None => {
                let state = Rc::new(RefCell::new(fret::view::view_init_window::<
                    ActionFirstViewRuntimeDemo,
                >(&mut *cx.app, cx.window)));
                cx.with_state(ViewSlot::default, |slot| {
                    if slot.state.is_none() {
                        slot.state = Some(state.clone());
                    }
                    slot.state
                        .clone()
                        .expect("slot must contain view state after init")
                })
            }
        };

        let mut st = view_state.borrow_mut();
        st.view.last_action = Some(last_action);

        let elements = fret::view::view_view(cx, &mut *st);
        elements
            .into_vec()
            .into_iter()
            .next()
            .expect("view runtime must produce a root element")
    })
}

#[cfg(target_arch = "wasm32")]
pub fn render(cx: &mut ElementContext<'_, App>, _last_action: Model<Arc<str>>) -> AnyElement {
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
