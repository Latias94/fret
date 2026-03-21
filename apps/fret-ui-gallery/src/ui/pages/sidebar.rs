use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::sidebar as snippets;

pub(super) fn preview_sidebar(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let usage = snippets::usage::render(cx);
    let controlled = snippets::controlled::render(cx);
    let demo = snippets::demo::render(cx);
    let structure = snippets::structure::render(cx);
    let use_sidebar = snippets::use_sidebar::render(cx);
    let mobile = snippets::mobile::render(cx);
    let rtl = snippets::rtl::render(cx);

    let api_reference = doc_layout::notes_block([
        "`SidebarProvider` owns default open state, controlled models, viewport/mobile inference, and width defaults (`width`, `width_icon`, `width_mobile`).",
        "`Sidebar` owns `side`, `variant`, and `collapsible`; `SidebarInset` remains required for the inset variant.",
        "`SidebarHeader`, `SidebarFooter`, `SidebarContent`, `SidebarGroup`, `SidebarGroupLabel`, `SidebarGroupAction`, and `SidebarGroupContent` cover the upstream section structure directly.",
        "`SidebarMenu`, `SidebarMenuItem`, `SidebarMenuButton`, `SidebarMenuAction`, `SidebarMenuBadge`, `SidebarMenuSub`, `SidebarMenuSubItem`, `SidebarMenuSubButton`, and `SidebarRail` are already landed in the recipe layer.",
        "Focused composition seams already exist: `SidebarMenuButton::children(...)`, `SidebarGroupAction::children(...)`, `SidebarMenuAction::children(...)`, `Sidebar::into_element_with_children(...)`, and `SidebarMenuItem::into_element_with_children(...)`.",
        "Current conclusion: sidebar does not primarily need a broader generic children API; the bigger gap was that the gallery page did not teach the existing composition surface clearly enough.",
    ]);

    let notes = doc_layout::notes_block([
        "Width ownership follows upstream: use `SidebarProvider::width`, `width_icon`, and `width_mobile` first; `Sidebar` keeps theme-token fallback defaults.",
        "Keep `test_id_prefix` stable: `tools/diag-scripts/ui-gallery/sidebar/*` depend on DocSection tab trigger IDs.",
        "Mobile example forces `is_mobile(true)` for deterministic overlay + focus-restore diagnostics.",
        "The official docs split many sidebar parts into separate headings; the gallery keeps one consolidated `Structure` section so the copyable Fret authoring surface stays compact.",
        "Children/composition support is already present on the sidebar family. The page now makes that explicit instead of implying the recipe is incomplete.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-sidebar-api-reference")
        .description("Public surface summary, layer ownership, and composition-seam guidance.");
    let notes = DocSection::build(cx, "Notes", notes)
        .no_shell()
        .test_id_prefix("ui-gallery-sidebar-notes");
    let usage = DocSection::build(cx, "Usage", usage)
        .description("Minimal `SidebarProvider + Sidebar + SidebarInset` composition with provider-owned width defaults.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example")
        .no_shell();
    let controlled = DocSection::build(cx, "SidebarProvider", controlled)
        .description(
            "Controlled open state via `SidebarProvider`; width overrides also belong here.",
        )
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-controlled")
        .code_rust_from_file_region(snippets::controlled::SOURCE, "example")
        .no_shell();
    let demo = DocSection::build(cx, "Sidebar", demo)
        .description("Desktop sidebar shell with icon collapse, groups, content, and inset layout.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example")
        .no_shell();
    let structure = DocSection::build(cx, "Structure", structure)
        .description(
            "Copyable Fret consolidation of the upstream Header/Footer/Content/Group/Menu/Action/Sub/Rail sidebar parts.",
        )
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-structure")
        .code_rust_from_file_region(snippets::structure::SOURCE, "example")
        .no_shell();
    let use_sidebar = DocSection::build(cx, "useSidebar", use_sidebar)
        .description("Read provider state and resolved widths from `use_sidebar(cx)` inside the provider subtree.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-use-sidebar")
        .code_rust_from_file_region(snippets::use_sidebar::SOURCE, "example")
        .no_shell();
    let mobile = DocSection::build(cx, "Extras: Mobile", mobile)
        .description("Forced-mobile sheet path kept for deterministic overlay/focus diagnostics.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-mobile")
        .code_rust_from_file_region(snippets::mobile::SOURCE, "example")
        .no_shell();
    let rtl = DocSection::build(cx, "Extras: RTL", rtl)
        .description("RTL composition retained as a gallery extension for parity spot-checks.")
        .max_w(Px(980.0))
        .test_id_prefix("ui-gallery-sidebar-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example")
        .no_shell();

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview follows the shadcn Sidebar docs path first, then consolidates the many part-specific docs headings into one copyable Fret `Structure` section. Mobile remains a gallery-specific extra for deterministic diagnostics.",
        ),
        vec![
            usage,
            controlled,
            demo,
            structure,
            use_sidebar,
            mobile,
            rtl,
            api_reference,
            notes,
        ],
    );

    vec![body.test_id("ui-gallery-sidebar").into_element(cx)]
}
