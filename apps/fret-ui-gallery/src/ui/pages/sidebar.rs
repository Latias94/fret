use super::super::*;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::sidebar as snippets;

pub(super) fn preview_sidebar(cx: &mut ElementContext<'_, App>) -> Vec<AnyElement> {
    let usage = snippets::usage::render(cx);
    let controlled = snippets::controlled::render(cx);
    let demo = snippets::demo::render(cx);
    let use_sidebar = snippets::use_sidebar::render(cx);
    let mobile = snippets::mobile::render(cx);
    let rtl = snippets::rtl::render(cx);

    let notes = doc_layout::notes(
        cx,
        [
            "Width ownership follows upstream: use `SidebarProvider::width`, `width_icon`, and `width_mobile` first; `Sidebar` keeps theme-token fallback defaults.",
            "Keep `test_id_prefix` stable: `tools/diag-scripts/ui-gallery/sidebar/*` depend on DocSection tab trigger IDs.",
            "Mobile example forces `is_mobile(true)` for deterministic overlay + focus-restore diagnostics.",
        ],
    );

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn Sidebar docs flow: Usage -> SidebarProvider -> Sidebar -> useSidebar. Mobile and RTL remain gallery extras.",
        ),
        vec![
            DocSection::new("Usage", usage)
                .description("Minimal `SidebarProvider + Sidebar + SidebarInset` composition with provider-owned width defaults.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-usage")
                .code_rust_from_file_region(snippets::usage::SOURCE, "example")
                .no_shell(),
            DocSection::new("SidebarProvider", controlled)
                .description("Controlled open state via `SidebarProvider`; width overrides also belong here.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-controlled")
                .code_rust_from_file_region(snippets::controlled::SOURCE, "example")
                .no_shell(),
            DocSection::new("Sidebar", demo)
                .description("Desktop sidebar shell with icon collapse, groups, content, and inset layout.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-demo")
                .code_rust_from_file_region(snippets::demo::SOURCE, "example")
                .no_shell(),
            DocSection::new("useSidebar", use_sidebar)
                .description("Read provider state and resolved widths from `use_sidebar(cx)` inside the provider subtree.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-use-sidebar")
                .code_rust_from_file_region(snippets::use_sidebar::SOURCE, "example")
                .no_shell(),
            DocSection::new("Extras: Mobile", mobile)
                .description("Forced-mobile sheet path kept for deterministic overlay/focus diagnostics.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-mobile")
                .code_rust_from_file_region(snippets::mobile::SOURCE, "example")
                .no_shell(),
            DocSection::new("Extras: RTL", rtl)
                .description("RTL composition retained as a gallery extension for parity spot-checks.")
                .max_w(Px(980.0))
                .test_id_prefix("ui-gallery-sidebar-rtl")
                .code_rust_from_file_region(snippets::rtl::SOURCE, "example")
                .no_shell(),
            DocSection::new("Notes", notes)
                .no_shell()
                .test_id_prefix("ui-gallery-sidebar-notes"),
        ],
    );

    vec![body.test_id("ui-gallery-sidebar")]
}
