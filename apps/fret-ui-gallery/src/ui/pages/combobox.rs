use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::combobox as snippets;

pub(super) fn preview_combobox(
    cx: &mut UiCx<'_>,
    value: Model<Option<Arc<str>>>,
    open: Model<bool>,
    query: Model<String>,
) -> Vec<AnyElement> {
    let conformance_demo =
        snippets::conformance_demo::render(cx, value.clone(), open.clone(), query.clone());
    let basic = snippets::basic::render(cx);
    let usage = snippets::usage::render(cx);
    let label = snippets::label::render(cx);
    let auto_highlight = snippets::auto_highlight::render(cx);
    let clear = snippets::clear_button::render(cx);
    let groups = snippets::groups::render(cx);
    let groups_with_separator = snippets::groups_with_separator::render(cx);
    let trigger_button = snippets::trigger_button::render(cx);
    let multiple = snippets::multiple_selection::render(cx);
    let custom_items = snippets::custom_items::render(cx);
    let long_list = snippets::long_list::render(cx);
    let invalid = snippets::invalid::render(cx);
    let disabled = snippets::disabled::render(cx);
    let input_group = snippets::input_group::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/combobox.rs`.",
        "`Combobox::new(value, open)` plus the direct builder chain (`.trigger(...).input(...).clear(...).content(...)`) is the default recipe root lane, while `into_element_parts(...)` stays the focused upstream-shaped patch seam on that same lane rather than a separate `compose()` story.",
        "Combobox is intentionally a Popover + Command recipe surface; it already supports upstream-shaped authoring through `Combobox::into_element_parts(...)` with trigger/input/content patches, so the main parity gap here was usage clarity rather than missing mechanism work.",
        "Upstream nested children composition maps to Fret's typed parts (`ComboboxContent::new([ComboboxContentPart::...])`) rather than arbitrary `AnyElement` children; keep that API narrow unless a concrete upstream outcome is blocked.",
        "`Extras: Input Group` demonstrates typed `ComboboxInput::children([InputGroupAddon...])` composition for inline addons; keep that surface narrow instead of widening to generic arbitrary children.",
        "Multi-select chips is a recipe-level surface (`ComboboxChips`) built on top of Command + Popover primitives.",
        "For invalid visuals today, apply style overrides on trigger and pair with field-level error copy.",
        "When adding richer item/group APIs, keep test IDs stable so existing diag scripts remain reusable.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .test_id_prefix("ui-gallery-combobox-notes")
        .description("Guidelines and parity notes for combobox recipes.");
    let conformance_demo = DocSection::build(cx, "Conformance Demo", conformance_demo)
        .description(
            "Small deterministic surface for `fretboard diag suite ui-gallery-combobox` scripts.",
        )
        .no_shell()
        .code_rust_from_file_region(snippets::conformance_demo::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("Upstream shadcn demo: basic framework combobox with search.")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal direct builder chain for the recipe root lane.")
        .test_id_prefix("ui-gallery-combobox-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association", label)
        .description("Use `FieldLabel::for_control`, `Combobox::control_id`, and `Combobox::test_id_prefix` so label clicks focus the trigger and keep derived automation anchors stable.")
        .test_id_prefix("ui-gallery-combobox-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let auto_highlight = DocSection::build(cx, "Auto Highlight", auto_highlight)
        .description(
            "Base UI opt-in: highlight the first enabled match on open/filter (`autoHighlight`).",
        )
        .code_rust_from_file_region(snippets::auto_highlight::SOURCE, "example")
        .no_shell();
    let clear = DocSection::build(cx, "Clear Button", clear)
        .description("Enable `show_clear` to show a clear affordance when a value is selected.")
        .code_rust_from_file_region(snippets::clear_button::SOURCE, "example");
    let groups = DocSection::build(cx, "Groups", groups)
        .description("Upstream groups items; Fret exposes grouped entries via `ComboboxGroup`.")
        .code_rust_from_file_region(snippets::groups::SOURCE, "example")
        .no_shell();
    let groups_with_separator = DocSection::build(cx, "Groups + Separator", groups_with_separator)
        .description(
            "Use `.group_separators(true)` to insert separators between groups (shadcn `ComboboxSeparator`).",
        )
        .code_rust_from_file_region(snippets::groups_with_separator::SOURCE, "example")
        .no_shell();
    let trigger_button = DocSection::build(cx, "Trigger Button", trigger_button)
        .description("Matches the Base UI/shadcn popup recipe: a button-like trigger with the searchable listbox in the popover content.")
        .code_rust_from_file_region(snippets::trigger_button::SOURCE, "example")
        .no_shell();
    let multiple = DocSection::build(cx, "Multiple Selection", multiple)
        .description(
            "Upstream multi-select chips recipe: select multiple values and remove them via chips.",
        )
        .code_rust_from_file_region(snippets::multiple_selection::SOURCE, "example");
    let custom_items = DocSection::build(cx, "Extras: Custom Items", custom_items)
        .description(
            "Structured item details (e.g. suffix metadata) without pre-formatting richer labels.",
        )
        .code_rust_from_file_region(snippets::custom_items::SOURCE, "example");
    let long_list = DocSection::build(cx, "Extras: Long List", long_list)
        .description(
            "Supports long-list scroll regression gates (and future virtualization invariants).",
        )
        .code_rust_from_file_region(snippets::long_list::SOURCE, "example");
    let invalid = DocSection::build(cx, "Extras: Invalid", invalid)
        .description("Invalid visual uses `aria_invalid(true)` on the combobox trigger.")
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let disabled = DocSection::build(cx, "Extras: Disabled", disabled)
        .description("Disabled state should block open/selection and use muted styling.")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let input_group = DocSection::build(cx, "Extras: Input Group", input_group)
        .description(
            "Typed `ComboboxInput` addon composition plus state rows for diagnostics coverage.",
        )
        .code_rust_from_file_region(snippets::input_group::SOURCE, "example");
    let rtl = DocSection::build(cx, "Extras: RTL", rtl)
        .description("All shadcn components should work under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn ComboboxDemo order, with a small conformance-first section at the top and a copyable Usage example near the front.",
        ),
        vec![
            conformance_demo,
            basic,
            usage,
            label,
            auto_highlight,
            clear,
            groups,
            groups_with_separator,
            trigger_button,
            multiple,
            custom_items,
            long_list,
            invalid,
            disabled,
            input_group,
            rtl,
            notes,
        ],
    );

    vec![body.into_element(cx)]
}
