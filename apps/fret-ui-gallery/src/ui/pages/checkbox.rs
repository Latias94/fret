use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::checkbox as snippets;

pub(super) fn preview_checkbox(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let checked_state = snippets::checked_state::render(cx);
    let label = snippets::label::render(cx);
    let invalid_state = snippets::invalid_state::render(cx);
    let basic = snippets::basic::render(cx);
    let description_section = snippets::description::render(cx);
    let disabled_section = snippets::disabled::render(cx);
    let with_title_section = snippets::with_title::render(cx);
    let group = snippets::group::render(cx);
    let table = snippets::table::render(cx);
    let rtl_section = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "Upstream docs path: `repo-ref/ui/apps/v4/content/docs/components/base/checkbox.mdx`.",
        "`Checkbox::new(...)`, `Checkbox::new_optional(...)`, and `Checkbox::new_tristate(...)` cover the model-backed checked and mixed-state paths; all three lanes now accept the narrow checked-state bridge traits, while `Checkbox::from_checked(...)` / `from_checked_state(...)` plus `.action(...)` cover the default source-aligned snapshot/action path. `.on_click(...)` remains the lower-level command bridge when explicit command routing is genuinely needed.",
        "Checkbox remains a leaf control surface: labels, descriptions, and larger click targets are composed through `Field`, `FieldContent`, `FieldLabel::for_control(...)`, and `FieldLabel::wrap(...)` rather than a generic children/`compose()` API on the checkbox itself.",
        "Visual defaults such as control size, border, focus ring, and indicator chrome stay recipe-owned, while row width and form layout remain caller-owned.",
        "The docs-aligned `Description`, `Group`, and `Table` sections now keep the upstream row order, fieldset framing, and mixed select-all teaching surface visible on the page instead of hiding them behind unrelated composition shortcuts.",
        "`Label Association` and `With Title` stay after the upstream docs path because they document Fret-specific control-registry and wrapped-field composition patterns.",
        "This page is docs/public-surface parity work, not a mechanism-layer fix.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .description("Public surface summary, docs-parity notes, and children API ownership.");

    let demo = DocSection::build(cx, "Demo", demo)
        .description("Single checkbox with a label.")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Copyable minimal usage for `Checkbox`.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let checked_state = DocSection::build(cx, "Checked State", checked_state)
        .description(
            "Controlled checked model, optional/indeterminate model, and source-aligned snapshot/action path.",
        )
        .code_rust_from_file_region(snippets::checked_state::SOURCE, "example");
    let invalid_state = DocSection::build(cx, "Invalid State", invalid_state)
        .description(
            "Invalid styling uses `aria_invalid` on the checkbox and destructive field text.",
        )
        .code_rust_from_file_region(snippets::invalid_state::SOURCE, "example");
    let basic = DocSection::build(cx, "Basic", basic)
        .description("Field plus checkbox plus label composition.")
        .code_rust_from_file_region(snippets::basic::SOURCE, "example");
    let description_section = DocSection::build(cx, "Description", description_section)
        .description(
            "Checkbox plus label and helper text aligned like the upstream description example.",
        )
        .code_rust_from_file_region(snippets::description::SOURCE, "example");
    let disabled_section = DocSection::build(cx, "Disabled", disabled_section)
        .description("Disabled checkboxes block interaction and use muted styling.")
        .code_rust_from_file_region(snippets::disabled::SOURCE, "example");
    let group = DocSection::build(cx, "Group", group)
        .description("Fieldset + legend + checkbox list pattern from the upstream docs.")
        .code_rust_from_file_region(snippets::group::SOURCE, "example");
    let table = DocSection::build(cx, "Table", table)
        .description("Table selection pattern with a derived mixed-state select-all checkbox on the action-first path.")
        .code_rust_from_file_region(snippets::table::SOURCE, "example");
    let rtl_section = DocSection::build(cx, "RTL", rtl_section)
        .description("Checkbox and label alignment under an RTL direction provider.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let label = DocSection::build(cx, "Label Association (Fret)", label)
        .description("Use `FieldLabel::for_control` plus `Checkbox::control_id` so label clicks toggle the checkbox.")
        .test_id_prefix("ui-gallery-checkbox-label")
        .code_rust_from_file_region(snippets::label::SOURCE, "example");
    let with_title_section = DocSection::build(cx, "With Title (Fret)", with_title_section)
        .description("`FieldLabel` can wrap a full field layout for card-style checkbox rows.")
        .code_rust_from_file_region(snippets::with_title::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Checkbox docs path first, surfaces the source-aligned snapshot/action story in `API Reference`, then keeps `Label Association` and `With Title` as focused Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            checked_state,
            invalid_state,
            basic,
            description_section,
            disabled_section,
            group,
            table,
            rtl_section,
            api_reference,
            label,
            with_title_section,
        ],
    );

    vec![body.test_id("ui-gallery-checkbox").into_element(cx)]
}
