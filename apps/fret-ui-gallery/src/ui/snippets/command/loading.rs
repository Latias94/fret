pub const SOURCE: &str = include_str!("loading.rs");

// region: example
use fret_ui_shadcn::{self as shadcn, prelude::*};
use std::sync::Arc;

#[derive(Default)]
struct Models {
    query: Option<Model<String>>,
    loading_enabled: Option<Model<bool>>,
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let (query, loading_enabled) = cx.with_state(Models::default, |st| {
        (st.query.clone(), st.loading_enabled.clone())
    });

    let query = match query {
        Some(query) => query,
        None => {
            let query = cx.app.models_mut().insert(String::new());
            cx.with_state(Models::default, |st| st.query = Some(query.clone()));
            query
        }
    };

    let loading_enabled = match loading_enabled {
        Some(model) => model,
        None => {
            let model = cx.app.models_mut().insert(false);
            cx.with_state(Models::default, |st| {
                st.loading_enabled = Some(model.clone())
            });
            model
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

    ui::v_flex(move |cx: &mut ElementContext<'_, H>| {
        let loading_enabled_value = cx
            .app
            .models()
            .get_cloned(&loading_enabled)
            .unwrap_or(false);

        let entries: Vec<shadcn::CommandEntry> = if loading_enabled_value {
            vec![
                shadcn::CommandLoading::new("Fetching commands…")
                    .test_id("ui-gallery-command-loading-row")
                    .into(),
            ]
        } else {
            vec![
                shadcn::CommandGroup::new([
                    shadcn::CommandItem::new("Calendar")
                        .on_select_action(on_select(Arc::from("command.loading.calendar"))),
                    shadcn::CommandItem::new("Search Emoji")
                        .on_select_action(on_select(Arc::from("command.loading.search-emoji"))),
                ])
                .heading("Loaded items")
                .into(),
            ]
        };

        let toggle_row = ui::h_row(|cx| {
            vec![
                shadcn::Checkbox::new(loading_enabled.clone())
                    .control_id("command-loading-enabled")
                    .a11y_label("Loading (demo-only)")
                    .test_id("ui-gallery-command-loading-enabled")
                    .into_element(cx),
                shadcn::FieldLabel::new("Loading (demo-only)")
                    .for_control("command-loading-enabled")
                    .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .items_center()
        .into_element(cx);

        vec![
            toggle_row,
            shadcn::CommandPalette::new(query.clone(), Vec::new())
                .placeholder("Type a command or search...")
                .a11y_label("Command loading demo")
                .entries(entries)
                .test_id_input("ui-gallery-command-loading-input")
                .list_test_id("ui-gallery-command-loading-listbox")
                .test_id_item_prefix("ui-gallery-command-loading-item-")
                .into_element(cx)
                .test_id("ui-gallery-command-loading"),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
