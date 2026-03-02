use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::radio_group as snippets;

pub(super) fn preview_radio_group(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let plans = snippets::plans::render(cx);
    let extras = snippets::extras::render(cx);

    let notes = doc_layout::notes(
        cx,
        ["Preview follows shadcn RadioGroup demo (new-york-v4)."],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn RadioGroup demo order: basic options, plan cards. Extras include invalid/fieldset/RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-radio-group-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Plans", plans)
                .max_w(Px(760.0))
                .test_id_prefix("ui-gallery-radio-group-plans")
                .code_rust_from_file_region(snippets::plans::SOURCE, "example"),
            DocSection::new("Extras", extras)
                .no_shell()
                .test_id_prefix("ui-gallery-radio-group-extras")
                .code_rust_from_file_region(snippets::extras::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-radio-group-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-radio-group")]
}
