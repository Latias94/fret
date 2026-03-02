use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::field as snippets;

pub(super) fn preview_field(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let input = snippets::input::render(cx);
    let validation_and_errors = snippets::validation_and_errors::render(cx);
    let textarea = snippets::textarea::render(cx);
    let select = snippets::select::render(cx);
    let slider = snippets::slider::render(cx);
    let fieldset = snippets::fieldset::render(cx);
    let checkbox = snippets::checkbox::render(cx);
    let radio = snippets::radio::render(cx);
    let switch = snippets::switch::render(cx);
    let choice_card = snippets::choice_card::render(cx);
    let field_group = snippets::field_group::render(cx);
    let rtl = snippets::rtl::render(cx);
    let responsive = snippets::responsive::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/field.rs` (Field, FieldSet, FieldGroup, FieldLabel, FieldDescription, FieldSeparator).",
            "Field page follows upstream docs section order for deterministic parity checks.",
            "Each section keeps a stable `test_id` so diag scripts can target specific examples.",
            "RTL and Responsive samples are included to exercise orientation and direction contracts.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Field docs order: Input, Textarea, Select, Slider, Fieldset, Checkbox, Radio, Switch, Choice Card, Field Group, RTL, Responsive Layout.",
        ),
        vec![
            DocSection::new("Input", input)
                .description("Basic text inputs with labels + helper copy.")
                .code_rust_from_file_region(include_str!("../snippets/field/input.rs"), "example"),
            DocSection::new("Validation and Errors", validation_and_errors)
                .description("Field invalid state + control `aria_invalid` styling.")
                .code_rust_from_file_region(
                    include_str!("../snippets/field/validation_and_errors.rs"),
                    "example",
                ),
            DocSection::new("Textarea", textarea)
                .description("Textarea field with explicit height and helper copy.")
                .code_rust_from_file_region(
                    include_str!("../snippets/field/textarea.rs"),
                    "example",
                ),
            DocSection::new("Select", select)
                .description("Select composed inside a Field shell.")
                .code_rust_from_file_region(include_str!("../snippets/field/select.rs"), "example"),
            DocSection::new("Slider", slider)
                .description(
                    "Non-text controls should still use FieldTitle/Description for context.",
                )
                .code_rust_from_file_region(include_str!("../snippets/field/slider.rs"), "example"),
            DocSection::new("Fieldset", fieldset)
                .description("FieldSet groups multiple fields with a legend + description.")
                .code_rust_from_file_region(
                    include_str!("../snippets/field/fieldset.rs"),
                    "example",
                ),
            DocSection::new("Checkbox", checkbox)
                .description("Horizontal Field orientation keeps checkbox + label aligned.")
                .code_rust_from_file_region(
                    include_str!("../snippets/field/checkbox.rs"),
                    "example",
                ),
            DocSection::new("Radio", radio)
                .description("RadioGroup nested under Field for label copy.")
                .code_rust_from_file_region(include_str!("../snippets/field/radio.rs"), "example"),
            DocSection::new("Switch", switch)
                .description("Switch composed with title + description.")
                .code_rust_from_file_region(include_str!("../snippets/field/switch.rs"), "example"),
            DocSection::new("Choice Card", choice_card)
                .description("Choice-card radios combine FieldContent with rich labels.")
                .code_rust_from_file_region(
                    include_str!("../snippets/field/choice_card.rs"),
                    "example",
                ),
            DocSection::new("Field Group", field_group)
                .description("FieldGroup provides separators and checkbox-group composition.")
                .code_rust_from_file_region(
                    include_str!("../snippets/field/field_group.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("All Field compositions should render correctly under RTL direction.")
                .code_rust_from_file_region(include_str!("../snippets/field/rtl.rs"), "example"),
            DocSection::new("Responsive Layout", responsive)
                .description(
                    "Responsive orientation collapses label/content layouts for narrow containers.",
                )
                .code_rust_from_file_region(
                    include_str!("../snippets/field/responsive.rs"),
                    "example",
                ),
            DocSection::new("Notes", notes)
                .description("API reference pointers and stability guidance."),
        ],
    );

    vec![body.test_id("ui-gallery-field")]
}
