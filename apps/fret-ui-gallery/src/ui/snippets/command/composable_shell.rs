pub const SOURCE: &str = include_str!("composable_shell.rs");

// region: example
use std::sync::Arc;

use fret::{AppComponentCx, UiChild};
use fret_core::Px;
use fret_ui::Invalidation;
use fret_ui_shadcn::{facade as shadcn, prelude::*};

pub fn render(cx: &mut AppComponentCx<'_>) -> impl UiChild + use<> {
    let query = cx.local_model_keyed("query", String::new);
    let last_action = super::last_action_model(cx);
    let last_action_value = cx
        .get_model_cloned(&last_action, Invalidation::Layout)
        .unwrap_or_else(|| Arc::<str>::from("<none>"));
    let on_select = super::on_select_for_last_action(last_action.clone());

    ui::v_flex(move |cx| {
        let entries: Vec<shadcn::CommandEntry> = vec![
            shadcn::CommandGroup::new([
                shadcn::CommandItem::new("Calendar")
                    .keywords(["events", "schedule"])
                    .on_select_action(on_select(Arc::from("command.composable-shell.calendar"))),
                shadcn::CommandItem::new("Search Emoji")
                    .keywords(["emoji", "symbols"])
                    .on_select_action(on_select(Arc::from(
                        "command.composable-shell.search-emoji",
                    ))),
                shadcn::CommandItem::new("Calculator")
                    .shortcut("⌘K")
                    .on_select_action(on_select(Arc::from("command.composable-shell.calculator"))),
            ])
            .heading("Suggestions")
            .into(),
            shadcn::CommandSeparator::new().into(),
            shadcn::CommandGroup::new([
                shadcn::CommandItem::new("Profile")
                    .shortcut("⌘P")
                    .on_select_action(on_select(Arc::from("command.composable-shell.profile"))),
                shadcn::CommandItem::new("Billing")
                    .shortcut("⌘B")
                    .on_select_action(on_select(Arc::from("command.composable-shell.billing"))),
                shadcn::CommandItem::new("Settings")
                    .shortcut("⌘S")
                    .on_select_action(on_select(Arc::from("command.composable-shell.settings"))),
            ])
            .heading("Settings")
            .into(),
        ];

        let shell = shadcn::Command::new(vec![
            shadcn::CommandInput::new(query.clone())
                .a11y_label("Command shell input")
                .placeholder("Type a command or search...")
                .input_test_id("ui-gallery-command-composable-shell-input")
                .into_element(cx),
            shadcn::CommandList::new_entries(entries)
                .query_model(query.clone())
                .highlight_query_model(query.clone())
                .empty_text("No results found.")
                .refine_scroll_layout(
                    LayoutRefinement::default()
                        .w_full()
                        .min_w_0()
                        .max_h(Px(240.0)),
                )
                .into_element(cx)
                .test_id("ui-gallery-command-composable-shell-list"),
        ])
        .refine_layout(
            LayoutRefinement::default()
                .w_full()
                .max_w(Px(384.0))
                .min_w_0(),
        )
        .into_element(cx)
        .test_id("ui-gallery-command-composable-shell");

        vec![
            shell,
            cx.text(format!("Last action: {last_action_value}"))
                .test_id("ui-gallery-command-composable-shell-last-action"),
        ]
    })
    .gap(Space::N2)
    .items_start()
    .layout(LayoutRefinement::default().w_full().min_w_0())
    .into_element(cx)
}
// endregion: example
