mod support;

use support::{gallery_rust_sources, manifest_path, read, read_path};

fn assert_curated_facade_only(relative_paths: &[&str]) {
    for relative_path in relative_paths {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        assert!(
            source.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"),
            "{} should import the curated shadcn facade",
            path.display()
        );

        for line in source.lines() {
            if !line.contains("fret_ui_shadcn::") {
                continue;
            }

            assert_eq!(
                line.trim(),
                "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
                "{} reintroduced a direct fret_ui_shadcn root path: {}",
                path.display(),
                line.trim()
            );
        }
    }
}

fn assert_curated_facade_root_only(relative_paths: &[&str]) {
    for relative_path in relative_paths {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        assert!(
            source.contains("use fret_ui_shadcn::facade as shadcn;"),
            "{} should import the curated shadcn facade root lane",
            path.display()
        );

        for line in source.lines() {
            if !line.contains("fret_ui_shadcn::") {
                continue;
            }

            assert_eq!(
                line.trim(),
                "use fret_ui_shadcn::facade as shadcn;",
                "{} reintroduced a direct fret_ui_shadcn root path: {}",
                path.display(),
                line.trim()
            );
        }
    }
}

fn assert_only_documented_raw_shadcn_modules(path: &std::path::Path, source: &str) {
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.contains("shadcn::raw::") {
            continue;
        }

        let allowed = trimmed.contains("shadcn::raw::typography::")
            || trimmed.contains("shadcn::raw::accordion::")
            || trimmed.contains("shadcn::raw::extras::")
            || trimmed.contains("shadcn::raw::breadcrumb::")
            || trimmed.contains("shadcn::raw::collapsible::")
            || trimmed.contains("shadcn::raw::experimental::")
            || trimmed.contains("shadcn::raw::icon::")
            || trimmed.contains("shadcn::raw::button::")
            || trimmed.contains("shadcn::raw::calendar::")
            || trimmed.contains("shadcn::raw::context_menu::")
            || trimmed.contains("shadcn::raw::dropdown_menu::")
            || trimmed.contains("shadcn::raw::kbd::")
            || trimmed.contains("shadcn::raw::menubar::")
            || trimmed.contains("shadcn::raw::select::")
            || trimmed.contains("shadcn::raw::switch::")
            || trimmed.contains("shadcn::raw::tabs::")
            || trimmed.contains("shadcn::raw::toggle_group::");
        assert!(
            allowed,
            "{}:{} used an undocumented shadcn raw escape hatch: {}",
            path.display(),
            line_idx + 1,
            trimmed
        );
    }
}

#[test]
fn gallery_curated_shadcn_surfaces_stay_explicit() {
    let chrome = read("src/driver/chrome.rs");
    let runtime_driver = read("src/driver/runtime_driver.rs");
    let ui_mod = read("src/ui/mod.rs");
    let settings_sheet = read("src/driver/settings_sheet.rs");
    let theme_runtime = read("src/driver/theme_runtime.rs");

    for source in [&chrome, &runtime_driver, &ui_mod] {
        assert!(!source.contains("use fret_ui_shadcn as shadcn;"));
        assert!(!source.contains("use fret_ui_shadcn::{self as shadcn"));
    }

    assert!(chrome.contains("use fret_ui_shadcn::facade as shadcn;"));
    assert!(runtime_driver.contains("use fret_ui_shadcn::facade as shadcn;"));
    assert!(ui_mod.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));
    assert!(settings_sheet.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"));

    assert!(!theme_runtime.contains("fret_ui_shadcn::shadcn_themes::"));
    assert!(theme_runtime.contains("shadcn::themes::ShadcnBaseColor::"));
    assert!(theme_runtime.contains("shadcn::themes::apply_shadcn_new_york"));
}

