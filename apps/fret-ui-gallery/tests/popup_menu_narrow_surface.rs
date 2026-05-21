#[test]
fn context_menu_demo_snippet_uses_a_unique_overlay_panel_test_id() {
    let demo = include_str!("../src/ui/snippets/context_menu/demo.rs");

    assert!(
        demo.contains("\"ui-gallery-context-menu-demo-panel\""),
        "context-menu demo snippet should expose a unique overlay panel test id for diag scripts",
    );
    assert!(
        !demo.contains("\"ui-gallery-context-menu-demo-content\""),
        "context-menu demo snippet should not reuse the DocSection content test id for the open menu panel",
    );
}

#[test]
fn context_menu_basic_snippet_uses_a_unique_overlay_panel_test_id() {
    let basic = include_str!("../src/ui/snippets/context_menu/basic.rs");

    assert!(
        basic.contains("\"ui-gallery-context-menu-basic-panel\""),
        "context-menu basic snippet should expose a unique overlay panel test id for diag scripts",
    );
    assert!(
        !basic.contains("\"ui-gallery-context-menu-basic-content\""),
        "context-menu basic snippet should not reuse the DocSection content test id for the open menu panel",
    );
}

#[test]
fn popup_menu_narrow_sweep_covers_select_combobox_context_menu_and_dropdown_menu() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json"
    );

    for needle in [
        "\"ui-gallery-select-shadcn-demo-trigger\"",
        "\"select-scroll-viewport\"",
        "\"ui-gallery-combobox-demo-trigger\"",
        "\"ui-gallery-combobox-demo-input\"",
        "\"ui-gallery-combobox-demo-content\"",
        "\"ui-gallery-combobox-demo-listbox\"",
        "\"type\": \"click_stable\"",
        "\"type\": \"wait_overlay_placement_trace\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"ui-gallery-context-menu-demo-trigger\"",
        "\"ui-gallery-context-menu-demo-panel\"",
        "\"ui-gallery-dropdown-menu-demo-trigger.chrome\"",
        "\"ui-gallery-dropdown-menu-demo-profile.chrome\"",
        "\"ui-gallery-popup-menu-narrow-sweep.dropdown-menu\"",
        "\"bounds_within_window\"",
    ] {
        assert!(
            script.contains(needle),
            "popup/menu narrow sweep should keep the selector stable; missing `{needle}`",
        );
    }
}

#[test]
fn select_and_combobox_demo_snippets_clamp_width_inside_narrow_doc_columns() {
    let select_demo = include_str!("../src/ui/snippets/select/demo.rs");
    let combobox_demo = include_str!("../src/ui/snippets/combobox/conformance_demo.rs");
    let normalized_select_demo = normalize_ws(select_demo);
    let normalized_combobox_demo = normalize_ws(combobox_demo);

    assert!(
        normalized_select_demo.contains(".w_full().max_w(Px(180.0)).min_w_0()"),
        "select demo snippet should clamp to the available doc-column width while keeping the upstream 180px max width",
    );
    assert!(
        !select_demo.contains(".w_px(Px(180.0))"),
        "select demo snippet should not force a fixed-width trigger that can overflow the narrow docs column",
    );

    assert!(
        normalized_combobox_demo.contains(".w_full().max_w(Px(260.0)).min_w_0()"),
        "combobox conformance demo should clamp to the available doc-column width while keeping the upstream 260px max width",
    );
    assert!(
        !combobox_demo.contains(".width_px(Px(260.0))"),
        "combobox conformance demo should not force a fixed-width trigger that can overflow the narrow docs column",
    );
}

#[test]
fn popup_menu_narrow_sweep_uses_combobox_content_shell_for_overlay_trace() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/overlay/ui-gallery-popup-menu-narrow-sweep.json"
    );

    assert!(
        script.contains("\"content_test_id\": \"ui-gallery-combobox-demo-content\""),
        "popup/menu narrow sweep should query combobox overlay placement on the positioned content shell",
    );
    assert!(
        script.contains("\"kind\": \"bounds_within_window\"")
            && script.contains("\"id\": \"ui-gallery-combobox-demo-listbox\""),
        "popup/menu narrow sweep should still check inner combobox listbox geometry separately",
    );
    assert!(
        !script.contains("\"content_test_id\": \"ui-gallery-combobox-demo-listbox\""),
        "combobox listbox is an inner geometry target, not the overlay placement content shell",
    );
}

