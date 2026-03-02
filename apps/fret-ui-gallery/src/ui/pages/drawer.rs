use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::drawer as snippets;

pub(super) fn preview_drawer(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let snap_points = snippets::snap_points::render(cx);
    let scrollable_content = snippets::scrollable_content::render(cx);
    let sides = snippets::sides::render(cx);
    let responsive_dialog = snippets::responsive_dialog::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Docs parity follows the upstream order: scrollable content and sides are explicit recipes after the basic demo.",
            "Responsive dialog recipe is represented as explicit desktop/mobile branches for deterministic gallery validation.",
            "Use stable test IDs on every scenario so diag scripts can capture open/close and layout outcomes reliably.",
            "DrawerClose-as-child composition is not modeled yet; current examples close through toggle_model actions.",
        ],
    );

    let usage = doc_layout::notes(
        cx,
        [
            "See the Preview sections above for the canonical shadcn-aligned recipes (Demo, Snap Points, Scrollable Content, Sides, Responsive Dialog, RTL).",
            "Each section includes a minimal code snippet that you can copy into an app and adapt.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows shadcn Drawer docs order with an extra snap-points recipe: Demo, Snap Points, Scrollable Content, Sides, Responsive Dialog, RTL.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description("Basic drawer with header copy and footer actions.")
                .test_id_prefix("ui-gallery-drawer")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example"),
            DocSection::new("Snap Points", snap_points)
                .description("Drag settles to the nearest snap point (Vaul-style).")
                .code_rust_from_file_region(snippets::snap_points::SOURCE, "example"),
            DocSection::new("Scrollable Content", scrollable_content)
                .description("Keep actions visible while the content area scrolls.")
                .code_rust_from_file_region(snippets::scrollable_content::SOURCE, "example"),
            DocSection::new("Sides", sides)
                .description("Use the `side` prop to control drawer placement.")
                .code_rust_from_file_region(snippets::sides::SOURCE, "example"),
            DocSection::new("Responsive Dialog", responsive_dialog).descriptions([
                "Responsive patterns often use Dialog on desktop and Drawer on mobile.",
                "Gallery renders both branches explicitly for deterministic testing (no viewport switches).",
            ])
            .code_rust_from_file_region(snippets::responsive_dialog::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Drawer layout should follow right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("Usage", usage)
                .title_test_id("ui-gallery-section-usage-title")
                .description("Quick reference for using the drawer recipes."),
            DocSection::new("Notes", notes)
                .title_test_id("ui-gallery-section-notes-title")
                .description("Implementation notes and regression guidelines."),
        ],
    );

    vec![body]
}