#[test]
fn gallery_source_tree_rejects_legacy_shadcn_alias_patterns() {
    for path in gallery_rust_sources() {
        if path.ends_with("src/lib.rs") {
            continue;
        }

        let source = read_path(&path);
        assert!(
            !source.contains("fret_ui_fret_ui_shadcn::"),
            "{} duplicated an explicit fret_ui_shadcn path prefix",
            path.display()
        );
        assert!(
            !source.contains("use fret_ui_shadcn as shadcn;"),
            "{} reintroduced the legacy root alias import",
            path.display()
        );
        assert!(
            !source.contains("use fret_ui_shadcn::{self as shadcn"),
            "{} reintroduced the legacy self-as-shadcn import",
            path.display()
        );

        for (line_idx, line) in source.lines().enumerate() {
            for (offset, _) in line.match_indices("shadcn::") {
                let previous = line[..offset].chars().next_back();
                if previous.is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_') {
                    continue;
                }

                let after = &line[offset + "shadcn::".len()..];
                let segment_len = after
                    .chars()
                    .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
                    .count();
                if segment_len == 0 {
                    continue;
                }

                let segment = &after[..segment_len];
                let is_module_path = after[segment_len..].starts_with("::");
                let starts_with_lowercase = segment
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_lowercase());
                if is_module_path
                    && starts_with_lowercase
                    && !matches!(segment, "raw" | "themes" | "app")
                {
                    panic!(
                        "{}:{} reintroduced non-curated shadcn module path `shadcn::{}::`",
                        path.display(),
                        line_idx + 1,
                        segment
                    );
                }
            }
        }
    }
}

#[test]
fn gallery_source_tree_avoids_root_shadcn_glue_paths() {
    for path in gallery_rust_sources() {
        let source = read_path(&path);
        assert!(
            !source.contains("fret_ui_shadcn::decl_style"),
            "{} reintroduced the root `decl_style` glue lane",
            path.display()
        );
        assert!(
            !source.contains("fret_ui_shadcn::icon::"),
            "{} reintroduced the root `icon` glue lane",
            path.display()
        );
    }
}

#[test]
fn gallery_source_tree_avoids_legacy_conversion_trait_names() {
    for path in gallery_rust_sources() {
        let source = read_path(&path);
        for legacy_name in [
            "UiIntoElement",
            "UiChildIntoElement",
            "UiHostBoundIntoElement",
            "UiBuilderHostBoundIntoElementExt",
        ] {
            assert!(
                !source.contains(legacy_name),
                "{} reintroduced legacy conversion name `{}` into the first-party gallery surface",
                path.display(),
                legacy_name
            );
        }
    }
}

#[test]
fn gallery_ai_snippet_batch_prefers_curated_shadcn_facade_imports() {
    assert_curated_facade_only(&[
        "src/ui/snippets/ai/audio_player_demo.rs",
        "src/ui/snippets/ai/canvas_world_layer_spike.rs",
        "src/ui/snippets/ai/checkpoint_demo.rs",
        "src/ui/snippets/ai/commit_custom_children.rs",
        "src/ui/snippets/ai/confirmation_demo.rs",
        "src/ui/snippets/ai/confirmation_request.rs",
        "src/ui/snippets/ai/model_selector_demo.rs",
        "src/ui/snippets/ai/persona_demo.rs",
        "src/ui/snippets/ai/persona_state_management.rs",
        "src/ui/snippets/ai/plan_demo.rs",
        "src/ui/snippets/ai/prompt_input_docs_demo.rs",
        "src/ui/snippets/ai/prompt_input_referenced_sources_demo.rs",
        "src/ui/snippets/ai/reasoning_demo.rs",
        "src/ui/snippets/ai/speech_input_demo.rs",
        "src/ui/snippets/ai/task_demo.rs",
        "src/ui/snippets/ai/transcript_torture.rs",
        "src/ui/snippets/ai/workflow_canvas_demo.rs",
        "src/ui/snippets/ai/workflow_toolbar_demo.rs",
    ]);
}