#[test]
fn ui_gallery_overlay_trace_steps_use_stable_selectors() {
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    fn visit_scripts(dir: &Path, checked: &mut usize) {
        for entry in fs::read_dir(dir).expect("diag script directory") {
            let entry = entry.expect("diag script entry");
            let path = entry.path();
            if path.is_dir() {
                visit_scripts(&path, checked);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let text = fs::read_to_string(&path).expect("diag script text");
            let script: Value = serde_json::from_str(&text).unwrap_or_else(|err| {
                panic!(
                    "invalid diag script JSON: path={} err={err}",
                    path.display()
                )
            });
            let Some(steps) = script.get("steps").and_then(Value::as_array) else {
                continue;
            };

            for (index, step) in steps.iter().enumerate() {
                if step.get("type").and_then(Value::as_str) != Some("wait_overlay_placement_trace")
                {
                    continue;
                }

                *checked += 1;
                let query = step.get("query").unwrap_or_else(|| {
                    panic!(
                        "overlay trace step should have a query object: path={} step={index}",
                        path.display()
                    )
                });
                let anchor = query.get("anchor_test_id").and_then(Value::as_str);
                let content = query.get("content_test_id").and_then(Value::as_str);
                assert!(
                    anchor.is_some() || content.is_some(),
                    "overlay trace step should name anchor_test_id or content_test_id so it cannot match an unrelated overlay: path={} step={index}",
                    path.display()
                );
            }
        }
    }

    let scripts_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/diag-scripts/ui-gallery");
    let mut checked = 0usize;
    visit_scripts(&scripts_dir, &mut checked);
    assert!(
        checked >= 50,
        "UI Gallery should keep a broad overlay placement trace corpus; checked={checked}",
    );
}

#[test]
fn ui_gallery_bottom_overlay_trace_steps_declare_non_colliding_viewport() {
    use serde_json::Value;
    use std::fs;
    use std::path::Path;

    fn visit_scripts(dir: &Path, checked: &mut usize, violations: &mut Vec<String>) {
        for entry in fs::read_dir(dir).expect("diag script directory") {
            let entry = entry.expect("diag script entry");
            let path = entry.path();
            if path.is_dir() {
                visit_scripts(&path, checked, violations);
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }

            let text = fs::read_to_string(&path).expect("diag script text");
            let script: Value = serde_json::from_str(&text).unwrap_or_else(|err| {
                panic!(
                    "invalid diag script JSON: path={} err={err}",
                    path.display()
                )
            });
            let Some(steps) = script.get("steps").and_then(Value::as_array) else {
                continue;
            };

            let mut latest_window_height: Option<f64> = None;
            for (index, step) in steps.iter().enumerate() {
                if step.get("type").and_then(Value::as_str) == Some("set_window_inner_size") {
                    latest_window_height = step.get("height_px").and_then(Value::as_f64);
                    continue;
                }

                if step.get("type").and_then(Value::as_str) != Some("wait_overlay_placement_trace")
                {
                    continue;
                }

                let Some(query) = step.get("query") else {
                    continue;
                };
                let expects_bottom =
                    query.get("chosen_side").and_then(Value::as_str) == Some("bottom");
                let expects_no_flip = query.get("flipped").and_then(Value::as_bool) == Some(false);
                if !expects_bottom || !expects_no_flip {
                    continue;
                }

                *checked += 1;
                let Some(height) = latest_window_height else {
                    violations.push(format!(
                        "{} step={index} asserts bottom/no-flip without set_window_inner_size",
                        path.display()
                    ));
                    continue;
                };
                if height < 700.0 {
                    violations.push(format!(
                        "{} step={index} asserts bottom/no-flip with a short viewport height={height}",
                        path.display()
                    ));
                }
            }
        }
    }

    let scripts_dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tools/diag-scripts/ui-gallery");
    let mut checked = 0usize;
    let mut violations = Vec::new();
    visit_scripts(&scripts_dir, &mut checked, &mut violations);

    assert!(
        checked >= 4,
        "UI Gallery should keep explicit bottom/no-flip overlay trace coverage; checked={checked}",
    );
    assert!(
        violations.is_empty(),
        "bottom/no-flip overlay trace scripts must declare a non-colliding viewport; {}",
        violations.join("; ")
    );
}
fn normalize_ws(source: &str) -> String {
    source.split_whitespace().collect()
}
