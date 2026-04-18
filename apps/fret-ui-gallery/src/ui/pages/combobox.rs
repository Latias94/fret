use super::super::*;
use fret::AppComponentCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::combobox as snippets;

pub(super) fn preview_combobox(
    cx: &mut AppComponentCx<'_>,
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
    let popup = snippets::trigger_button::render(cx);
    let multiple = snippets::multiple_selection::render(cx);
    let custom_items = snippets::custom_items::render(cx);
    let long_list = snippets::long_list::render(cx);
    let invalid = snippets::invalid::render(cx);
    let disabled = snippets::disabled::render(cx);
    let input_group = snippets::input_group::render(cx);
    let rtl = snippets::rtl::render(cx);

    let basic = DocSection::build(cx, "Basic", basic)
        .description("Source-aligned base combobox: input trigger with inline search results.")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Copyable minimal direct builder chain for the recipe root lane.")
        .test_id_prefix("ui-gallery-combobox-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let custom_items = DocSection::build(cx, "Custom Items", custom_items)
        .description(
            "Structured item details and richer filtering without widening the item surface to arbitrary root children.",
        )
        .code_rust_from_file_region(snippets::custom_items::SOURCE, "example");
    let multiple = DocSection::build(cx, "Multiple Selection", multiple)
        .description("Docs-aligned chips recipe for multi-select combobox flows.")
        .code_rust_from_file_region(snippets::multiple_selection::SOURCE, "example");
    let clear = DocSection::build(cx, "Clear Button", clear)
        .description("Enable `show_clear` to show a clear affordance when a value is selected.")
        .code_rust_from_file_region(snippets::clear_button::SOURCE, "example");
    let groups = DocSection::build(cx, "Groups", groups)
        .description("Upstream groups items; Fret exposes grouped entries via `ComboboxGroup`.")
        .code_rust_from_file_region(snippets::groups::SOURCE, "example")
        .no_shell();
    let invalid = DocSection::build(cx, "Invalid", invalid)
        .description(
            "Invalid state uses root `Combobox::aria_invalid(true)` so invalid chrome and semantics stay on the combobox surface, then pairs it with caller-owned field/error copy.",
        )
        .code_rust_from_file_region(snippets::invalid::SOURCE, "example");
    let disabled = DocSection::build(cx, "Disabled", disabled)
        .description("Disabled state should block open/selection and use muted styling.")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let auto_highlight = DocSection::build(cx, "Auto Highlight", auto_highlight)
        .description(
            "Base UI opt-in: highlight the first enabled match on open/filter (`autoHighlight`).",
        )
        .code_rust_from_file_region(snippets::auto_highlight::SOURCE, "example")
        .no_shell();
    let popup = DocSection::build(cx, "Popup", popup)
        .description(
            "Matches the Base UI/shadcn popup recipe: a button-like trigger with the searchable input moved into the popup content via typed content parts.",
        )
        .code_rust_from_file_region(snippets::trigger_button::SOURCE, "example")
        .no_shell();
    let input_group = DocSection::build(cx, "Input Group", input_group)
        .description(
            "Typed `ComboboxInput::children([InputGroupAddon...])` mapping for inline addon composition.",
        )
        .code_rust_from_file_region(snippets::input_group::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("All shadcn components should work under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let api_reference = doc_layout::notes_block([
        "Reference stack: shadcn Combobox docs and examples plus the default registry recipe.",
        "Headless contract references: Base UI combobox parts and combobox root semantics.",
        "Radix Primitives does not ship a standalone `Combobox` primitive, so this family aligns shadcn docs/recipes and Base UI headless combobox semantics rather than a dedicated Radix primitive export.",
        "API reference: `ecosystem/fret-ui-shadcn/src/combobox.rs`.",
        "`Combobox::new(value, open)` plus the direct builder chain (`.trigger(...).input(...).clear(...).content(...)`) is the default recipe root lane, while `into_element_parts(...)` stays the focused upstream-shaped patch seam on that same lane rather than a separate `compose()` story.",
        "`Combobox::device_shell_responsive(true)` remains the explicit viewport/device-shell follow-up for the responsive example instead of widening the default docs path, and it stays recipe-owned even though the shell classification now delegates to `fret_ui_kit::adaptive::device_shell_mode(...)`.",
        "`Combobox::required(true)` now covers both the closed trigger surface and the open search input surface, so required semantics follow the actual combobox node across states without widening the recipe to a generic children API.",
        "`Combobox::aria_invalid(true)` is the root invalid lane; callers should not restate invalid state on `ComboboxInput` just to get trigger/search chrome.",
        "Combobox is intentionally a Popover + Command recipe surface; the remaining work here is docs/public-surface drift rather than a `fret-ui` mechanism bug.",
        "Upstream nested children composition maps to typed parts in Fret: `ComboboxContent::new([ComboboxContentPart::...])`, `ComboboxList::{items,groups}`, and `ComboboxInput::children([InputGroupAddon...])` cover the documented lanes without widening the root to arbitrary generic children.",
        "No extra generic root `children(...)` / `compose()` / `asChild` API is warranted here: the documented upstream lanes are already represented by `ComboboxContent::new([...])`, `ComboboxList::{items,groups}`, `ComboboxInput::children([InputGroupAddon...])`, and `ComboboxItem::content(...)`.",
        "`Input Group` demonstrates typed `ComboboxInput::children([InputGroupAddon...])` composition for inline addons; keep that surface narrow instead of widening to generic arbitrary children.",
        "If a future upstream example cannot be expressed through that typed surface, widen the recipe/public surface there first; do not push policy into `fret-ui`.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-combobox-api-reference")
        .description(
            "Public surface summary, ownership notes, and the current children-API decision.",
        );
    let conformance_demo = DocSection::build(cx, "Conformance Demo", conformance_demo)
        .description(
            "Small deterministic surface for `fretboard-dev diag suite ui-gallery-combobox` scripts.",
        )
        .no_shell()
        .code_rust_from_file_region(snippets::conformance_demo::SOURCE, "example");
    let groups_with_separator = DocSection::build(cx, "Groups + Separator", groups_with_separator)
        .description("Fret follow-up for explicit separator coverage between grouped sections.")
        .code_rust_from_file_region(snippets::groups_with_separator::SOURCE, "example")
        .no_shell();
    let label = DocSection::build(cx, "Label Association", label)
        .description("Use `FieldLabel::for_control`, `Combobox::control_id`, and `Combobox::test_id_prefix` so label clicks focus the trigger and keep derived automation anchors stable.")
        .test_id_prefix("ui-gallery-combobox-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let long_list = DocSection::build(cx, "Long List", long_list)
        .description(
            "Large-list follow-up for scroll ownership and future virtualization invariants.",
        )
        .code_rust_from_file_region(snippets::long_list::SOURCE, "example");
    let notes = doc_layout::notes_block([
        "Base UI lifecycle parity already covers `onValueChange`, `onOpenChange`, reason-aware open changes, and transition-complete callbacks.",
        "Multi-select chips is a recipe-level surface (`ComboboxChips`) built on top of Command + Popover primitives.",
        "`Conformance Demo`, `Groups + Separator`, `Label Association`, and `Long List` stay after `API Reference` as explicit Fret follow-ups so the docs path remains readable without losing diagnostics coverage.",
        "For invalid visuals, use root `Combobox::aria_invalid(true)` and pair it with caller-owned field-level error copy.",
        "When adding richer item/group APIs, keep test IDs stable so existing diag scripts remain reusable.",
    ]);
    let notes = DocSection::build(cx, "Notes", notes)
        .test_id_prefix("ui-gallery-combobox-notes")
        .description("Parity notes, follow-up guidance, and diagnostics reminders.");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn/Base UI Combobox docs path after folding the top preview into `Basic` and skipping `Installation`: `Basic`, `Usage`, `Custom Items`, `Multiple Selection`, `Clear Button`, `Groups`, `Invalid`, `Disabled`, `Auto Highlight`, `Popup`, `Input Group`, `RTL`, and `API Reference`. `Conformance Demo`, `Groups + Separator`, `Label Association`, and `Long List` stay as explicit Fret follow-ups.",
        ),
        vec![
            basic,
            usage,
            custom_items,
            multiple,
            clear,
            groups,
            invalid,
            disabled,
            auto_highlight,
            popup,
            input_group,
            rtl,
            api_reference,
            conformance_demo,
            groups_with_separator,
            label,
            long_list,
            notes,
        ],
    );

    vec![body.into_element(cx)]
}
