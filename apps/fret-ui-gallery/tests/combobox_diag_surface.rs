#[test]
fn combobox_demo_narrow_diag_script_waits_for_stable_listbox_bounds() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-demo-narrow-open-screenshot.json"
    );

    for needle in [
        "\"ui-gallery-combobox-demo-trigger\"",
        "\"ui-gallery-combobox-demo-input\"",
        "\"ui-gallery-combobox-demo-listbox\"",
        "\"type\": \"click_stable\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"type\": \"wait_overlay_placement_trace\"",
        "\"ui-gallery-combobox-demo-open-narrow\"",
    ] {
        assert!(
            script.contains(needle),
            "combobox narrow diag script should keep the open-chain and placement evidence stable; missing `{needle}`",
        );
    }
}

#[test]
fn combobox_responsive_diag_scripts_pin_exact_viewport_variants() {
    let desktop = include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-open.json"
    );
    let mobile = include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-responsive-vp375x240-open.json"
    );

    for needle in [
        "\"ui-gallery-combobox-responsive-trigger\"",
        "\"ui-gallery-combobox-responsive-input\"",
        "\"ui-gallery-combobox-responsive-content\"",
        "\"ui-gallery-combobox-responsive-command\"",
        "\"ui-gallery-combobox-responsive-listbox\"",
        "\"type\": \"capture_layout_sidecar\"",
        "\"FRET_UI_GALLERY_START_PAGE\": \"combobox\"",
        "\"FRET_UI_GALLERY_START_SECTION\": \"ui-gallery-combobox-responsive-docsec\"",
    ] {
        assert!(
            desktop.contains(needle) && mobile.contains(needle),
            "combobox responsive diag scripts should keep the open-chain and placement evidence stable; missing `{needle}`",
        );
    }

    assert!(
        desktop.contains("\"type\": \"wait_overlay_placement_trace\""),
        "desktop responsive diag script should wait for anchored-panel overlay trace on the content shell",
    );
    {
        let desktop_json: serde_json::Value =
            serde_json::from_str(desktop).expect("desktop responsive combobox script json");
        let trace_query = desktop_json
            .get("steps")
            .and_then(serde_json::Value::as_array)
            .and_then(|steps| {
                steps.iter().find_map(|step| {
                    (step.get("type").and_then(serde_json::Value::as_str)
                        == Some("wait_overlay_placement_trace"))
                    .then(|| step.get("query"))
                    .flatten()
                })
            })
            .expect("desktop responsive combobox overlay trace query");
        assert_eq!(
            trace_query
                .get("side_offset_px")
                .and_then(serde_json::Value::as_f64),
            Some(4.0),
            "responsive combobox desktop script should follow the upstream PopoverContent path; ComboboxContent's v4 default sideOffset=6 is a different source axis",
        );
    }
    assert!(
        !mobile.contains("\"type\": \"wait_overlay_placement_trace\""),
        "mobile responsive diag script should stay on the drawer/layout-surface lane instead of waiting for anchored-panel overlay trace",
    );
    assert!(
        mobile.contains("\"ui-gallery-combobox-responsive-vp375x240-open.preassert.layout\""),
        "mobile responsive diag script should capture a preassert layout sidecar before the first strict bounds check",
    );
    assert!(
        mobile.contains("effective 375x240 viewport"),
        "mobile responsive diag script should document the effective viewport contract",
    );
    assert!(
        mobile.contains("\"kind\": \"window_inner_size_approx_equal\""),
        "mobile responsive diag script should gate the runner-level effective viewport before component geometry",
    );

    for needle in [
        "\"width_px\": 1440.0",
        "\"width_px\": 375.0",
        "\"height_px\": 220.0",
        "\"height_px\": 240.0",
    ] {
        assert!(
            desktop.contains(needle) || mobile.contains(needle),
            "combobox responsive diag scripts should pin exact viewport dimensions; missing `{needle}`",
        );
    }

    assert!(
        desktop.contains("\"ui-gallery-combobox-responsive-open\""),
        "desktop responsive diag script should keep its exact capture label",
    );
    assert!(
        mobile.contains("\"ui-gallery-combobox-responsive-vp375x240-open\""),
        "mobile responsive diag script should keep its exact capture label",
    );
}