#[test]
fn gallery_ai_snippet_tree_avoids_direct_shadcn_root_paths() {
    for path in support::rust_sources("src/ui/snippets/ai") {
        let source = read_path(&path);
        for (line_idx, line) in source.lines().enumerate() {
            if !line.contains("fret_ui_shadcn::") {
                continue;
            }

            let trimmed = line.trim();
            let allowed_import = trimmed == "use fret_ui_shadcn::prelude::*;"
                || trimmed == "use fret_ui_shadcn::{facade as shadcn, prelude::*};"
                || (trimmed.starts_with("use fret_ui_shadcn::prelude::{")
                    && trimmed.ends_with("};"));
            assert!(
                allowed_import,
                "{}:{} reintroduced a direct fret_ui_shadcn root path outside allowed imports: {}",
                path.display(),
                line_idx + 1,
                trimmed
            );
        }
    }
}

#[test]
fn gallery_shadcn_extras_batch_uses_explicit_raw_escape_hatch() {
    let relative_paths = [
        "src/ui/snippets/shadcn_extras/announcement.rs",
        "src/ui/snippets/shadcn_extras/avatar_stack.rs",
        "src/ui/snippets/shadcn_extras/banner.rs",
        "src/ui/snippets/shadcn_extras/kanban.rs",
        "src/ui/snippets/shadcn_extras/marquee.rs",
        "src/ui/snippets/shadcn_extras/rating.rs",
        "src/ui/snippets/shadcn_extras/relative_time.rs",
        "src/ui/snippets/shadcn_extras/tags.rs",
        "src/ui/snippets/shadcn_extras/ticker.rs",
    ];

    assert_curated_facade_only(&relative_paths);

    for relative_path in relative_paths {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        assert!(
            source.contains("shadcn::raw::extras::"),
            "{} should use the explicit raw extras escape hatch",
            path.display()
        );
    }
}

#[test]
fn gallery_breadcrumb_primitive_batch_uses_explicit_raw_escape_hatch() {
    let relative_paths = [
        "src/ui/snippets/breadcrumb/demo.rs",
        "src/ui/snippets/breadcrumb/dropdown.rs",
        "src/ui/snippets/breadcrumb/link_component.rs",
        "src/ui/snippets/breadcrumb/responsive.rs",
        "src/ui/snippets/breadcrumb/rtl.rs",
    ];

    for relative_path in relative_paths {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        assert!(
            source.contains("use fret_ui_shadcn::facade as shadcn;")
                || source.contains("use fret_ui_shadcn::{facade as shadcn, prelude::*};"),
            "{} should import the curated shadcn facade before reopening raw breadcrumb primitives",
            path.display()
        );

        for line in source.lines() {
            if !line.contains("fret_ui_shadcn::") {
                continue;
            }

            let trimmed = line.trim();
            assert!(
                trimmed == "use fret_ui_shadcn::facade as shadcn;"
                    || trimmed == "use fret_ui_shadcn::{facade as shadcn, prelude::*};",
                "{} reintroduced a direct fret_ui_shadcn root path: {}",
                path.display(),
                trimmed
            );
        }
        assert!(
            source.contains("use shadcn::raw::breadcrumb::primitives as bc;"),
            "{} should use the explicit raw breadcrumb primitive escape hatch",
            path.display()
        );
    }
}

#[test]
fn gallery_breadcrumb_docs_examples_prefer_curated_parts_aliases() {
    let relative_paths = [
        "src/ui/snippets/breadcrumb/basic.rs",
        "src/ui/snippets/breadcrumb/collapsed.rs",
        "src/ui/snippets/breadcrumb/custom_separator.rs",
    ];
    assert_curated_facade_only(&relative_paths);

    for relative_path in relative_paths {
        let path = manifest_path(relative_path);
        let source = read_path(&path);
        assert!(
            source.contains("shadcn::BreadcrumbRoot::new()"),
            "{} should keep the docs-path root on the curated breadcrumb facade lane",
            path.display()
        );
        assert!(
            source.contains("shadcn::BreadcrumbList::new()"),
            "{} should keep the docs-path list on the curated breadcrumb facade lane",
            path.display()
        );
        assert!(
            source.contains("shadcn::BreadcrumbItemPart::new()"),
            "{} should keep the docs-path item composition on the curated breadcrumb facade lane",
            path.display()
        );
        assert!(
            source.contains("shadcn::BreadcrumbSeparatorPart::new()"),
            "{} should keep the docs-path separator on the curated breadcrumb facade lane",
            path.display()
        );
        assert!(
            !source.contains("shadcn::raw::breadcrumb::primitives"),
            "{} should not reopen the raw breadcrumb primitive escape hatch for docs-path examples",
            path.display()
        );
    }
}

