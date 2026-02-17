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
        groups_query: Option<Model<String>>,
        scroll_query: Option<Model<String>>,
        rtl_query: Option<Model<String>>,
    }

    let (basic_open, basic_query, shortcuts_query, groups_query, scroll_query, rtl_query) = cx
        .with_state(CommandPageModels::default, |st| {
            (
                st.basic_open.clone(),
                st.basic_query.clone(),
                st.shortcuts_query.clone(),
                st.groups_query.clone(),
                st.scroll_query.clone(),
                st.rtl_query.clone(),
            )
        });

    let (basic_open, basic_query, shortcuts_query, groups_query, scroll_query, rtl_query) = match (
        basic_open,
        basic_query,
        shortcuts_query,
        groups_query,
        scroll_query,
        rtl_query,
    ) {
        (
            Some(basic_open),
            Some(basic_query),
            Some(shortcuts_query),
            Some(groups_query),
            Some(scroll_query),
            Some(rtl_query),
        ) => (
            basic_open,
            basic_query,
            shortcuts_query,
            groups_query,
            scroll_query,
            rtl_query,
        ),
        _ => {
            let basic_open = cx.app.models_mut().insert(false);
            let basic_query = cx.app.models_mut().insert(String::new());
            let shortcuts_query = cx.app.models_mut().insert(String::new());
            let groups_query = cx.app.models_mut().insert(String::new());
            let scroll_query = cx.app.models_mut().insert(String::new());
            let rtl_query = cx.app.models_mut().insert(String::new());
            cx.with_state(CommandPageModels::default, |st| {
                st.basic_open = Some(basic_open.clone());
                st.basic_query = Some(basic_query.clone());
                st.shortcuts_query = Some(shortcuts_query.clone());
                st.groups_query = Some(groups_query.clone());
                st.scroll_query = Some(scroll_query.clone());
                st.rtl_query = Some(rtl_query.clone());
            });
            (
                basic_open,
                basic_query,
                shortcuts_query,
                groups_query,
                scroll_query,
                rtl_query,
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
    let shortcuts_palette = shadcn::CommandPalette::new(shortcuts_query.clone(), Vec::new())
        .placeholder("Type a command or search...")
        .a11y_label("Command shortcuts")
        .entries(shortcuts_entries)
        .into_element(cx)
        .test_id("ui-gallery-command-shortcuts");

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
    let rtl_content = fret_ui_kit::primitives::direction::with_direction_provider(
        cx,
        fret_ui_kit::primitives::direction::LayoutDirection::Rtl,
        |cx| {
            shadcn::CommandPalette::new(rtl_query.clone(), Vec::new())
                .placeholder("Type a command or search...")
                .a11y_label("RTL command list")
                .entries(rtl_entries)
                .into_element(cx)
                .test_id("ui-gallery-command-rtl")
        },
    );

    let last = cx
        .app
        .models()
        .get_cloned(&last_action)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));

    let notes = stack::vstack(
        cx,
        stack::VStackProps::default()
            .gap(Space::N2)
            .items_start()
            .layout(LayoutRefinement::default().w_full().min_w_0()),
        |cx| {
            vec![
                shadcn::typography::muted(
                    cx,
                    "API reference: `ecosystem/fret-ui-shadcn/src/command.rs`.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Use `CommandDialog` for global discovery (Ctrl/Cmd+P), and keep `CommandPalette` embedded for local filtering surfaces.",
                ),
                shadcn::typography::muted(
                    cx,
                    "Attach either `on_select` or `on_select_action` for every interactive item; otherwise entries are treated as disabled.",
                ),
                shadcn::typography::muted(
                    cx,
                    "For long command catalogs, constrain list height via `refine_scroll_layout` to keep dialog geometry stable.",
                ),
                shadcn::typography::muted(cx, Arc::from(format!("Last action: {last}"))),
            ]
        },
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Command docs order: Basic, Shortcuts, Groups, Scrollable, RTL.",
        ),
        vec![
            DocSection::new("Basic", basic_dialog)
                .description("CommandDialog with a trigger button and an overlay list.")
                .code(
                    "rust",
                    r#"let dialog = shadcn::CommandDialog::new(open, query, items)
    .a11y_label("Basic command dialog")
    .empty_text("No results found.")
    .into_element(cx, |cx| {
        shadcn::Button::new("Open Command Menu").toggle_model(open).into_element(cx)
    });"#,
                )
                .max_w(Px(760.0)),
            DocSection::new("Shortcuts", shortcuts_palette)
                .description("Inline CommandPalette entries with visible keyboard shortcuts.")
                .max_w(Px(760.0)),
            DocSection::new("Groups", groups_palette)
                .description("Group headings + separators for structured command catalogs.")
                .max_w(Px(760.0)),
            DocSection::new("Scrollable", scrollable_palette)
                .description(
                    "Constrain list height to keep layouts stable while filtering long lists.",
                )
                .max_w(Px(760.0)),
            DocSection::new("RTL", rtl_content)
                .description(
                    "Direction provider sample to validate RTL layout and shortcut rendering.",
                )
                .max_w(Px(760.0)),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .max_w(Px(820.0)),
        ],
    );

    vec![body.test_id("ui-gallery-command-component")]
}
