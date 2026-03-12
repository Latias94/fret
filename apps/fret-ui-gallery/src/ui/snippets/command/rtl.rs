pub const SOURCE: &str = include_str!("rtl.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let query = cx.local_model(String::new);

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

    let entries = vec![
        shadcn::CommandGroup::new([
            shadcn::CommandItem::new("Search")
                .shortcut("Cmd+F")
                .on_select_action(on_select(Arc::from("command.rtl.search"))),
            shadcn::CommandItem::new("Dashboard")
                .shortcut("Cmd+D")
                .on_select_action(on_select(Arc::from("command.rtl.dashboard"))),
            shadcn::CommandItem::new("Settings")
                .shortcut("Cmd+,")
                .on_select_action(on_select(Arc::from("command.rtl.settings"))),
        ])
        .heading("RTL")
        .into(),
    ];

    with_direction_provider(cx, LayoutDirection::Rtl, |cx| {
        shadcn::CommandPalette::new(query.clone(), Vec::new())
            .placeholder("Type a command or search...")
            .a11y_label("RTL command list")
            .entries(entries)
            .test_id_input("ui-gallery-command-rtl-input")
            .list_test_id("ui-gallery-command-rtl-listbox")
            .test_id_item_prefix("ui-gallery-command-rtl-item-")
            .into_element(cx)
            .test_id("ui-gallery-command-rtl")
    })
}
// endregion: example