#[test]
fn combobox_rtl_flip_diag_script_separates_overlay_shell_from_listbox_geometry() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/combobox/ui-gallery-combobox-flip-tight-window.json"
    );

    for needle in [
        "\"ui-gallery-content-viewport\"",
        "\"ui-gallery-combobox-rtl-trigger\"",
        "\"ui-gallery-combobox-rtl-content\"",
        "\"ui-gallery-combobox-rtl-listbox\"",
        "\"type\": \"scroll_into_view\"",
        "\"type\": \"wait_bounds_stable\"",
        "\"type\": \"wait_overlay_placement_trace\"",
        "\"flipped\": true",
    ] {
        assert!(
            script.contains(needle),
            "combobox RTL flip diag script should keep shell/listbox selectors and the scroll/placement gates stable; missing `{needle}`",
        );
    }

    assert!(
        script.contains("\"content_test_id\": \"ui-gallery-combobox-rtl-content\""),
        "overlay placement traces are recorded for the anchored panel shell, not the internal listbox",
    );
    assert!(
        script.contains("\"kind\": \"bounds_within_window\"")
            && script.contains("\"id\": \"ui-gallery-combobox-rtl-listbox\""),
        "the visible listbox geometry should still be checked separately after the panel placement trace",
    );
}

#[test]
fn combobox_overlay_trace_scripts_target_content_shells_not_inner_listboxes() {
    use serde_json::Value;
    use std::fs;

    let scripts_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/diag-scripts/ui-gallery/combobox"
    );
    let mut checked = 0usize;

    for entry in fs::read_dir(scripts_dir).expect("combobox diag scripts dir") {
        let entry = entry.expect("combobox diag script entry");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }

        let text = fs::read_to_string(&path).expect("combobox diag script text");
        let script: Value = serde_json::from_str(&text).unwrap_or_else(|err| {
            panic!(
                "invalid combobox diag script JSON: path={} err={err}",
                path.display()
            )
        });
        let Some(steps) = script.get("steps").and_then(Value::as_array) else {
            continue;
        };

        for (index, step) in steps.iter().enumerate() {
            if step.get("type").and_then(Value::as_str) != Some("wait_overlay_placement_trace") {
                continue;
            }

            checked += 1;
            let content_test_id = step
                .get("query")
                .and_then(|query| query.get("content_test_id"))
                .and_then(Value::as_str)
                .unwrap_or_else(|| {
                    panic!(
                        "combobox overlay trace step should name content_test_id: path={} step={index}",
                        path.display()
                    )
                });

            assert!(
                !content_test_id.ends_with("-listbox"),
                "combobox overlay placement trace should target the positioned content shell, not the inner listbox: path={} step={index} content_test_id={content_test_id}",
                path.display()
            );
        }
    }

    assert!(
        checked >= 10,
        "combobox diag surface should keep a broad overlay placement trace corpus; checked={checked}",
    );
}

