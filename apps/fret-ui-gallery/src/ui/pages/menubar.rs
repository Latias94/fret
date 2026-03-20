use super::super::*;
use fret::UiCx;

use crate::ui::doc_layout::{self, DocSection};
use crate::ui::snippets::menubar as snippets;

pub(super) fn preview_menubar(cx: &mut UiCx<'_>) -> Vec<AnyElement> {
    let demo = snippets::demo::render(cx);
    let usage = snippets::usage::render(cx);
    let checkbox = snippets::checkbox::render(cx);
    let radio = snippets::radio::render(cx);
    let submenu = snippets::submenu::render(cx);
    let with_icons = snippets::with_icons::render(cx);
    let rtl = snippets::rtl::render(cx);
    let parts = snippets::parts::render(cx);

    let api_reference = doc_layout::notes_block([
        "Upstream docs path: `repo-ref/ui/apps/v4/content/docs/components/base/menubar.mdx`; example references: `repo-ref/ui/apps/v4/examples/base/menubar-*.tsx`; chrome reference: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/menubar.tsx`.",
        "`MenubarTrigger::new(...).into_menu().entries_parts(MenubarContent::new(), [...])` is the default copyable docs-aligned lane, while `Menubar::new([MenubarMenu::new(...).entries([...])])` remains the compact Fret-first shorthand for app code.",
        "`MenubarGroup::new([...])`, `MenubarSub::new(...)`, `MenubarSubContent::new([...])`, `MenubarSeparator`, and `MenubarShortcut::new(...)` keep the upstream part vocabulary visible while still landing into an explicit typed `MenubarEntry` tree.",
        "`MenubarCheckboxItem::from_checked(...).on_checked_change(...)` and `MenubarRadioGroup::from_value(...).on_value_change(...)` cover the docs examples without widening this family into per-row model plumbing.",
        "`MenubarContent::{min_width, submenu_min_width}` own explicit panel sizing overrides; root width remains caller-owned page/layout negotiation.",
        "A generic heterogeneous children API is still not warranted here: the typed `MenubarEntry` tree already preserves submenu structure, collection semantics, and diagnostics-friendly test ids without adding hidden scope contracts.",
    ]);
    let notes = doc_layout::notes_block([
        "Preview now mirrors the upstream shadcn/base Menubar docs path first: `Demo`, `Usage`, `Checkbox`, `Radio`, `Submenu`, `With Icons`, `RTL`, and `API Reference`.",
        "The docs-path snippets now stay on the part-shaped lane so the copyable code tabs teach the same `MenubarTrigger` / `MenubarContent` / `MenubarSub*` vocabulary that upstream docs do.",
        "The compact Fret-first root shorthand still exists, but it is documented in `API Reference` instead of displacing the docs-aligned authoring lane.",
        "`Parts` stays as the focused advanced adapter example after the docs path; it is not the primary teaching surface anymore.",
        "Keep `ui-gallery-menubar-*` and `ui-gallery-page-menubar` test IDs stable; existing diag scripts depend on them.",
    ]);
    let api_reference = DocSection::build(cx, "API Reference", api_reference)
        .no_shell()
        .test_id_prefix("ui-gallery-menubar-api-reference")
        .description("Public surface summary, ownership notes, and children API conclusion.");
    let notes = DocSection::build(cx, "Notes", notes).test_id_prefix("ui-gallery-menubar-notes");
    let demo = DocSection::build(cx, "Demo", demo)
        .description(
            "Full docs-aligned menubar demo with submenu, checkbox, radio, and shortcut rows.",
        )
        .test_id_prefix("ui-gallery-menubar-demo")
        .code_rust_from_file_region(snippets::demo::SOURCE, "example");
    let usage = DocSection::build(cx, "Usage", usage)
        .title_test_id("ui-gallery-section-usage-title")
        .description("Minimal part-shaped usage mirroring the upstream docs snippet.")
        .test_id_prefix("ui-gallery-menubar-usage")
        .code_rust_from_file_region(snippets::usage::SOURCE, "example");
    let checkbox = DocSection::build(cx, "Checkbox", checkbox)
        .description("Toggle-style menu rows using checkbox items and inset trailing shortcuts.")
        .test_id_prefix("ui-gallery-menubar-checkbox")
        .code_rust_from_file_region(snippets::checkbox::SOURCE, "example");
    let radio = DocSection::build(cx, "Radio", radio)
        .description(
            "Mutually exclusive choices via `MenubarRadioGroup` and `MenubarRadioItemSpec`.",
        )
        .test_id_prefix("ui-gallery-menubar-radio")
        .code_rust_from_file_region(snippets::radio::SOURCE, "example");
    let submenu = DocSection::build(cx, "Submenu", submenu)
        .description(
            "Nested menu content using `MenubarSub`, `MenubarSubTrigger`, and `MenubarSubContent`.",
        )
        .test_id_prefix("ui-gallery-menubar-submenu")
        .code_rust_from_file_region(snippets::submenu::SOURCE, "example");
    let with_icons = DocSection::build(cx, "With Icons", with_icons)
        .description("Leading-icon rows keep slot alignment while preserving shortcut and destructive variants.")
        .test_id_prefix("ui-gallery-menubar-with-icons")
        .code_rust_from_file_region(snippets::with_icons::SOURCE, "example");
    let rtl = DocSection::build(cx, "RTL", rtl)
        .description(
            "RTL layout mirrors the fuller docs demo shape instead of a reduced one-menu sample.",
        )
        .test_id_prefix("ui-gallery-menubar-rtl")
        .code_rust_from_file_region(snippets::rtl::SOURCE, "example");
    let parts = DocSection::build(cx, "Parts", parts)
        .description("Advanced Trigger/Content adapter surface kept outside the default docs path.")
        .test_id_prefix("ui-gallery-menubar-parts")
        .code_rust_from_file_region(snippets::parts::SOURCE, "example");

    let body = doc_layout::render_doc_page(
        cx,
        Some(
            "Preview mirrors the upstream shadcn/base Menubar docs path first, then keeps `Parts` and the surrounding notes as explicit Fret follow-ups.",
        ),
        vec![
            demo,
            usage,
            checkbox,
            radio,
            submenu,
            with_icons,
            rtl,
            api_reference,
            parts,
            notes,
        ],
    );

    let component = body
        .test_id("ui-gallery-menubar-component")
        .into_element(cx);
    let page = ui::v_flex(move |_cx| vec![component])
        .layout(LayoutRefinement::default().w_full().min_w_0())
        .items_start();
    vec![page.into_element(cx)]
}
