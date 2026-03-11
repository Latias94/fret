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

    let api_reference = doc_layout::notes(
        cx,
        [
            "`Drawer::direction(...)` is the upstream-aligned placement setter and accepts the documented `top`, `right`, `bottom`, and `left` directions.",
            "`Drawer::new_controllable(cx, None, false).compose()` is the preferred Fret equivalent of upstream nested children composition, with `DrawerTrigger::build(...)` covering `asChild`-style trigger ownership.",
            "`DrawerClose::from_scope().build(cx, child)` is the closest Fret equivalent to upstream `DrawerClose asChild` for caller-owned close buttons.",
            "`snap_points(...)` and `default_snap_point(...)` are Vaul-oriented extensions that stay outside the core shadcn docs path even though they are first-class Drawer policy in Fret.",
        ],
    );

    let notes = doc_layout::notes(
        cx,
        [
            "API reference: `ecosystem/fret-ui-shadcn/src/drawer.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/radix/drawer.mdx` and Vaul docs.",
            "Preview mirrors the shadcn Drawer docs path after the prose-only `About` and `Installation` sections: `Demo`, `Usage`, `Scrollable Content`, `Sides`, `Responsive Dialog`, `RTL`, and `API Reference`.",
            "`Snap Points` stays after `API Reference` as an explicit Vaul/Fret follow-up instead of being mixed into the docs path.",
            "`Drawer::compose()` is the recipe-level composable children bridge for shadcn-style part authoring without pushing children API concerns into the mechanism layer.",
            "`Demo` and `Responsive Dialog` keep the official inner content structure (centered max-width body, profile form layout) so gallery visuals stay close to shadcn docs instead of only proving the raw mechanism works.",
            "Responsive dialog recipe is represented as explicit desktop/mobile branches for deterministic gallery validation.",
            "Use stable test IDs on every scenario so diag scripts can capture open/close and layout outcomes reliably.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Drawer docs path after `About` and `Installation`, then keeps Vaul-specific `Snap Points` as a focused follow-up.",
        ),
        vec![
            DocSection::new("Demo", demo)
                .description(
                    "Official shadcn drawer demo with a centered max-width body, goal controls, and footer actions.",
                )
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
                    "Both branches preserve the official profile-form structure, while gallery renders them side by side for deterministic testing.",
                ])
                .code_rust_from_file_region(snippets::responsive_dialog::SOURCE, "example"),
            DocSection::new("RTL", rtl)
                .description("Drawer layout should follow right-to-left direction context.")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example"),
            DocSection::new("API Reference", api_reference)
                .no_shell()
                .test_id_prefix("ui-gallery-drawer-api-reference")
                .description("Public surface summary and Vaul-specific ownership notes."),
            DocSection::new("Snap Points", snap_points)
                .description("Drag settles to the nearest snap point (Vaul-style).")
                .code_rust_from_file_region(snippets::snap_points::SOURCE, "example"),
            DocSection::new("Notes", notes)
                .no_shell()
                .title_test_id("ui-gallery-section-notes-title")
                .description("Implementation notes and regression guidelines."),
        ],
    );

    vec![body]
}
