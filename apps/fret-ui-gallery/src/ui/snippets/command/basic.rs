pub const SOURCE: &str = include_str!("basic.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    open: Option<Model<bool>>,
    query: Option<Model<String>>,
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let (open, query) = cx.with_state(Models::default, |st| (st.open.clone(), st.query.clone()));

    let open = match open {
        Some(open) => open,
        None => {
            let open = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| st.open = Some(open.clone()));
            open
        }
    };

    let query = match query {
        Some(query) => query,
        None => {
            let query = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.query = Some(query.clone()));
            query
        }
    };

    let last_action_model = last_action.clone();
    let on_select = {
        let last_action = last_action_model.clone();
        move |tag: Arc<str>| {
            let last_action = last_action.clone();
            Arc::new(
                move |host: &mut dyn fret_ui::action::UiActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      _reason: fret_ui::action::ActivateReason| {
                    let value = tag.clone();
                    let _ = host
                        .models_mut()
                        .update(&last_action, |cur: &mut Arc<str>| {
                            *cur = value.clone();
                        });
                    host.request_redraw(action_cx.window);
                },
            ) as fret_ui::action::OnActivate
        }
    };

    let items = vec![
        shadcn::CommandItem::new("Calendar")
            .shortcut("Cmd+C")
            .keywords(["events", "schedule"])
            .on_select_action(on_select(Arc::from("command.basic.calendar"))),
        shadcn::CommandItem::new("Search Emoji")
            .shortcut("Cmd+E")
            .keywords(["emoji", "insert"])
            .on_select_action(on_select(Arc::from("command.basic.search-emoji"))),
        shadcn::CommandItem::new("Calculator")
            .shortcut("Cmd+K")
            .keywords(["math", "calc"])
            .on_select_action(on_select(Arc::from("command.basic.calculator"))),
    ];

    shadcn::CommandDialog::new(open.clone(), query.clone(), items)
        .a11y_label("Basic command dialog")
        .empty_text("No results found.")
        .into_element(cx, |cx| {
            shadcn::Button::new("Open Command Menu")
                .variant(shadcn::ButtonVariant::Outline)
                .toggle_model(open.clone())
                .test_id("ui-gallery-command-basic-trigger")
                .into_element(cx)
        })
        .test_id("ui-gallery-command-basic")
}
// endregion: example
