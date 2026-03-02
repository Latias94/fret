use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::form as snippets;

pub(super) fn preview_forms(
    cx: &mut ElementContext<'_, App>,
    text_input: Model<String>,
    text_area: Model<String>,
    checkbox: Model<bool>,
    switch: Model<bool>,
) -> Vec<AnyElement> {
    let max_w_md = LayoutRefinement::default().w_full().max_w(Px(520.0));
    let max_w_sm = LayoutRefinement::default()
        .w_full()
        .max_w(Px(384.0))
        .min_w_0();

    let upstream_demo = snippets::upstream_demo::render(cx, max_w_sm.clone());

    let demo = snippets::demo::render(
        cx,
        text_input.clone(),
        text_area.clone(),
        checkbox.clone(),
        switch.clone(),
        max_w_md.clone(),
    );

    let input = snippets::input::render(cx, text_input.clone(), max_w_md.clone());

    let textarea = snippets::textarea::render(cx, text_area.clone(), max_w_md.clone());

    let controls =
        snippets::controls::render(cx, checkbox.clone(), switch.clone(), max_w_md.clone());

    let fieldset =
        snippets::fieldset::render(cx, text_input.clone(), text_area.clone(), max_w_md.clone());

    let rtl = snippets::rtl::render(cx, text_input.clone(), switch.clone(), max_w_md.clone());

    let notes = snippets::notes::render(cx);

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Start with an upstream-aligned FormDemo, then keep a set of gallery recipes for composing Input/Textarea/Checkbox/Switch/FieldSet.",
        ),
        vec![
            DocSection::new("Form Demo", upstream_demo)
                .description("Aligned with shadcn/ui `form-demo.tsx` (new-york-v4).")
                .max_w(Px(840.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/form/upstream_demo.rs"),
                    "example",
                ),
            DocSection::new("Demo", demo)
                .description("FieldSet + FieldGroup recipe with multiple controls.")
                .max_w(Px(840.0))
                .code_rust_from_file_region(include_str!("../snippets/form/demo.rs"), "example"),
            DocSection::new("Input", input)
                .description("A model-bound input control.")
                .max_w(Px(840.0))
                .code_rust_from_file_region(include_str!("../snippets/form/input.rs"), "example"),
            DocSection::new("Textarea", textarea)
                .description("A model-bound textarea control with fixed height.")
                .max_w(Px(840.0))
                .code_rust_from_file_region(include_str!("../snippets/form/textarea.rs"), "example"),
            DocSection::new("Checkbox + Switch", controls)
                .description("Basic checkbox + switch controls with labels.")
                .max_w(Px(840.0))
                .code_rust_from_file_region(include_str!("../snippets/form/controls.rs"), "example"),
            DocSection::new("Fieldset", fieldset)
                .description("FieldSet recipe with grouped fields and action row.")
                .max_w(Px(840.0))
                .code_rust_from_file_region(
                    include_str!("../snippets/form/fieldset.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .description("Form composition under an RTL direction provider.")
                .max_w(Px(840.0))
                .code_rust_from_file_region(include_str!("../snippets/form/rtl.rs"), "example"),
            DocSection::new("Notes", notes)
                .description("API reference pointers and authoring notes.")
                .max_w(Px(820.0))
                .code_rust_from_file_region(include_str!("../snippets/form/notes.rs"), "example"),
        ],
    );

    vec![body.test_id("ui-gallery-form")]
}