#[test]
fn gallery_breadcrumb_usage_snippet_prefers_curated_parts_aliases() {
    let relative_path = "src/ui/snippets/breadcrumb/usage.rs";
    assert_curated_facade_only(&[relative_path]);

    let path = manifest_path(relative_path);
    let source = read_path(&path);
    assert!(
        source.contains("shadcn::BreadcrumbRoot::new()"),
        "{} should teach the curated breadcrumb root alias",
        path.display()
    );
    assert!(
        source.contains("shadcn::BreadcrumbList::new()"),
        "{} should keep the breadcrumb list on the curated facade lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::BreadcrumbItemPart::new()"),
        "{} should teach the curated breadcrumb item alias",
        path.display()
    );
    assert!(
        source.contains("shadcn::BreadcrumbSeparatorPart::new()"),
        "{} should teach the curated breadcrumb separator alias",
        path.display()
    );
    assert!(
        source.contains("shadcn::BreadcrumbLink::new(\"Home\")"),
        "{} should keep link composition on the curated facade lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::BreadcrumbPage::new(\"Breadcrumb\")"),
        "{} should keep current-page composition on the curated facade lane",
        path.display()
    );
    assert!(
        !source.contains("shadcn::raw::breadcrumb::primitives"),
        "{} should not require the raw breadcrumb primitive escape hatch for the copyable usage lane",
        path.display()
    );
}

#[test]
fn gallery_collapsible_usage_snippet_prefers_curated_parts_aliases() {
    let relative_path = "src/ui/snippets/collapsible/usage.rs";
    assert_curated_facade_only(&[relative_path]);

    let path = manifest_path(relative_path);
    let source = read_path(&path);
    assert!(
        source.contains("shadcn::CollapsibleRoot::new()"),
        "{} should teach the curated collapsible root alias",
        path.display()
    );
    assert!(
        source.contains("shadcn::CollapsibleTriggerPart::new("),
        "{} should teach the curated collapsible trigger alias",
        path.display()
    );
    assert!(
        source.contains("shadcn::CollapsibleContentPart::new("),
        "{} should teach the curated collapsible content alias",
        path.display()
    );
    assert!(
        !source.contains("shadcn::raw::collapsible::primitives::{"),
        "{} should not require the raw collapsible escape hatch for the copyable usage lane",
        path.display()
    );
}

#[test]
fn gallery_accordion_usage_snippet_prefers_curated_parts_aliases() {
    let relative_path = "src/ui/snippets/accordion/usage.rs";
    assert_curated_facade_only(&[relative_path]);

    let path = manifest_path(relative_path);
    let source = read_path(&path);
    assert!(
        source.contains("shadcn::AccordionRoot::single_uncontrolled("),
        "{} should teach the curated accordion root alias",
        path.display()
    );
    assert!(
        source.contains("shadcn::AccordionItemPart::new("),
        "{} should teach the curated accordion item alias",
        path.display()
    );
    assert!(
        source.contains("shadcn::AccordionTriggerPart::new("),
        "{} should teach the curated accordion trigger alias",
        path.display()
    );
    assert!(
        source.contains("shadcn::AccordionContentPart::new("),
        "{} should teach the curated accordion content alias",
        path.display()
    );
    assert!(
        !source.contains("shadcn::raw::accordion::composable"),
        "{} should not require the raw accordion composable escape hatch for the copyable usage lane",
        path.display()
    );
}

