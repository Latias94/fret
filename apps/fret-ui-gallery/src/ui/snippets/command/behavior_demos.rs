pub const SOURCE: &str = include_str!("behavior_demos.rs");

// region: example
use fret::{AppComponentCx, UiChild};
use fret_ui::Invalidation;
use fret_ui_shadcn::{facade as shadcn, prelude::*};
use std::sync::Arc;

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let noop: fret_ui::action::OnActivate = Arc::new(|_host, _action_cx, _reason| {});
    let behavior_query = cx.local_model_keyed("behavior_query", String::new);
    let behavior_disable_pointer_selection =
        cx.local_model_keyed("behavior_disable_pointer_selection", || false);
    let behavior_group_query = cx.local_model_keyed("behavior_group_query", String::new);
    let demo_filter_query = cx.local_model_keyed("demo_filter_query", String::new);
    let demo_disable_filtering = cx.local_model_keyed("demo_disable_filtering", || false);
    let demo_filter_value =
        cx.local_model_keyed("demo_filter_value", || Some(Arc::<str>::from("Calendar")));
    let demo_group_force_query = cx.local_model_keyed("demo_group_force_query", String::new);

    let behavior_disable_pointer_selection_value = cx
        .get_model_copied(&behavior_disable_pointer_selection, Invalidation::Layout)
        .unwrap_or(false);
    let demo_disable_filtering_value = cx
        .get_model_copied(&demo_disable_filtering, Invalidation::Layout)
        .unwrap_or(false);
    let demo_filter_value_value = cx
        .get_model_cloned(&demo_filter_value, Invalidation::Layout)
        .unwrap_or(None);

    ui::v_flex(move |cx| {
        let behavior_toggle_row = ui::h_row(|cx| {
            vec![
                shadcn::Checkbox::new(behavior_disable_pointer_selection.clone())
                    .control_id("behavior-disable-pointer-selection")
                    .a11y_label("Disable pointer selection (demo-only)")
                    .test_id("ui-gallery-command-behavior-disable-pointer-selection")
                    .into_element(cx),
                shadcn::FieldLabel::new("Disable pointer selection (demo-only)")
                    .for_control("behavior-disable-pointer-selection")
                    .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .items_center()
        .into_element(cx);

        let behavior_entries: Vec<shadcn::CommandEntry> = vec![
            shadcn::CommandItem::new("Open Project")
                .shortcut("Cmd+O")
                .keywords(["workspace", "folder"])
                .on_select_action(noop.clone())
                .into(),
            shadcn::CommandItem::new("Toggle Sidebar")
                .shortcut("Cmd+B")
                .keywords(["panel", "layout"])
                .on_select_action(noop.clone())
                .into(),
            shadcn::CommandItem::new("Go to File")
                .shortcut("Cmd+P")
                .keywords(["quick open", "jump"])
                .on_select_action(noop.clone())
                .into(),
            shadcn::CommandItem::new("Toggle Terminal")
                .shortcut("Cmd+J")
                .keywords(["console", "output"])
                .on_select_action(noop.clone())
                .into(),
        ];

        let behavior_palette = shadcn::CommandPalette::new(behavior_query.clone(), Vec::new())
            .placeholder("Type a command or search... (demo-only)")
            .a11y_label("Command behavior demo")
            .entries(behavior_entries)
            .disable_pointer_selection(behavior_disable_pointer_selection_value)
            .test_id_input("ui-gallery-command-behavior-input")
            .list_test_id("ui-gallery-command-behavior-listbox")
            .test_id_item_prefix("ui-gallery-command-behavior-item-")
            .into_element(cx)
            .test_id("ui-gallery-command-behavior");

        let icon_id = fret_icons::IconId::new_static;
        let behavior_group_entries = vec![
            shadcn::CommandGroup::new([
                shadcn::CommandItem::new("Calendar")
                    .leading_icon(icon_id("lucide.calendar"))
                    .on_select_action(noop.clone()),
                shadcn::CommandItem::new("Search Emoji")
                    .leading_icon(icon_id("lucide.smile"))
                    .on_select_action(noop.clone()),
                shadcn::CommandItem::new("Calculator")
                    .leading_icon(icon_id("lucide.calculator"))
                    .on_select_action(noop.clone()),
            ])
            .heading("Suggestions")
            .into(),
            shadcn::CommandSeparator::new().into(),
            shadcn::CommandGroup::new([
                shadcn::CommandItem::new("Profile")
                    .leading_icon(icon_id("lucide.user"))
                    .shortcut("⌘P")
                    .on_select_action(noop.clone()),
                shadcn::CommandItem::new("Billing")
                    .leading_icon(icon_id("lucide.credit-card"))
                    .shortcut("⌘B")
                    .on_select_action(noop.clone()),
                shadcn::CommandItem::new("Settings")
                    .leading_icon(icon_id("lucide.settings"))
                    .shortcut("⌘S")
                    .on_select_action(noop.clone()),
            ])
            .heading("Settings")
            .into(),
        ];

        let behavior_group_palette =
            shadcn::CommandPalette::new(behavior_group_query.clone(), Vec::new())
                .placeholder("Type to test grouped navigation... (demo-only)")
                .a11y_label("Command grouped navigation demo")
                .entries(behavior_group_entries)
                .test_id_input("ui-gallery-command-behavior-groups-input")
                .list_test_id("ui-gallery-command-behavior-groups-listbox")
                .test_id_item_prefix("ui-gallery-command-behavior-groups-item-")
                .into_element(cx)
                .test_id("ui-gallery-command-behavior-groups");

        let set_demo_selection = |next: Option<Arc<str>>| {
            let demo_filter_value = demo_filter_value.clone();
            Arc::new(
                move |host: &mut dyn fret_ui::action::UiActionHost,
                      action_cx: fret_ui::action::ActionCx,
                      _reason: fret_ui::action::ActivateReason| {
                    let next = next.clone();
                    let _ = host
                        .models_mut()
                        .update(&demo_filter_value, |current: &mut Option<Arc<str>>| {
                            *current = next;
                        });
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
                shadcn::FieldLabel::new("Disable filtering (shouldFilter=false) (demo-only)")
                    .for_control("demo-disable-filtering")
                    .into_element(cx),
            ]
        })
        .gap(Space::N3)
        .items_center()
        .into_element(cx);

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
        })
        .gap(Space::N2)
        .items_center()
        .into_element(cx);

        let demo_filter_entries = vec![
            shadcn::CommandItem::new("Calendar")
                .shortcut("Cmd+C")
                .keywords(["events", "schedule"])
                .on_select_action(noop.clone())
                .into(),
            shadcn::CommandItem::new("Search Emoji")
                .shortcut("Cmd+E")
                .keywords(["emoji", "insert"])
                .on_select_action(noop.clone())
                .into(),
            shadcn::CommandItem::new("Calculator")
                .shortcut("Cmd+K")
                .keywords(["math", "calc"])
                .on_select_action(noop.clone())
                .into(),
            shadcn::CommandSeparator::new()
                .always_render(true)
                .test_id("ui-gallery-command-demo-filter-separator")
                .into(),
            shadcn::CommandItem::new("Force mounted row (cmdk forceMount)")
                .value("force-mounted")
                .force_mount(true)
                .on_select_action(noop.clone())
                .into(),
        ];

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
                shadcn::CommandItem::new("Alpha").on_select_action(noop.clone()),
                shadcn::CommandItem::new("Beta").on_select_action(noop.clone()),
            ])
            .heading("Letters")
            .force_mount(true)
            .into(),
            shadcn::CommandSeparator::new().into(),
            shadcn::CommandGroup::new([shadcn::CommandItem::new("Giraffe")
                .on_select_action(noop.clone())])
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

        let controlled_value_block = ui::v_flex(|cx| {
            vec![
                cx.text("Controlled selection demo (cmdk `value`)"),
                cx.text(format!(
                    "active value: {}",
                    demo_filter_value_value.as_deref().unwrap_or("<none>")
                )),
                controlled_selection_row,
                demo_toggle_row,
                demo_palette,
                cx.text(
                    "Demo-only: cmdk `Group forceMount` keeps headings visible even when all items are filtered out.",
                ),
                group_force_palette,
            ]
        })
        .gap(Space::N2)
        .items_start()
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .into_element(cx);

        vec![
            cx.text("disablePointerSelection + vim/home/end navigation"),
            behavior_toggle_row,
            behavior_palette,
            cx.text("Grouped keyboard navigation"),
            behavior_group_palette,
            controlled_value_block,
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
