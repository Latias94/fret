fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}

#[test]
fn sidebar_page_documents_group_label_children_lane_and_page_order() {
    let source = include_str!("../src/ui/pages/sidebar.rs");

    for needle in [
        "`SidebarGroupLabel::children(...).as_child(true)`",
        "sidebar still does not primarily need a broader generic root-children API",
        "the gallery keeps one consolidated `Structure` section",
        "`SidebarGroupLabel asChild + CollapsibleTrigger` lane",
        "The structure snippet now carries the official `SidebarGroupLabel asChild` collapsible-group lane",
        "`SidebarProvider::device_shell_mode(...)` and `device_shell_switch_policy(...)` are app-shell/device-shell controls",
        "`Sidebar` should stay an app-shell surface; editor rails and inspector sidebars should use a separate container-aware surface",
        "That forced-mobile example documents the app-shell sheet path only",
    ] {
        assert!(
            source.contains(needle),
            "sidebar page should document the focused children decision and docs-path follow-up; missing `{needle}`"
        );
    }

    let normalized = normalize_ws(source);
    let ordered_sections = normalize_ws(
        r#"
        vec![
            usage,
            controlled,
            demo,
            structure,
            app_sidebar,
            use_sidebar,
            mobile,
            rtl,
            api_reference,
            notes,
        ]
        "#,
    );
    assert!(
        normalized.contains(&ordered_sections),
        "sidebar page should keep the current docs-first order before notes"
    );
}

#[test]
fn sidebar_structure_snippet_keeps_group_label_as_child_copyable() {
    let source = include_str!("../src/ui/snippets/sidebar/structure.rs");

    for needle in [
        "let help_group_open = cx.local_model_keyed(\"help_group_open\", || true);",
        "shadcn::SidebarGroupLabel::new(\"Help\")",
        ".as_child(true)",
        "shadcn::CollapsibleTriggerPart::new([label_row])",
        "shadcn::CollapsibleContentPart::new([",
        "\"ui-gallery-sidebar-structure-group-label-row\"",
        "\"ui-gallery-sidebar-structure-group-label-content\"",
    ] {
        assert!(
            source.contains(needle),
            "sidebar structure snippet should keep the group-label-as-child docs lane copyable; missing `{needle}`"
        );
    }
}
