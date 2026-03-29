fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn menubar_page_documents_docs_path_and_children_api_decision() {
    let source = include_str!("../src/ui/pages/menubar.rs");

    for needle in [
        "`MenubarTrigger::new(...).into_menu().entries_parts(MenubarContent::new(), [...])` is the default copyable docs-aligned lane, while `Menubar::new([MenubarMenu::new(...).entries([...])])` remains the compact Fret-first shorthand for app code.",
        "A generic heterogeneous children API is still not warranted here: the typed `MenubarEntry` tree already preserves submenu structure, collection semantics, and diagnostics-friendly test ids without adding hidden scope contracts.",
        "Preview now mirrors the upstream shadcn/base Menubar docs path first: `Demo`, `Usage`, `Checkbox`, `Radio`, `Submenu`, `With Icons`, `RTL`, and `API Reference`.",
        "The compact Fret-first root shorthand still exists, but it is documented in `API Reference` instead of displacing the docs-aligned authoring lane.",
        "DocSection::build(cx, \"API Reference\", api_reference)",
        "DocSection::build(cx, \"Parts\", parts)",
    ] {
        assert!(
            source.contains(needle),
            "menubar page should document the docs-path order and children API decision; missing `{needle}`"
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
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
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "menubar page should keep `Parts` and `Notes` after the docs-path sections and `API Reference`"
    );
}

#[test]
fn menubar_usage_snippet_stays_full_and_copyable() {
    let source = include_str!("../src/ui/snippets/menubar/usage.rs");

    for needle in [
        "use fret::{UiChild, UiCx};",
        "use fret_runtime::CommandId;",
        "use fret_ui_shadcn::facade as shadcn;",
        "pub fn render(cx: &mut UiCx<'_>) -> impl UiChild + use<>",
        "shadcn::MenubarTrigger::new(\"File\")",
        ".into_menu()",
        ".entries_parts(",
        "shadcn::MenubarGroup::new([",
        "shadcn::MenubarShortcut::new(\"⌘T\").into_element(cx)",
        "shadcn::Menubar::new([file]).into_element(cx)",
    ] {
        assert!(
            source.contains(needle),
            "menubar usage snippet should remain a complete copyable docs-path example; missing `{needle}`"
        );
    }
}

#[test]
fn menubar_demo_and_rtl_profiles_keep_upstream_separator_split() {
    let demo = normalize_ws(include_str!("../src/ui/snippets/menubar/demo.rs"));
    let rtl = normalize_ws(include_str!("../src/ui/snippets/menubar/rtl.rs"));

    let demo_split = normalize_ws(
        r#"
        shadcn::MenubarSeparator::new().into(),
        shadcn::MenubarGroup::new([shadcn::MenubarItem::new("Edit...").inset(true).into()])
            .into(),
        shadcn::MenubarSeparator::new().into(),
        shadcn::MenubarGroup::new([shadcn::MenubarItem::new("Add Profile...")
            .inset(true)
            .into()])
            .into(),
        "#,
    );
    assert!(
        demo.contains(&demo_split),
        "menubar demo should keep `Edit...` and `Add Profile...` split by their own separator to match the upstream docs/demo grouping"
    );

    let demo_combined = normalize_ws(
        r#"
        shadcn::MenubarGroup::new([
            shadcn::MenubarItem::new("Edit...").inset(true).into(),
            shadcn::MenubarItem::new("Add Profile...")
                .inset(true)
                .into(),
        ])
        .into(),
        "#,
    );
    assert!(
        !demo.contains(&demo_combined),
        "menubar demo should not collapse `Edit...` and `Add Profile...` into one group"
    );

    let rtl_split = normalize_ws(
        r#"
        shadcn::MenubarSeparator::new().into(),
        shadcn::MenubarGroup::new([shadcn::MenubarItem::new("تعديل...")
            .inset(true)
            .into()])
            .into(),
        shadcn::MenubarSeparator::new().into(),
        shadcn::MenubarGroup::new([shadcn::MenubarItem::new("إضافة ملف شخصي...")
            .inset(true)
            .into()])
            .into(),
        "#,
    );
    assert!(
        rtl.contains(&rtl_split),
        "RTL menubar demo should keep the same split group structure as the upstream docs/demo"
    );

    let rtl_combined = normalize_ws(
        r#"
        shadcn::MenubarGroup::new([
            shadcn::MenubarItem::new("تعديل...").inset(true).into(),
            shadcn::MenubarItem::new("إضافة ملف شخصي...")
                .inset(true)
                .into(),
        ])
        .into(),
        "#,
    );
    assert!(
        !rtl.contains(&rtl_combined),
        "RTL menubar demo should not collapse the trailing profile actions into one group"
    );
}
