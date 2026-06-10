#[test]
fn imui_editor_workbench_demo_is_the_canonical_editor_workbench_route() {
    let route_source = include_str!("../src/imui_editor_workbench_demo.rs");
    let quick_actions_source = include_str!("../src/imui_editor_workbench_demo/quick_actions.rs");
    let quick_action_catalog_source =
        include_str!("../src/imui_editor_workbench_demo/quick_actions/catalog.rs");
    let quick_action_copy_source =
        include_str!("../src/imui_editor_workbench_demo/quick_actions/copy.rs");
    let shared_first_open_source = include_str!("../../fret-first-open/src/lib.rs");
    let editor_notes_source = include_str!("../src/editor_notes_demo.rs");
    let lib_source = include_str!("../src/lib.rs");
    let bin_source = include_str!("../../fret-demo/src/bin/imui_editor_workbench_demo.rs");
    let demo_main_source = include_str!("../../fret-demo/src/main.rs");
    let workbench_contract_source = format!(
        "{route_source}\n{quick_actions_source}\n{quick_action_catalog_source}\n{quick_action_copy_source}\n{shared_first_open_source}"
    );

    for needle in [
        "Canonical IMUI editor workbench route.",
        "stable product-facing editor workbench entrypoint",
        "editor-notes workflow as the first converged editor workflow",
        "Demo/Metrics/Debug route visible as persistent workbench chrome",
        "FretApp::new(\"imui-editor-workbench-demo\")",
        ".window(\"imui_editor_workbench_demo\"",
        "crate::editor_notes_demo::install_editor_notes_demo_theme",
        ".view::<ImUiEditorWorkbenchView>()?",
        "struct ImUiEditorWorkbenchView",
        "notes: crate::editor_notes_demo::EditorNotesDemoView",
        "crate::editor_notes_demo::EditorNotesDemoView::init(app, window)",
        "self.notes.render(cx)",
        "mod quick_actions;",
        "quick_actions::render_workbench_quick_action_strip(cx)",
    ] {
        assert!(
            route_source.contains(needle),
            "canonical IMUI workbench route should own the app shell and mount the first converged editor workflow; missing `{needle}`"
        );
    }

    assert!(
        editor_notes_source.contains("pub(crate) struct EditorNotesDemoView"),
        "editor notes should expose its reusable workflow view to the canonical workbench route"
    );
    for needle in [
        "theme_preset_model: Model<EditorThemePresetV1>",
        "fret_ui_editor::theme::installed_editor_theme_preset_v1(app)",
        "EditorThemePresetPicker::new(theme_preset_model.clone())",
        "EditorThemePresetPickerOptions",
        "row_cx.label_text(cx, \"Theme preset\")",
        "TEST_ID_THEME_PRESET_PICKER",
    ] {
        assert!(
            editor_notes_source.contains(needle),
            "canonical editor-notes workflow should surface the editor-owned theme preset picker in the inspector; missing `{needle}`"
        );
    }
    for needle in [
        "const TEST_ID_ACTION_STRIP: &str = \"imui-editor-workbench.action-strip\";",
        "const TEST_ID_ACTION_COMMAND: &str = \"imui-editor-workbench.action-command\";",
        "const TEST_ID_ACTION_COPY_SELECTED: &str = \"imui-editor-workbench.action.copy-selected-command\";",
        "const TEST_ID_ACTION_COPY_BUNDLE: &str = \"imui-editor-workbench.action.copy-command-bundle\";",
        "const TEST_ID_ACTION_COPY_STATUS: &str = \"imui-editor-workbench.action-copy-status\";",
        "const TEST_ID_WORKFLOW: &str = \"imui-editor-workbench.workflow\";",
        "const WORKBENCH_QUICK_ACTIONS: &[WorkbenchQuickActionSpec]",
        "WorkbenchQuickAction::Workbench",
        "WorkbenchQuickAction::Proof",
        "WorkbenchQuickAction::Metrics",
        "WorkbenchQuickAction::Debug",
        "WorkbenchQuickAction::Wayland",
        "cargo run -p fret-demo --bin imui_editor_workbench_demo",
        "cargo run -p fret-demo --bin imui_editor_proof_demo",
        "cargo run -p fretboard-dev -- diag stats <bundle-or-dir> --json",
        "cargo run -p fretboard-dev -- diag trace <bundle-or-dir> --json",
        "imui-p3-wayland-real-host",
        "DevTools and fretboard own execution.",
        "Ready to copy the selected command or the full command bundle.",
        "fn workbench_quick_action_command_bundle_text() -> String {",
        "fn workbench_copy_text_on_activate(",
        "Effect::ClipboardWriteText",
        "shadcn::Button::new(\"Copy command\")",
        "shadcn::Button::new(\"Copy commands\")",
        ".on_activate(workbench_copy_text_on_activate(",
        "active_spec.command.to_string(),",
        "workbench_quick_action_command_bundle_text(),",
        "Copied Demo/Metrics/Debug command bundle.",
        ".test_id(TEST_ID_ACTION_COPY_STATUS)",
    ] {
        assert!(
            workbench_contract_source.contains(needle),
            "canonical workbench route should keep Demo/Metrics/Debug quick actions resident in first-open chrome; missing `{needle}`"
        );
    }
    for needle in [
        "const WORKBENCH_QUICK_ACTIONS: &[WorkbenchQuickActionSpec]",
        "pub(super) fn install_workbench_quick_action_commands(",
        "pub(super) fn workbench_quick_action_command(",
        "fn workbench_quick_action_command_bundle_text() -> String {",
        "SelectWorkbench = \"imui_editor_workbench_demo.action.open_workbench.v1\"",
    ] {
        assert!(
            quick_action_catalog_source.contains(needle),
            "canonical workbench quick-action catalog should carry Demo/Metrics/Debug command metadata; missing `{needle}`"
        );
        assert!(
            !route_source.contains(needle),
            "canonical workbench route root should delegate Demo/Metrics/Debug command metadata to quick_actions/catalog; unexpected `{needle}`"
        );
        assert!(
            !quick_actions_source.contains(needle),
            "canonical workbench quick-action render owner should delegate Demo/Metrics/Debug command metadata to catalog; unexpected `{needle}`"
        );
    }
    for needle in [
        "mod catalog;",
        "mod copy;",
        "install_workbench_quick_action_commands(cx, &active_action);",
        "copy::initial_workbench_copy_status",
        "copy::render_workbench_quick_action_copy_buttons(",
        "copy::render_workbench_quick_action_copy_status(cx, copy_status)",
    ] {
        assert!(
            quick_actions_source.contains(needle),
            "canonical workbench quick-action render owner should carry resident chrome and delegate copy behavior; missing `{needle}`"
        );
    }
    for needle in [
        "fn workbench_copy_text_on_activate(",
        "Effect::ClipboardWriteText",
        "shadcn::Button::new(\"Copy command\")",
        "shadcn::Button::new(\"Copy commands\")",
        ".on_activate(workbench_copy_text_on_activate(",
        "active_spec.command.to_string(),",
        "workbench_quick_action_command_bundle_text(),",
        "Copied Demo/Metrics/Debug command bundle.",
        ".test_id(TEST_ID_ACTION_COPY_STATUS)",
    ] {
        assert!(
            quick_action_copy_source.contains(needle),
            "canonical workbench quick-action copy owner should carry clipboard affordance behavior; missing `{needle}`"
        );
        assert!(
            !quick_actions_source.contains(needle),
            "canonical workbench quick-action render owner should delegate clipboard affordance behavior to copy.rs; unexpected `{needle}`"
        );
        assert!(
            !quick_action_catalog_source.contains(needle),
            "canonical workbench quick-action catalog should not own clipboard affordance behavior; unexpected `{needle}`"
        );
    }
    assert!(
        lib_source.contains("pub mod imui_editor_workbench_demo;"),
        "fret-examples should export the canonical IMUI workbench route"
    );
    assert!(
        lib_source.contains("EditorThemePresetV1::from_key(key)")
            && lib_source.contains("parse_editor_theme_preset_key(\"IMGUI-LIKE-DENSE\")"),
        "fret-examples should route preset parsing through the editor-owned canonical parser"
    );
    assert!(
        bin_source.contains("fret_examples::imui_editor_workbench_demo::run()"),
        "fret-demo should expose a direct canonical IMUI workbench binary"
    );
    assert!(
        demo_main_source.contains("imui_editor_workbench_demo"),
        "fret-demo -- --list should advertise the canonical IMUI workbench route"
    );
    assert!(
        demo_main_source
            .contains("\"imui_editor_workbench_demo\" | \"imui-editor-workbench-demo\""),
        "fret-demo should dispatch the canonical IMUI workbench route by snake_case and kebab-case"
    );

    for unexpected in [
        "crate::workspace_shell_demo::run()",
        "imui_editor_proof_demo::run()",
        "editor_notes_demo::run()",
        "docking_arbitration_demo::run()",
    ] {
        assert!(
            !route_source.contains(unexpected),
            "canonical route should compose reusable workflow views instead of hopping between supporting proof demo run functions; unexpected `{unexpected}`"
        );
    }
}
#[test]
fn imui_editor_workbench_demo_is_promoted_in_docs_and_discovery() {
    let examples_readme = include_str!("../../../docs/examples/README.md");
    let cookbook_readme = include_str!("../../fret-cookbook/README.md");
    let cookbook_examples = include_str!("../../fret-cookbook/EXAMPLES.md");
    let diagnostics_first_open = include_str!("../../../docs/diagnostics-first-open.md");
    let shared_first_open_source = include_str!("../../fret-first-open/src/lib.rs");
    let fretboard_demos = include_str!("../../fretboard/src/demos.rs");
    let devtools_native = include_str!("../../fret-devtools/src/native.rs");
    let devtools_command_catalog =
        include_str!("../../fret-devtools/src/native/command_catalog.rs");
    let devtools_demo_metrics_debug = include_str!("../../fret-devtools/src/demo_metrics_debug.rs");
    let product_chain_gate = include_str!("../../../tools/diag_gate_imui_product_chain.py");
    let shared_discovery_contract = format!("{fretboard_demos}\n{shared_first_open_source}");

    for (name, source) in [
        ("docs/examples/README.md", examples_readme),
        ("apps/fret-cookbook/README.md", cookbook_readme),
        ("apps/fret-cookbook/EXAMPLES.md", cookbook_examples),
    ] {
        assert!(
            source.contains("cargo run -p fret-demo --bin imui_editor_workbench_demo"),
            "{name} should promote the canonical editor workbench route"
        );
        assert!(
            source.contains("supporting"),
            "{name} should classify older proof demos as supporting surfaces"
        );
    }

    assert!(
        examples_readme.contains("Product workbench"),
        "examples docs should name the canonical IMUI product workbench section"
    );
    assert!(
        diagnostics_first_open.contains("canonical editor workbench demo"),
        "diagnostics first-open docs should describe the workbench in first_open_routes"
    );
    assert!(
        diagnostics_first_open.contains("supporting editor proof"),
        "diagnostics first-open docs should demote the older proof to supporting status"
    );

    for (name, source) in [
        ("apps/fretboard/src/demos.rs", fretboard_demos),
        (
            "apps/fret-devtools/src/demo_metrics_debug.rs",
            devtools_demo_metrics_debug,
        ),
        ("tools/diag_gate_imui_product_chain.py", product_chain_gate),
    ] {
        assert!(
            source.contains("demo editor workbench"),
            "{name} should expose the canonical workbench label in the Demo/Metrics/Debug route"
        );
        assert!(
            source.contains("demo editor proof supporting"),
            "{name} should keep the old proof discoverable only as supporting evidence"
        );
    }

    for (name, source) in [
        (
            "apps/fretboard/src/demos.rs",
            shared_discovery_contract.as_str(),
        ),
        ("tools/diag_gate_imui_product_chain.py", product_chain_gate),
    ] {
        assert!(
            source.contains("cargo run -p fret-demo --bin imui_editor_workbench_demo"),
            "{name} should expose the canonical workbench command"
        );
    }

    assert!(
        devtools_demo_metrics_debug.contains("DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND")
            && devtools_demo_metrics_debug.contains("DEVTOOLS_DEMO_EDITOR_PROOF_COMMAND"),
        "apps/fret-devtools/src/demo_metrics_debug.rs should project shared DevTools demo command constants"
    );

    assert!(
        devtools_native.contains("use command_catalog::*;")
            && devtools_command_catalog.contains("const DEVTOOLS_DEMO_EDITOR_WORKBENCH_COMMAND")
            && shared_first_open_source.contains("pub const DEMO_EDITOR_WORKBENCH_COMMAND: &str =")
            && shared_first_open_source
                .contains("cargo run -p fret-demo --bin imui_editor_workbench_demo"),
        "apps/fret-devtools/src/native/command_catalog.rs should keep the shared canonical workbench command alias and native.rs should import the catalog"
    );
}
