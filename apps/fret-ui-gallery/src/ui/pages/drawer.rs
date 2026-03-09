use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::drawer as snippets;

pub(super) fn preview_drawer(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let snap_points = snippets::snap_points::render(cx);
    let scrollable_content = snippets::scrollable_content::render(cx);
    let sides = snippets::sides::render(cx);
    let responsive_dialog = snippets::responsive_dialog::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Gallery sections mirror shadcn Drawer docs first; `Snap Points` is a Fret/Vaul-specific extra appended afterward.",
            "`Drawer::compose()` is a recipe-level bridge for shadcn-style part composition without pushing children API concerns into the mechanism layer.",
            "`Drawer::direction(...)` now mirrors the upstream Vaul/shadcn prop name; `side(...)` remains as a compatibility escape hatch.",
            "Responsive dialog recipe is represented as explicit desktop/mobile branches for deterministic gallery validation.",
            "Use stable test IDs on every scenario so diag scripts can capture open/close and layout outcomes reliably.",
            "`DrawerClose::from_scope().build(cx, child)` gives a composable close-child surface that maps more directly to upstream `DrawerClose asChild` usage.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Drawer docs order first, then appends a Fret-specific `Snap Points` recipe.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Basic drawer with header copy and footer actions.")
                .test_id_prefix("ui-gallery-drawer")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Copyable shadcn-style composition reference for Drawer.")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example"),
            DocSection::new("Scrollable Content", scrollable_content)
                .description("Keep actions visible while the content area scrolls.")
                .code_rust_from_file_region(snippets::scrollable_content::SOURCE, "example"),
            DocSection::new("Sides", sides)
                .description("Use the `direction` prop to control drawer placement.")
                .code_rust_from_file_region(snippets::sides::SOURCE, "example"),
            DocSection::new("Responsive Dialog", responsive_dialog)
                .descriptions([
                    "Responsive patterns often use Dialog on desktop and Drawer on mobile.",
                    "Gallery renders both branches explicitly for deterministic testing (no viewport switches).",
                ])
                .code_rust_from_file_region(snippets::responsive_dialog::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Drawer layout should follow right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Snap Points", snap_points)
                .description("Drag settles to the nearest snap point (Vaul-style).")
                .code_rust_from_file_region(snippets::snap_points::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .title_test_id("ui-gallery-section-notes-title")
                .description("Implementation notes and regression guidelines."),
        ],
    );

    vec![body]
}
