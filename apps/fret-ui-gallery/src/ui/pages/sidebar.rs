use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::sidebar as snippets;

pub(super) fn preview_sidebar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let controlled = snippets::controlled::render(cx);
    let mobile = snippets::mobile::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "This is a Fret-specific demo (not part of upstream shadcn sink components).",
            "Keep `test_id_prefix` stable: `tools/diag-scripts/ui-gallery/sidebar/*` depend on DocSection tab trigger IDs.",
            "Mobile example forces `is_mobile(true)` for deterministic overlay + focus-restore diagnostics.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some("A composable, themeable and customizable sidebar component."),
        vec![
            DocSection::new("Demo", demo)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-demo")
                .code_rust_from_file_region(include_str!("../snippets/sidebar/demo.rs"), "example"),
            DocSection::new("Controlled", controlled)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-controlled")
                .code_rust_from_file_region(
                    include_str!("../snippets/sidebar/controlled.rs"),
                    "example",
                ),
            DocSection::new("Mobile", mobile)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-mobile")
                .code_rust_from_file_region(
                    include_str!("../snippets/sidebar/mobile.rs"),
                    "example",
                ),
            DocSection::new("RTL", rtl)
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-rtl")
                .code_rust_from_file_region(include_str!("../snippets/sidebar/rtl.rs"), "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-sidebar-notes"),
        ],
    );

    vec![body]
}

