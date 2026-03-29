use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::command as snippets;

pub(super) fn preview_command_palette(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let usage_palette = snippets::usage::render(cx);
    let docs_demo_palette = snippets::docs_demo::render(cx);
    let basic_dialog = snippets::basic::render(cx);
    let composable_shell = snippets::composable_shell::render(cx);
    let shortcuts_section = snippets::shortcuts::render(cx);
    let action_first_view_runtime = snippets::action_first_view::render(cx);
    let behavior_demos = snippets::behavior_demos::render(cx);
    let groups_palette = snippets::groups::render(cx);
    let scrollable_palette = snippets::scrollable::render(cx);
    let rtl = snippets::rtl::render(cx);
    let loading_palette = snippets::loading::render(cx);
    let about = doc_layout::notes_block([
        "shadcn's `Command` is built on `cmdk`: focus stays in the input while the active row is exposed through active-descendant semantics.",
        "Use `CommandPalette` for embedded filtering/search surfaces, and `CommandDialog` for global discovery overlays such as Ctrl/Cmd+P.",
        "Filtering and ranking use the visible label plus optional `value` and `keywords`, so cmdk-style fuzzy matching stays available without depending on DOM internals.",
        "Base UI's command-palette example also treats this space as an autocomplete + dialog composition problem, which reinforces that the remaining gap here is public-surface teaching, not a missing overlay/runtime mechanism.",
    ]);
    let api_reference = doc_layout::notes_block([
        "`command(...)` is the direct visual shell helper. `CommandInput` / `CommandList` stay available for lower-level shell composition and legacy roving lists, but they do not share the cmdk query + active-descendant state machine.",
        "`CommandPalette::new(query, items)` and `.entries(...)` therefore remain the default embedded interactive lane for first-party Fret code when the goal is cmdk-aligned behavior rather than a custom shell.",
        "`CommandDialog::new(open, query, items)` wraps that palette with dialog lifecycle, input placeholder forwarding, close-on-select behavior, and open-change reason hooks for global command menus.",
        "`CommandItem` owns row-level affordances such as `leading_icon(...)`, `shortcut(...)`, `keywords(...)`, `checkmark(...)`, `force_mount(...)`, and `children(...)`.",
        "`CommandItem::children(...)` already covers row-level composability today. The deferred gap is the shared root context that upstream cmdk uses so `CommandInput`, `CommandList`, `CommandEmpty`, and `CommandGroup` can compose without manual query/selection wiring.",
        "`Composable Shell (Fret)` shows the current explicit manual lane: share a query model between `CommandInput` and `CommandList` when you need a custom shell, but keep cmdk-style active-descendant, committed selection, and dialog lifecycle on `CommandPalette` / `CommandDialog`.",
        "A fully composable split `Command` + `CommandInput` + `CommandList` children API is still deferred: upstream cmdk composes those parts through shared internal state, so promoting the same shape in Fret would first require an explicit shared context contract for query, active row, and selection rather than ad-hoc glue.",
    ]);

    let notes_stack = doc_layout::notes_block([
        "Use `CommandDialog` for global discovery (Ctrl/Cmd+P), and keep `CommandPalette` embedded for local filtering surfaces.",
        "`command(...)` / `CommandPalette` remain the default recipe root story; split `CommandInput` / `CommandList` / `CommandItem` authoring stays out of the default surface until a shared context contract is explicitly introduced.",
        "Treat row-level children support and root-level shared-context support as separate questions: `CommandItem::children(...)` already ships, while split root composition still needs an explicit context contract.",
        "No new runtime/mechanism bug was identified in this pass: Base UI and cmdk both support the conclusion that the remaining drift is teaching-surface ergonomics, not missing dialog/focus/dismiss infrastructure.",
        "Attach either `on_select`, `on_select_action`, or `on_select_value_action` for every interactive item; otherwise entries are treated as disabled.",
        "Mirror docs order even when APIs differ so parity gaps stay explicit and testable. For Command, root chrome is recipe-owned while width caps such as `max-w-sm` remain caller-owned.",
        "For long command catalogs, constrain list height via `refine_scroll_layout` to keep dialog geometry stable.",
    ]);
    let notes_stack =
        DocSection::build(cx, "Notes", notes_stack).test_id_prefix("ui-gallery-command-notes");
    let about = DocSection::build(cx, "About", about)
        .no_shell()
        .test_id_prefix("ui-gallery-command-about")
        .description("Outcome-level cmdk/shadcn semantics summary.");
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-command-api-reference")
        .description("Authoring lanes, ownership notes, and the current children-API decision.");
    let composable_shell = DocSection::build(cx, "Composable Shell (Fret)", composable_shell)
        .test_id_prefix("ui-gallery-command-composable-shell")
        .descriptions([
            "Explicit lower-level `Command` + `CommandInput` + `CommandList` composition for custom shells.",
            "This shared-query shell can filter and highlight rows, but it is intentionally not promoted as a cmdk-equivalent children API: active-descendant, active-row state, and dialog lifecycle still belong to `CommandPalette` / `CommandDialog`.",
        ])
        .code_rust_from_file_region(snippets::composable_shell::SOURCE, "example");
    let docs_demo = DocSection::build(cx, "Demo", docs_demo_palette)
        .test_id_prefix("ui-gallery-command-docs-demo")
        .descriptions([
            "This aligns with the shadcn `command-demo` example (icons + disabled item + shortcuts).",
            "The demo follows the public docs example surface (`max-w-sm`, rounded border, copyable example shape), while recipe-owned chrome is validated separately against the registry source.",
            "Use `leading_icon(...)` so icons inherit the row foreground (`currentColor`) for hover/active/disabled states.",
        ])
        .code_rust_from_file_region(snippets::docs_demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage_palette)
        .test_id_prefix("ui-gallery-command-usage")
        .descriptions([
            "This covers the same embedded search outcome as the shadcn `Usage` block, but the default copyable Fret lane is `CommandPalette::new(...)` rather than literal split-children composition.",
            "Keep root chrome recipe-owned, but keep width caps such as `max-w-sm` caller-owned at the usage site.",
        ])
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic_dialog)
        .test_id_prefix("ui-gallery-command-basic")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let shortcuts = DocSection::build(cx, "Shortcuts", shortcuts_section)
        .test_id_prefix("ui-gallery-command-shortcuts")
        .code_rust_from_file_region(snippets::shortcuts::SOURCE, "example");
    let groups = DocSection::build(cx, "Groups", groups_palette)
        .test_id_prefix("ui-gallery-command-groups")
        .code_rust_from_file_region(snippets::groups::SOURCE, "example");
    let scrollable = DocSection::build(cx, "Scrollable", scrollable_palette)
        .test_id_prefix("ui-gallery-command-scrollable")
        .code_rust_from_file_region(snippets::scrollable::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .test_id_prefix("ui-gallery-command-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let behavior_demos = DocSection::build(cx, "Behavior Demos", behavior_demos)
        .test_id_prefix("ui-gallery-command-behavior-demos")
        .descriptions([
            "Fret-only follow-up coverage for cmdk behaviors that do not map 1:1 to a single shadcn docs example.",
            "This keeps `disablePointerSelection`, controlled active value, `shouldFilter=false`, and `forceMount` demos explicit without overloading the docs-aligned examples above.",
        ])
        .code_rust_from_file_region(snippets::behavior_demos::SOURCE, "example");
    let loading = DocSection::build(cx, "Loading", loading_palette)
        .test_id_prefix("ui-gallery-command-loading")
        .descriptions([
            "cmdk supports a non-selectable loading row inside the list (`Command.Loading`).",
            "In Fret this maps to `shadcn::CommandLoading` as an extra `CommandEntry`.",
        ])
        .code_rust_from_file_region(snippets::loading::SOURCE, "example");
    let action_first_view_runtime =
        DocSection::build(cx, "Action-first (View runtime)", action_first_view_runtime)
            .test_id_prefix("ui-gallery-command-action-first-view-runtime")
            .descriptions([
                "This section demonstrates action-first authoring using the ecosystem view runtime (`View` + `AppUi`).",
                "The button binds a stable `ActionId` via `.action(...)`, while the view root stays on the grouped default surface (`cx.state()` + `cx.actions().models::<...>(...)`).",
                "Advanced host-side action-handler cases stay in cookbook/reference docs; the gallery keeps the default teaching path small on purpose.",
            ])
            .code_rust_from_file_region(snippets::action_first_view::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Command docs order after skipping `Installation`: Demo, About, Usage, Basic, Shortcuts, Groups, Scrollable, RTL, API Reference. Fret-specific Composable Shell, Behavior Demos, Loading, and Action-first sections stay explicit follow-ups.",
        ),
        vec![
            docs_demo,
            about,
            usage,
            basic,
            shortcuts,
            groups,
            scrollable,
            rtl,
            api_reference,
            composable_shell,
            behavior_demos,
            loading,
            action_first_view_runtime,
            notes_stack,
        ],
    );

    let body = body.test_id("ui-gallery-command-component");
    vec![body.into_element(cx)]
}
