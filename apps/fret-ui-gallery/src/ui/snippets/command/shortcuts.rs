pub const SOURCE: &str = include_str!("shortcuts.rs");

// region: example
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

fn on_select_for_last_action(
    last_action: Model<Arc<str>>,
) -> impl Fn(Arc<str>) -> fret_ui::action::OnActivate + Clone {
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
}

pub fn render<H: UiHost>(
    cx: &mut ElementContext<'_, H>,
    last_action: Model<Arc<str>>,
) -> AnyElement {
    let query = cx.local_model_keyed("query", String::new);
    let disable_pointer_selection = cx.local_model_keyed("disable_pointer_selection", || false);
    let demo_filter_query = cx.local_model_keyed("demo_filter_query", String::new);
    let demo_disable_filtering = cx.local_model_keyed("demo_disable_filtering", || false);
    let demo_filter_value =
        cx.local_model_keyed("demo_filter_value", || Some(Arc::<str>::from("Calendar")));
    let demo_group_force_query = cx.local_model_keyed("demo_group_force_query", String::new);

    let on_select = on_select_for_last_action(last_action.clone());

    ui::v_flex(move |cx: &mut ElementContext<'_, H>| {
            let disable_pointer_selection_value = cx
                .app
                .models()
                .get_cloned(&disable_pointer_selection)
                .unwrap_or(false);
            let demo_disable_filtering_value = cx
                .app
                .models()
                .get_cloned(&demo_disable_filtering)
                .unwrap_or(false);

            let toggle_row = ui::h_row(|cx| {
                    vec![
                        shadcn::Checkbox::new(disable_pointer_selection.clone())
                            .control_id("shortcuts-disable-pointer-selection")
                            .a11y_label("Disable pointer selection (demo-only)")
                            .test_id("ui-gallery-command-shortcuts-disable-pointer-selection")
                            .into_element(cx),
                        shadcn::FieldLabel::new("Disable pointer selection (demo-only)")
                            .for_control("shortcuts-disable-pointer-selection")
                            .into_element(cx),
                    ]
                }).gap(Space::N3).items_center().into_element(cx);

            let shortcuts_entries: Vec<shadcn::CommandEntry> = vec![
                shadcn::CommandItem::new("Open Project")
                    .shortcut("Cmd+O")
                    .keywords(["workspace", "folder"])
                    .on_select_action(on_select(Arc::from("command.shortcuts.open-project")))
                    .into(),
                shadcn::CommandItem::new("Toggle Sidebar")
                    .shortcut("Cmd+B")
                    .keywords(["panel", "layout"])
                    .on_select_action(on_select(Arc::from("command.shortcuts.toggle-sidebar")))
                    .into(),
                shadcn::CommandItem::new("Go to File")
                    .shortcut("Cmd+P")
                    .keywords(["quick open", "jump"])
                    .on_select_action(on_select(Arc::from("command.shortcuts.goto-file")))
                    .into(),
                shadcn::CommandItem::new("Toggle Terminal")
                    .shortcut("Cmd+J")
                    .keywords(["console", "output"])
                    .on_select_action(on_select(Arc::from("command.shortcuts.toggle-terminal")))
                    .into(),
            ];

            let palette = shadcn::CommandPalette::new(query.clone(), Vec::new())
                .placeholder("Type a command or search...")
                .a11y_label("Command shortcuts")
                .entries(shortcuts_entries)
                .disable_pointer_selection(disable_pointer_selection_value)
                .test_id_input("ui-gallery-command-shortcuts-input")
                .list_test_id("ui-gallery-command-shortcuts-listbox")
                .test_id_item_prefix("ui-gallery-command-shortcuts-item-")
                .into_element(cx)
                .test_id("ui-gallery-command-shortcuts");

            let set_demo_selection = |next: Option<Arc<str>>| {
                let demo_filter_value = demo_filter_value.clone();
                Arc::new(
                    move |host: &mut dyn fret_ui::action::UiActionHost,
                          action_cx: fret_ui::action::ActionCx,
                          _reason: fret_ui::action::ActivateReason| {
                        let next = next.clone();
                        let _ = host.models_mut().update(
                            &demo_filter_value,
                            |cur: &mut Option<Arc<str>>| {
                                *cur = next;
                            },
                        );
                        host.request_redraw(action_cx.window);
                    },
                ) as fret_ui::action::OnActivate
            };

            let demo_toggle_row = ui::h_row(|cx| {
                    vec![
                        shadcn::Checkbox::new(demo_disable_filtering.clone())
                            .control_id("demo-disable-filtering")
                            .a11y_label("Disable filtering (shouldFilter=false) (demo-only)")
                            .test_id("ui-gallery-command-demo-disable-filtering")
                            .into_element(cx),
                        shadcn::FieldLabel::new(
                            "Disable filtering (shouldFilter=false) (demo-only)",
                        )
                        .for_control("demo-disable-filtering")
                        .into_element(cx),
                    ]
                }).gap(Space::N3).items_center().into_element(cx);

            let controlled_selection_row = ui::h_row(|cx| {
                    vec![
                        shadcn::Button::new("Select Calendar")
                            .variant(shadcn::ButtonVariant::Outline)
                            .on_activate(set_demo_selection(Some(Arc::from("Calendar"))))
                            .test_id("ui-gallery-command-demo-selection-set-calendar")
                            .into_element(cx),
                        shadcn::Button::new("Select Search Emoji")
                            .variant(shadcn::ButtonVariant::Outline)
                            .on_activate(set_demo_selection(Some(Arc::from("Search Emoji"))))
                            .test_id("ui-gallery-command-demo-selection-set-search-emoji")
                            .into_element(cx),
                        shadcn::Button::new("Select Calculator")
                            .variant(shadcn::ButtonVariant::Outline)
                            .on_activate(set_demo_selection(Some(Arc::from("Calculator"))))
                            .test_id("ui-gallery-command-demo-selection-set-calculator")
                            .into_element(cx),
                    ]
                }).gap(Space::N2).items_center().into_element(cx);

            let basic_items: Vec<shadcn::CommandItem> = vec![
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

            let mut demo_filter_entries: Vec<shadcn::CommandEntry> =
                basic_items.into_iter().map(Into::into).collect();
            demo_filter_entries.push(
                shadcn::CommandSeparator::new()
                    .always_render(true)
                    .test_id("ui-gallery-command-demo-filter-separator")
                    .into(),
            );
            demo_filter_entries.push(
                shadcn::CommandItem::new("Force mounted row (cmdk forceMount)")
                    .value("force-mounted")
                    .force_mount(true)
                    .on_select_action(on_select(Arc::from("command.demo.force-mount")))
                    .into(),
            );

            let demo_palette = shadcn::CommandPalette::new(demo_filter_query.clone(), Vec::new())
                .placeholder("Type a command or search... (demo-only)")
                .a11y_label("Command controlled value demo")
                .value(Some(demo_filter_value.clone()))
                .entries(demo_filter_entries)
                .should_filter(!demo_disable_filtering_value)
                .test_id_input("ui-gallery-command-demo-filter-input")
                .list_test_id("ui-gallery-command-demo-filter-listbox")
                .test_id_item_prefix("ui-gallery-command-demo-filter-item-")
                .into_element(cx)
                .test_id("ui-gallery-command-demo-filter");

            let group_force_entries = vec![
                shadcn::CommandGroup::new([
                    shadcn::CommandItem::new("Alpha").on_select_value_action({
                        let last_action = last_action.clone();
                        move |host, action_cx, _reason, value| {
                            let msg = Arc::<str>::from(format!("command.group_force: {value}"));
                            let _ = host.models_mut().update(&last_action, |cur| {
                                *cur = msg.clone();
                            });
                            host.request_redraw(action_cx.window);
                        }
                    }),
                    shadcn::CommandItem::new("Beta").on_select_value_action({
                        let last_action = last_action.clone();
                        move |host, action_cx, _reason, value| {
                            let msg = Arc::<str>::from(format!("command.group_force: {value}"));
                            let _ = host.models_mut().update(&last_action, |cur| {
                                *cur = msg.clone();
                            });
                            host.request_redraw(action_cx.window);
                        }
                    }),
                ])
                .heading("Letters")
                .force_mount(true)
                .into(),
                shadcn::CommandSeparator::new().into(),
                shadcn::CommandGroup::new([shadcn::CommandItem::new("Giraffe")
                    .on_select_value_action({
                        let last_action = last_action.clone();
                        move |host, action_cx, _reason, value| {
                            let msg = Arc::<str>::from(format!("command.group_force: {value}"));
                            let _ = host.models_mut().update(&last_action, |cur| {
                                *cur = msg.clone();
                            });
                            host.request_redraw(action_cx.window);
                        }
                    })])
                .heading("Animals")
                .into(),
            ];
            let group_force_palette =
                shadcn::CommandPalette::new(demo_group_force_query.clone(), Vec::new())
                    .placeholder("Type to filter groups... (demo-only)")
                    .a11y_label("Command group forceMount demo")
                    .entries(group_force_entries)
                    .test_id_input("ui-gallery-command-group-force-input")
                    .list_test_id("ui-gallery-command-group-force-listbox")
                    .test_id_item_prefix("ui-gallery-command-group-force-item-")
                    .test_id_heading_prefix("ui-gallery-command-group-force-heading-")
                    .into_element(cx)
                    .test_id("ui-gallery-command-group-force");

            let demo_block = ui::v_flex(move |cx: &mut ElementContext<'_, H>| {
                    let demo_filter_value_value = cx
                        .app
                        .models()
                        .get_cloned(&demo_filter_value)
                        .unwrap_or(None);
                    vec![
                        cx.text("Controlled selection demo (cmdk `value`)"),
                        cx.text(format!(
                            "active value: {}",
                            demo_filter_value_value.as_deref().unwrap_or("<none>")
                        )),
                        controlled_selection_row,
                        demo_toggle_row,
                        demo_palette,
                        cx.text("Demo-only: cmdk `Group forceMount` keeps headings visible even when all items are filtered out."),
                        group_force_palette,
                    ]
                })
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx);

            vec![toggle_row, palette, demo_block]
        })
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()).into_element(cx)
}
// endregion: example
