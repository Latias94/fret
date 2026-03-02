// region: example
use fret_core::Px;
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

    let on_select = {
        let last_action = last_action.clone();
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

    let scroll_action = on_select(Arc::from("command.scrollable.item"));
    let recent_items = (1..=24)
        .map(|index| {
            shadcn::CommandItem::new(format!("Recent file {index:02}"))
                .keywords([format!("recent-{index:02}"), format!("file-{index:02}")])
                .on_select_action(scroll_action.clone())
        })
        .collect::<Vec<_>>();
    let workspace_items = (1..=18)
        .map(|index| {
            shadcn::CommandItem::new(format!("Workspace command {index:02}"))
                .keywords([format!("workspace-{index:02}")])
                .on_select_action(scroll_action.clone())
        })
        .collect::<Vec<_>>();

    let entries = vec![
        shadcn::CommandGroup::new(recent_items)
            .heading("Recent Files")
            .into(),
        shadcn::CommandSeparator::new().into(),
        shadcn::CommandGroup::new(workspace_items)
            .heading("Workspace")
            .into(),
    ];

    shadcn::CommandPalette::new(query.clone(), Vec::new())
        .placeholder("Search a long command list...")
        .a11y_label("Scrollable command list")
        .entries(entries)
        .test_id_input("ui-gallery-command-scrollable-input")
        .list_test_id("ui-gallery-command-scrollable-listbox")
        .test_id_item_prefix("ui-gallery-command-scrollable-item-")
        .refine_scroll_layout(LayoutRefinement::default().h_px(Px(220.0)).max_h(Px(220.0)))
        .into_element(cx)
        .test_id("ui-gallery-command-scrollable")
}
// endregion: example
