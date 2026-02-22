use super::super::*;

use crate::ui::doc_layout::{self, DocSection};

pub(super) fn preview_command_palette(
    cx: &mut ElementContext<'_, App>,
    _open: Model<bool>,
    _query: Model<String>,
    last_action: Model<Arc<str>>,
) -> Vec<AnyElement> {
    #[derive(Default)]
    struct CommandPageModels {
        basic_open: Option<Model<bool>>,
        basic_query: Option<Model<String>>,
        shortcuts_query: Option<Model<String>>,
        shortcuts_disable_pointer_selection: Option<Model<bool>>,
        groups_query: Option<Model<String>>,
        scroll_query: Option<Model<String>>,
        rtl_query: Option<Model<String>>,
        demo_filter_query: Option<Model<String>>,
        demo_disable_filtering: Option<Model<bool>>,
        demo_filter_value: Option<Model<Option<Arc<str>>>>,
    }

    let (
        basic_open,
        basic_query,
        shortcuts_query,
        shortcuts_disable_pointer_selection,
        groups_query,
        scroll_query,
        rtl_query,
        demo_filter_query,
        demo_disable_filtering,
        demo_filter_value,
    ) = cx.with_state(CommandPageModels::default, |st| {
        (
            st.basic_open.clone(),
            st.basic_query.clone(),
            st.shortcuts_query.clone(),
            st.shortcuts_disable_pointer_selection.clone(),
            st.groups_query.clone(),
            st.scroll_query.clone(),
            st.rtl_query.clone(),
            st.demo_filter_query.clone(),
            st.demo_disable_filtering.clone(),
            st.demo_filter_value.clone(),
        )
    });

    let (
        basic_open,
        basic_query,
        shortcuts_query,
        shortcuts_disable_pointer_selection,
        groups_query,
        scroll_query,
        rtl_query,
        demo_filter_query,
        demo_disable_filtering,
        demo_filter_value,
    ) = match (
        basic_open,
        basic_query,
        shortcuts_query,
        shortcuts_disable_pointer_selection,
        groups_query,
        scroll_query,
        rtl_query,
        demo_filter_query,
        demo_disable_filtering,
        demo_filter_value,
    ) {
        (
            Some(basic_open),
            Some(basic_query),
            Some(shortcuts_query),
            Some(shortcuts_disable_pointer_selection),
            Some(groups_query),
            Some(scroll_query),
            Some(rtl_query),
            Some(demo_filter_query),
            Some(demo_disable_filtering),
            Some(demo_filter_value),
        ) => (
            basic_open,
            basic_query,
            shortcuts_query,
            shortcuts_disable_pointer_selection,
            groups_query,
            scroll_query,
            rtl_query,
            demo_filter_query,
            demo_disable_filtering,
            demo_filter_value,
        ),
        _ => {
            let basic_open = cx.app.models_mut().insert(false);
            let basic_query = cx.app.models_mut().insert(String::new());
            let shortcuts_query = cx.app.models_mut().insert(String::new());
            let shortcuts_disable_pointer_selection = cx.app.models_mut().insert(false);
            let groups_query = cx.app.models_mut().insert(String::new());
            let scroll_query = cx.app.models_mut().insert(String::new());
            let rtl_query = cx.app.models_mut().insert(String::new());
            let demo_filter_query = cx.app.models_mut().insert(String::new());
            let demo_disable_filtering = cx.app.models_mut().insert(false);
            let demo_filter_value = cx
                .app
                .models_mut()
                .insert(Some(Arc::<str>::from("Calendar")));
            cx.with_state(CommandPageModels::default, |st| {
                st.basic_open = Some(basic_open.clone());
                st.basic_query = Some(basic_query.clone());
                st.shortcuts_query = Some(shortcuts_query.clone());
                st.shortcuts_disable_pointer_selection =
                    Some(shortcuts_disable_pointer_selection.clone());
                st.groups_query = Some(groups_query.clone());
                st.scroll_query = Some(scroll_query.clone());
                st.rtl_query = Some(rtl_query.clone());
                st.demo_filter_query = Some(demo_filter_query.clone());
                st.demo_disable_filtering = Some(demo_disable_filtering.clone());
                st.demo_filter_value = Some(demo_filter_value.clone());
            });
            (
                basic_open,
                basic_query,
                shortcuts_query,
                shortcuts_disable_pointer_selection,
                groups_query,
                scroll_query,
                rtl_query,
                demo_filter_query,
                demo_disable_filtering,
                demo_filter_value,
            )
        }
    };

    let on_select = |tag: Arc<str>| {
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
    };

    let basic_items = vec![
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
        basic_items.clone().into_iter().map(Into::into).collect();
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

    let basic_dialog =
        shadcn::CommandDialog::new(basic_open.clone(), basic_query.clone(), basic_items)
            .a11y_label("Basic command dialog")
            .empty_text("No results found.")
            .into_element(cx, |cx| {
                shadcn::Button::new("Open Command Menu")
                    .variant(shadcn::ButtonVariant::Outline)
                    .toggle_model(basic_open.clone())
                    .test_id("ui-gallery-command-basic-trigger")
                    .into_element(cx)
            })
            .test_id("ui-gallery-command-basic");

    let shortcuts_entries = vec![
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
    let shortcuts_disable_pointer_selection_value = cx
        .app
        .models()
        .get_cloned(&shortcuts_disable_pointer_selection)
        .unwrap_or(false);
    let shortcuts_disable_pointer_selection_for_toggle =
        shortcuts_disable_pointer_selection.clone();
    let demo_disable_filtering_for_toggle = demo_disable_filtering.clone();
    let shortcuts_section = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        move |cx| {
            let toggle_row = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N3).items_center(),
                |cx| {
                    vec![
                        shadcn::Checkbox::new(shortcuts_disable_pointer_selection_for_toggle)
                            .control_id("shortcuts-disable-pointer-selection")
                            .a11y_label("Disable pointer selection (demo-only)")
                            .test_id("ui-gallery-command-shortcuts-disable-pointer-selection")
                            .into_element(cx),
                        shadcn::FieldLabel::new("Disable pointer selection (demo-only)")
                            .for_control("shortcuts-disable-pointer-selection")
                            .into_element(cx),
                    ]
                },
            );

            let palette = shadcn::CommandPalette::new(shortcuts_query.clone(), Vec::new())
                .placeholder("Type a command or search...")
                .a11y_label("Command shortcuts")
                .entries(shortcuts_entries)
                .disable_pointer_selection(shortcuts_disable_pointer_selection_value)
                .test_id_input("ui-gallery-command-shortcuts-input")
                .list_test_id("ui-gallery-command-shortcuts-listbox")
                .test_id_item_prefix("ui-gallery-command-shortcuts-item-")
                .into_element(cx)
                .test_id("ui-gallery-command-shortcuts");

            let demo_disable_filtering_value = cx
                .app
                .models()
                .get_cloned(&demo_disable_filtering)
                .unwrap_or(false);

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

            let demo_toggle_row = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N3).items_center(),
                |cx| {
                    vec![
                        shadcn::Checkbox::new(demo_disable_filtering_for_toggle.clone())
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
                },
            );

            let controlled_selection_row = stack::hstack(
                cx,
                stack::HStackProps::default().gap(Space::N2).items_center(),
                |cx| {
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
                },
            );

            let demo_palette = shadcn::CommandPalette::new(demo_filter_query.clone(), Vec::new())
                .placeholder("Type a command or search... (demo-only)")
                .a11y_label("Command controlled value demo")
                .value(Some(demo_filter_value.clone()))
                .entries(demo_filter_entries.clone())
                .should_filter(!demo_disable_filtering_value)
                .test_id_input("ui-gallery-command-demo-filter-input")
                .list_test_id("ui-gallery-command-demo-filter-listbox")
                .test_id_item_prefix("ui-gallery-command-demo-filter-item-")
                .into_element(cx)
                .test_id("ui-gallery-command-demo-filter");

            let demo_block = stack::vstack(
                cx,
                stack::VStackProps::default()
                    .gap(Space::N2)
                    .items_start()
                    .layout(LayoutRefinement::default().w_full().min_w_0()),
                move |cx| {
                    let demo_filter_value_value = cx
                        .app
                        .models()
                        .get_cloned(&demo_filter_value)
                        .unwrap_or(None);
                    vec![
                        cx.text("Demo-only: controlled cmdk selection value"),
                        cx.text(format!(
                            "selection value (cmdk value): {}",
                            demo_filter_value_value.as_deref().unwrap_or("<none>")
                        )),
                        controlled_selection_row,
                        demo_toggle_row,
                        demo_palette,
                    ]
                },
            );

            vec![toggle_row, palette, demo_block]
        },
    );

    let groups_entries = vec![
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
    let groups_palette = shadcn::CommandPalette::new(groups_query.clone(), Vec::new())
        .placeholder("Search grouped commands...")
        .a11y_label("Command groups")
        .entries(groups_entries)
        .test_id_input("ui-gallery-command-groups-input")
        .list_test_id("ui-gallery-command-groups-listbox")
        .test_id_item_prefix("ui-gallery-command-groups-item-")
        .into_element(cx)
        .test_id("ui-gallery-command-groups");

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

    let scrollable_entries = vec![
        shadcn::CommandGroup::new(recent_items)
            .heading("Recent Files")
            .into(),
        shadcn::CommandSeparator::new().into(),
        shadcn::CommandGroup::new(workspace_items)
            .heading("Workspace")
            .into(),
    ];
    let scrollable_palette = shadcn::CommandPalette::new(scroll_query.clone(), Vec::new())
        .placeholder("Search a long command list...")
        .a11y_label("Scrollable command list")
        .entries(scrollable_entries)
        .test_id_input("ui-gallery-command-scrollable-input")
        .list_test_id("ui-gallery-command-scrollable-listbox")
        .test_id_item_prefix("ui-gallery-command-scrollable-item-")
        .refine_scroll_layout(LayoutRefinement::default().h_px(Px(220.0)).max_h(Px(220.0)))
        .into_element(cx)
        .test_id("ui-gallery-command-scrollable");

    let rtl_entries = vec![
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
    let rtl = doc_layout::rtl(cx, |cx| {
        shadcn::CommandPalette::new(rtl_query.clone(), Vec::new())
            .placeholder("Type a command or search...")
            .a11y_label("RTL command list")
            .entries(rtl_entries)
            .test_id_input("ui-gallery-command-rtl-input")
            .list_test_id("ui-gallery-command-rtl-listbox")
            .test_id_item_prefix("ui-gallery-command-rtl-item-")
            .into_element(cx)
            .test_id("ui-gallery-command-rtl")
    });

    let notes_stack = doc_layout::notes(
        cx,
        [
            "Use `CommandDialog` for global discovery (Ctrl/Cmd+P), and keep `CommandPalette` embedded for local filtering surfaces.",
            "Attach either `on_select` or `on_select_action` for every interactive item; otherwise entries are treated as disabled.",
            "Mirror docs order even when APIs differ so parity gaps stay explicit and testable.",
            "For long command catalogs, constrain list height via `refine_scroll_layout` to keep dialog geometry stable.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Command docs order: Basic, Shortcuts, Groups, Scrollable, RTL.",
        ),
        vec![
            DocSection::new("Basic", basic_dialog)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-command-basic")
                .code(
                    "rust",
                    r#"let dialog = shadcn::CommandDialog::new(open, query, items)
    .a11y_label("Basic command dialog")
    .empty_text("No results found.")
    .into_element(cx, |cx| {
        shadcn::Button::new("Open Command Menu").toggle_model(open).into_element(cx)
    });"#,
                ),
            DocSection::new("Shortcuts", shortcuts_section)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-command-shortcuts")
                .code(
                    "rust",
                    r#"let palette = shadcn::CommandPalette::new(query, Vec::new())
    .entries([
        shadcn::CommandGroup::new([item_a, item_b]).heading("Suggestions").into(),
        shadcn::CommandSeparator::new().into(),
        item_with_shortcut.into(),
    ])
    .into_element(cx);"#,
                ),
            DocSection::new("Groups", groups_palette)
                .max_w(Px(760.0))
                .code(
                    "rust",
                    r#"let entries = vec![
    shadcn::CommandGroup::new([
        shadcn::CommandItem::new("Calendar").keywords(["events"]),
        shadcn::CommandItem::new("Calculator").keywords(["math"]),
    ])
    .heading("Suggestions")
    .into(),
    shadcn::CommandSeparator::new().into(),
    shadcn::CommandGroup::new([
        shadcn::CommandItem::new("Profile").shortcut("Cmd+,"),
        shadcn::CommandItem::new("Settings").shortcut("Cmd+S"),
    ])
    .heading("Settings")
    .into(),
];

shadcn::CommandPalette::new(query, Vec::new())
    .placeholder("Search grouped commands...")
    .entries(entries)
    .into_element(cx);"#,
                ),
            DocSection::new("Scrollable", scrollable_palette)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-command-scrollable")
                .code(
                    "rust",
                    r#"shadcn::CommandPalette::new(query, Vec::new())
    .entries(long_entries)
    .refine_scroll_layout(LayoutRefinement::default().h_px(Px(220.0)).max_h(Px(220.0)))

with_direction_provider(LayoutDirection::Rtl, |cx| {
    shadcn::CommandPalette::new(rtl_query, Vec::new()).entries(rtl_entries).into_element(cx)
});"#,
                ),
            DocSection::new("RTL", rtl).max_w(Px(760.0)).code(
                "rust",
                r#"fret_ui_kit::primitives::direction::with_direction_provider(
    cx,
    fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
    |cx| shadcn::CommandPalette::new(rtl_query, Vec::new()).entries(rtl_entries).into_element(cx),
);"#,
            ),
            DocSection::new("Notes", notes_stack).max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-command-component")]
}
