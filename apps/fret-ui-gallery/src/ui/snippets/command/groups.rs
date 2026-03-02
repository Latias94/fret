pub const SOURCE: &str = include_str!("groups.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    query: Option<Model<String>>,
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let query = cx.with_state(Models::default, |st| st.query.clone());
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

    let entries: Vec<shadcn::CommandEntry> = vec![
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Calendar")
                .keywords(["events"])
                .on_select_action(on_select(Arc::from("command.groups.calendar"))),
            shadcn::CommandItem::new("Search Emoji")
                .keywords(["emoji"])
                .on_select_action(on_select(Arc::from("command.groups.search-emoji"))),
            shadcn::CommandItem::new("Calculator")
                .keywords(["math"])
                .on_select_action(on_select(Arc::from("command.groups.calculator"))),
        ])
        .heading("Suggestions")
        .into(),
        shadcn::CommandSeparator::new().into(),
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Profile")
                .shortcut("Cmd+,")
                .on_select_action(on_select(Arc::from("command.groups.profile"))),
            shadcn::CommandItem::new("Billing")
                .shortcut("Alt+Cmd+B")
                .on_select_action(on_select(Arc::from("command.groups.billing"))),
            shadcn::CommandItem::new("Settings")
                .shortcut("Cmd+S")
                .on_select_action(on_select(Arc::from("command.groups.settings"))),
        ])
        .heading("Settings")
        .into(),
    ];

    shadcn::CommandPalette::new(query.clone(), Vec::new())
        .placeholder("Search grouped commands...")
        .a11y_label("Command groups")
        .entries(entries)
        .test_id_input("ui-gallery-command-groups-input")
        .list_test_id("ui-gallery-command-groups-listbox")
        .test_id_item_prefix("ui-gallery-command-groups-item-")
        .into_element(cx)
        .test_id("ui-gallery-command-groups")
}
// endregion: example
