use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::drawer as snippets;

pub(super) fn preview_drawer(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let snap_points = snippets::snap_points::render(cx);
    let nested = snippets::nested::render(cx);
    let outside_press = snippets::outside_press::render(cx);
    let scrollable_content = snippets::scrollable_content::render(cx);
    let sides = snippets::sides::render(cx);
    let responsive_dialog = snippets::responsive_dialog::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`Drawer::direction(...)` is the upstream-aligned placement setter and accepts the documented `top`, `right`, `bottom`, and `left` directions.",
        "`Drawer::new_controllable(cx, None, false).children([...])` is the closest Fret equivalent of upstream nested children composition, with `DrawerPart::trigger(...)` and `DrawerPart::content_with(...)` keeping the default path copyable on the curated facade.",
        "`DrawerContent::new([]).with_children(cx, ...)` plus `DrawerHeader::new([]).with_children(cx, ...)` / `DrawerFooter::new([]).with_children(cx, ...)` is the default copyable content lane for upstream-like nested composition.",
        "`DrawerClose::from_scope().build(cx, child)` is the closest Fret equivalent to upstream `DrawerClose asChild` for caller-owned close buttons.",
        "`Drawer::disable_pointer_dismissal(...)` is the Base UI-style alias for turning off outside-press dismissal while keeping the same modal drawer mechanism.",
        "`Drawer::modal(false)` and `Drawer::modal_mode(DrawerModalMode::TrapFocus)` are Base UI-oriented follow-ups for non-modal and trap-focus drawer policy without widening the mechanism layer.",
        "`Drawer::compose()` remains the builder-first follow-up when callers prefer explicit trigger/content chaining over part collection.",
        "`DrawerContent::build(...)` remains the builder-first companion when a snippet genuinely wants `push_ui(...)` assembly instead of composable nested sections.",
        "`Usage` is the default copyable `children([...])` path, while `Snap Points` stays a Vaul/Fret policy follow-up rather than a separate root-authoring lane.",
        "`snap_points(...)` and `default_snap_point(...)` are Vaul-oriented extensions that stay outside the core shadcn docs path even though they are first-class Drawer policy in Fret.",
        "`snap_point(...)`, `on_snap_point_change(...)`, and `snap_to_sequential_points(...)` are recipe-owned controlled snap-point follow-ups on that same Drawer root lane.",
        "`Nested Drawers` is a Base UI/Fret follow-up that keeps the authored `children([...])` lane while using `Drawer::modal(false)` on the child to preserve child-first drag routing inside a modal parent.",
    ]);

    let notes = doc_layout::notes_block([
        "API reference: `ecosystem/fret-ui-shadcn/src/drawer.rs`. Upstream references: `repo-ref/ui/apps/v4/content/docs/components/base/drawer.mdx` and Vaul docs.",
        "Preview mirrors the shadcn Drawer docs path after the prose-only `About` and `Installation` sections: `Demo`, `Usage`, `Scrollable Content`, `Sides`, `Responsive Dialog`, `RTL`, and `API Reference`.",
        "`Usage` is the default copyable `children([...])` path, while `Snap Points` stays a Vaul/Fret policy follow-up rather than a separate root-authoring lane.",
        "`Usage` is the default copyable path; `Snap Points`, `Nested Drawers`, and `Outside Press` stay after `API Reference` as explicit Vaul/Fret follow-ups instead of being mixed into the docs path.",
        "The docs-path examples now share the same `Drawer::children([...])` root lane plus `DrawerContent::new([]).with_children(cx, ...)` content lane, while `compose()` and `DrawerContent::build(...)` remain builder-first alternatives without pushing children API concerns into the mechanism layer.",
        "Docs-path footer close actions now consistently use `DrawerClose::from_scope().build(cx, child)` so the copyable lane stays aligned with upstream `DrawerClose asChild` intent.",
        "Base UI-only policy variants such as `modal={false|'trap-focus'}` now exist as follow-up API, but they are intentionally not taught on this page because the shadcn docs path stays modal-first.",
        "Controlled snap points now exist as an authored-index follow-up surface, and nested non-modal child drawers now route drag input above the parent barrier while still suppressing parent drag and tracking frontmost child height.",
        "`Outside Press` is a gallery-only follow-up probe surface that makes modal dismissal and focus-restore evidence deterministic without widening the core recipe API.",
        "Modal-on-modal nested swipe choreography and background indentation remain wider follow-up work than the shadcn docs path.",
        "`Demo`, `Responsive Dialog`, and `RTL` keep the official inner content structure (centered max-width body, profile form layout, goal-adjust controls) so gallery visuals stay close to shadcn docs instead of only proving the raw mechanism works.",
        "Responsive dialog recipe is represented as explicit desktop/mobile branches for deterministic gallery validation.",
        "Use stable test IDs on every scenario so diag scripts can capture open/close and layout outcomes reliably.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-drawer-api-reference")
        .description("Public surface summary and Vaul-specific ownership notes.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .title_test_id("ui-gallery-section-notes-title")
        .description("Implementation notes and regression guidelines.");
    let demo = DocSection::build(cx, "Demo", demo)
        .description(
            "Official shadcn drawer demo with a centered max-width body, goal controls, and footer actions.",
        )
        .test_id_prefix("ui-gallery-drawer")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Default copyable `children([...])` root lane with composable `with_children(...)` content sections.")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let scrollable_content = DocSection::build(cx, "Scrollable Content", scrollable_content)
        .description("Keep actions visible while the content area scrolls.")
        .code_rust_from_file_region(snippets::scrollable_content::SOURCE, "example");
    let sides = DocSection::build(cx, "Sides", sides)
        .description(
            "Use the `direction` prop to set the side of the drawer. Available options are `top`, `right`, `bottom`, and `left`.",
        )
        .code_rust_from_file_region(snippets::sides::SOURCE, "example");
    let responsive_dialog = DocSection::build(cx, "Responsive Dialog", responsive_dialog)
        .descriptions([
            "You can combine the `Dialog` and `Drawer` components to create a responsive dialog.",
            "Both branches preserve the official profile-form structure, while gallery renders them side by side for deterministic testing.",
        ])
        .code_rust_from_file_region(snippets::responsive_dialog::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description("Drawer layout should follow right-to-left direction context.")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let snap_points = DocSection::build(cx, "Snap Points", snap_points)
        .description("Vaul/Fret policy follow-up built on the same Drawer root while drag settles to the nearest snap point.")
        .code_rust_from_file_region(snippets::snap_points::SOURCE, "example");
    let nested = DocSection::build(cx, "Nested Drawers", nested)
        .description("Base UI/Fret follow-up: parent modal drawer plus child `modal(false)` drawer, with child-first drag routing kept intact.")
        .code_rust_from_file_region(snippets::nested::SOURCE, "example");
    let outside_press = DocSection::build(cx, "Outside Press", outside_press)
        .description("Gallery-only follow-up: modal outside press closes the drawer and restores focus to the trigger while the underlay probe stays inert.")
        .code_rust_from_file_region(snippets::outside_press::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the shadcn Drawer docs path after `About` and `Installation`, then keeps Vaul-specific `Snap Points` plus Base UI/Fret-oriented `Nested Drawers` and `Outside Press` as focused follow-ups.",
        ),
        vec![
            demo,
            usage,
            scrollable_content,
            sides,
            responsive_dialog,
            rtl,
            api_reference,
            snap_points,
            nested,
            outside_press,
            notes,
        ],
    );

    vec![body.into_element(cx)]
}