#[test]
fn combobox_responsive_reports_isolate_shell_and_effective_viewport_from_command_parts() {
    use serde_json::Value;
    use std::fs;

    fn report_at(path: &str) -> Value {
        let report_text = fs::read_to_string(path).expect("combobox responsive report text");
        serde_json::from_str(&report_text).expect("valid combobox responsive report")
    }

    fn summary_u64<'a>(summary: &'a serde_json::Map<String, Value>, key: &str, name: &str) -> u64 {
        summary
            .get(key)
            .and_then(Value::as_object)
            .and_then(|counts| counts.get(name))
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("missing summary count `{key}.{name}`"))
    }

    fn summary_scalar_u64(summary: &serde_json::Map<String, Value>, key: &str) -> u64 {
        summary
            .get(key)
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("missing summary scalar `{key}`"))
    }

    fn part_status(report: &Value, part_id: &str) -> String {
        report
            .get("parts")
            .and_then(Value::as_array)
            .and_then(|parts| {
                parts.iter().find_map(|part| {
                    (part.get("id").and_then(Value::as_str) == Some(part_id)).then(|| {
                        part.get("status")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                    })
                })
            })
            .unwrap_or_else(|| panic!("missing report part `{part_id}`"))
            .to_owned()
    }

    let desktop = report_at(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_open_mismatch_report_v1.json",
    ));
    let mobile = report_at(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/workstreams/shadcn-parity-discovery-harness-v1/artifacts/combobox_responsive_vp375x240_open_mismatch_report_v1.json",
    ));

    for (report, expected_part_count, expected_recipe_passes, expected_diag_passes, shell_part) in [
        (&desktop, 4, 3, 0, "desktop_popover_shell_surface"),
        (&mobile, 6, 4, 1, "mobile_drawer_shell_surface"),
    ] {
        let summary = report
            .get("summary")
            .and_then(Value::as_object)
            .expect("report summary object");

        assert_eq!(
            summary.get("part_count").and_then(Value::as_u64),
            Some(expected_part_count),
            "combobox responsive report should keep shell and inner command parts segmented",
        );
        assert_eq!(
            summary_u64(summary, "status_counts", "blocked"),
            0,
            "combobox responsive report should evaluate from measured sidecars, not stay blocked",
        );
        assert_eq!(
            summary_u64(summary, "status_counts", "mismatch"),
            0,
            "combobox responsive report should not carry post-fix shell mismatches",
        );
        assert_eq!(
            summary_u64(summary, "status_counts", "pass_known"),
            expected_part_count,
            "combobox responsive report should keep every segmented part passing after the shell-sizing fix",
        );
        assert_eq!(
            summary_u64(summary, "owner_counts", "mechanism_core"),
            1,
            "combobox responsive shell drift should promote to the mechanism harness lane",
        );
        assert_eq!(
            summary_u64(summary, "owner_counts", "component_recipe"),
            expected_recipe_passes,
            "combobox responsive trigger/command/listbox checks should remain recipe-owned",
        );
        assert_eq!(
            summary_u64(summary, "owner_counts", "gallery_composition"),
            0,
            "combobox responsive shell drift should not be misattributed to gallery composition",
        );
        assert_eq!(
            summary_u64(summary, "owner_counts", "diagnostics_surface"),
            expected_diag_passes,
            "mobile responsive report should gate effective viewport size separately from component geometry",
        );
        assert_eq!(
            summary_u64(summary, "layer_counts", "mechanism"),
            1,
            "combobox responsive shell drift should remain mechanism-layer classified",
        );
        assert_eq!(
            summary_u64(summary, "layer_counts", "recipe"),
            expected_recipe_passes,
            "combobox responsive trigger/command/listbox checks should remain recipe-layer classified",
        );
        assert_eq!(
            summary_u64(summary, "layer_counts", "runner"),
            expected_diag_passes,
            "mobile responsive effective viewport should be runner-layer classified",
        );
        assert_eq!(
            summary_u64(summary, "promotion_target_counts", "mechanism_harness"),
            1,
            "combobox responsive shell drift should have a mechanism-harness promotion target",
        );
        assert_eq!(
            summary_scalar_u64(summary, "upstream_context_count"),
            1,
            "combobox responsive reports should expose the declared upstream viewport/theme context",
        );
        assert_eq!(
            summary_scalar_u64(summary, "upstream_dom_context_count"),
            1,
            "combobox responsive reports should expose the measured upstream DOM context",
        );
        assert_eq!(
            part_status(report, shell_part),
            "pass_known",
            "combobox responsive shell part should pass after shell sizing fixes",
        );
    }

    assert_eq!(
        part_status(&desktop, "desktop_command_root_surface"),
        "pass_known",
        "desktop command root should pass after shell sizing is separated",
    );
    assert_eq!(
        part_status(&desktop, "desktop_listbox_surface"),
        "pass_known",
        "desktop listbox should pass after shell sizing is separated",
    );
    assert_eq!(
        part_status(&mobile, "mobile_effective_viewport"),
        "pass_known",
        "mobile effective viewport guard should pass before drawer geometry is compared",
    );
    assert_eq!(
        part_status(&mobile, "mobile_command_wrapper_surface"),
        "pass_known",
        "mobile command wrapper should pass after drawer shell sizing is separated",
    );
    assert_eq!(
        part_status(&mobile, "mobile_command_root_surface"),
        "pass_known",
        "mobile command root should pass after drawer shell sizing is separated",
    );
    assert_eq!(
        part_status(&mobile, "mobile_listbox_surface"),
        "pass_known",
        "mobile listbox should pass after drawer shell sizing is separated",
    );
}

#[test]
fn window_inner_size_contract_diag_script_is_runner_owned() {
    let script = include_str!(
        "../../../tools/diag-scripts/ui-gallery/window/ui-gallery-window-inner-size-effective-vp375x240.json"
    );

    for needle in [
        "\"runner\"",
        "\"viewport_contract\"",
        "\"type\": \"set_window_inner_size\"",
        "\"width_px\": 375.0",
        "\"height_px\": 220.0",
        "\"kind\": \"window_inner_size_approx_equal\"",
        "\"height_px\": 240.0",
        "\"type\": \"capture_layout_sidecar\"",
        "\"ui-gallery-window-inner-size-effective-vp375x240\"",
    ] {
        assert!(
            script.contains(needle),
            "runner viewport contract script should make requested/effective size evidence explicit; missing `{needle}`",
        );
    }
}