#[test]
fn gallery_alert_dialog_usage_snippet_prefers_curated_part_lane() {
    let relative_path = "src/ui/snippets/alert_dialog/usage.rs";
    assert_curated_facade_root_only(&[relative_path]);

    let path = manifest_path(relative_path);
    let source = read_path(&path);
    assert!(
        source.contains("shadcn::AlertDialog::new_controllable("),
        "{} should keep the alert-dialog root on the curated facade lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::AlertDialogPart::trigger("),
        "{} should teach the curated alert-dialog trigger part lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::AlertDialogPart::content_with("),
        "{} should teach the deferred alert-dialog content part lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::AlertDialogContent::new([]).with_children"),
        "{} should keep content composition on the copyable alert-dialog lane",
        path.display()
    );
    assert!(
        !source.contains("shadcn::raw::alert_dialog::"),
        "{} should not require the raw alert-dialog escape hatch for the copyable usage lane",
        path.display()
    );
}

#[test]
fn gallery_dialog_usage_snippet_prefers_curated_part_lane() {
    let relative_path = "src/ui/snippets/dialog/usage.rs";
    assert_curated_facade_root_only(&[relative_path]);

    let path = manifest_path(relative_path);
    let source = read_path(&path);
    assert!(
        source.contains("shadcn::Dialog::new_controllable("),
        "{} should keep the dialog root on the curated facade lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::DialogPart::trigger("),
        "{} should teach the curated dialog trigger part lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::DialogPart::content_with("),
        "{} should teach the deferred curated dialog content part lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::DialogContent::new([]).with_children("),
        "{} should keep content composition on the copyable dialog with_children lane",
        path.display()
    );
    assert!(
        !source.contains("shadcn::raw::dialog::"),
        "{} should not require the raw dialog escape hatch for the copyable usage lane",
        path.display()
    );
}

#[test]
fn gallery_drawer_usage_snippet_prefers_curated_part_lane() {
    let relative_path = "src/ui/snippets/drawer/usage.rs";
    assert_curated_facade_root_only(&[relative_path]);

    let path = manifest_path(relative_path);
    let source = read_path(&path);
    assert!(
        source.contains("shadcn::Drawer::new_controllable("),
        "{} should keep the drawer root on the curated facade lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::DrawerPart::trigger("),
        "{} should teach the curated drawer trigger part lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::DrawerPart::content_with("),
        "{} should teach the deferred drawer content part lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::DrawerContent::new([])") && source.contains(".children(|cx| {"),
        "{} should keep content composition on the copyable drawer children() lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::DrawerClose::from_scope().child("),
        "{} should keep close actions on the copyable drawer lane",
        path.display()
    );
    assert!(
        !source.contains("shadcn::raw::drawer::"),
        "{} should not require the raw drawer escape hatch for the copyable usage lane",
        path.display()
    );
}

#[test]
fn gallery_sheet_usage_snippet_prefers_curated_part_lane() {
    let relative_path = "src/ui/snippets/sheet/usage.rs";
    assert_curated_facade_root_only(&[relative_path]);

    let path = manifest_path(relative_path);
    let source = read_path(&path);
    assert!(
        source.contains("shadcn::Sheet::new_controllable("),
        "{} should keep the sheet root on the curated facade lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::SheetPart::trigger("),
        "{} should teach the curated sheet trigger part lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::SheetPart::content_with("),
        "{} should teach the curated sheet content part lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::SheetPart::content_with("),
        "{} should keep content composition on the copyable sheet lane",
        path.display()
    );
    assert!(
        source.contains("shadcn::SheetContent::new([]).with_children("),
        "{} should keep nested sheet sections on the composable with_children() lane",
        path.display()
    );
    assert!(
        !source.contains("shadcn::raw::sheet::"),
        "{} should not require the raw sheet escape hatch for the copyable usage lane",
        path.display()
    );
}

#[test]
fn gallery_source_tree_limits_raw_shadcn_escape_hatches() {
    for path in gallery_rust_sources() {
        let source = read_path(&path);
        assert_only_documented_raw_shadcn_modules(&path, &source);
    }
}
