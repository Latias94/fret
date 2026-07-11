#!/usr/bin/env python3

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from types import ModuleType


TOOLS_DIR = Path(__file__).parent
if str(TOOLS_DIR) not in sys.path:
    sys.path.insert(0, str(TOOLS_DIR))


def load_policy_module() -> ModuleType:
    path = TOOLS_DIR / "examples_source_tree_policy/gate.py"
    spec = importlib.util.spec_from_file_location("examples_source_tree_policy_gate_test", path)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    assert spec.loader is not None
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


POLICY = load_policy_module()


class ExamplesSourceTreePolicyTests(unittest.TestCase):
    def assert_policy_checker_rejects_mutation(
        self,
        checker,
        path: Path,
        marker: str,
        *,
        forbidden: bool = False,
        **checker_kwargs,
    ) -> None:
        source = POLICY.read_source(path)
        if forbidden:
            mutated_source = f"{source}\n{marker}\n"
            expected_message = "forbidden legacy/raw marker"
        else:
            self.assertIn(marker, source)
            mutated_source = source.replace(marker, "")
            expected_message = "missing canonical marker"

        target = POLICY.rel_path(path)

        def fixture_read_source(candidate: Path) -> str:
            if POLICY.rel_path(candidate) == target:
                return mutated_source
            return POLICY.read_source(candidate)

        failures = []
        checker(
            failures,
            read_source=fixture_read_source,
            check_required_forbidden_markers=POLICY.check_required_forbidden_markers,
            **checker_kwargs,
        )

        self.assertTrue(
            any(
                POLICY.rel_path(failure.path) == target
                and expected_message in failure.message
                for failure in failures
            ),
            failures,
        )

    def test_repository_policy_is_green(self) -> None:
        self.assertEqual([], POLICY.collect_failures())

    def test_collect_failures_wires_every_split_checker(self) -> None:
        checker_names = (
            "check_app_facing_demo_source_policies",
            "check_advanced_helper_context_source_policies",
            "check_selected_grouped_state_source_policies",
            "check_low_level_interop_source_policies",
            "check_manual_ui_tree_source_policies",
            "check_owner_split_source_policies",
            "check_core_lane_source_policies",
            "check_structural_lane_source_policies",
        )
        called: set[str] = set()
        originals = {name: getattr(POLICY, name) for name in checker_names}

        def instrument(name, checker):
            def wrapped(*args, **kwargs):
                called.add(name)
                return checker(*args, **kwargs)

            return wrapped

        try:
            for name, checker in originals.items():
                setattr(POLICY, name, instrument(name, checker))
            self.assertEqual([], POLICY.collect_failures())
        finally:
            for name, checker in originals.items():
                setattr(POLICY, name, checker)

        self.assertEqual(set(checker_names), called)

    def test_default_app_requires_the_canonical_prelude(self) -> None:
        policy = POLICY.DEFAULT_APP_POLICIES[0]
        source = POLICY.read_source(policy.path).replace("use fret::app::prelude::*;", "")

        failures = POLICY.check_source_policy(policy, source)

        self.assertTrue(
            any("missing canonical marker: use fret::app::prelude::*;" in f.message for f in failures)
        )

    def test_rust_required_markers_in_comments_do_not_satisfy_policy(self) -> None:
        policy = POLICY.SourcePolicy(
            Path("fixture.rs"),
            "fixture",
            required=("required_call()",),
        )
        source = """
let quote = '"';
// required_call()
/* outer required_call()
   /* nested required_call() */
*/
let ordinary = "// ordinary marker";
let raw = r#"/* raw marker */"#;
"""

        failures = POLICY.check_source_policy(policy, source)

        self.assertTrue(any("missing canonical marker: required_call()" in f.message for f in failures))

    def test_rust_comment_stripping_preserves_markers_in_strings(self) -> None:
        policy = POLICY.SourcePolicy(
            Path("fixture.rs"),
            "fixture",
            required=("// ordinary marker", "/* raw marker */"),
        )
        source = 'let ordinary = "// ordinary marker";\nlet raw = r#"/* raw marker */"#;\n'

        self.assertEqual([], POLICY.check_source_policy(policy, source))

    def test_default_app_rejects_kernel_app_regression(self) -> None:
        policy = POLICY.DEFAULT_APP_POLICIES[0]
        source = POLICY.read_source(policy.path) + "\nuse fret::advanced::KernelApp;\n"

        failures = POLICY.check_source_policy(policy, source)

        self.assertTrue(any("forbidden legacy/raw marker: KernelApp" in f.message for f in failures))

    def test_default_app_rejects_grouped_advanced_prelude_import(self) -> None:
        policy = POLICY.DEFAULT_APP_POLICIES[0]
        source = POLICY.read_source(policy.path) + "\nuse fret::{advanced::prelude::*, shadcn};\n"

        failures = POLICY.check_source_policy(policy, source)

        self.assertTrue(
            any("forbidden legacy/raw marker: advanced::prelude" in f.message for f in failures)
        )

    def test_workspace_policy_pins_workbench_state_and_workspace_app(self) -> None:
        policy = POLICY.WORKSPACE_SHELL_POLICY
        source = POLICY.read_source(policy.path)
        source = source.replace("WorkspaceWorkbench::new", "legacy_workbench")
        source = source.replace("WorkspaceWindowState", "LegacyWorkspaceState")
        source = source.replace("fret::workspace::WorkspaceApp::new", "fret::FretApp::new")

        failures = POLICY.check_source_policy(policy, source)
        messages = "\n".join(f.message for f in failures)

        self.assertIn("missing canonical marker: WorkspaceWorkbench::new", messages)
        self.assertIn(
            "missing canonical marker: impl fret::workspace::WorkspaceWindowState", messages
        )
        self.assertIn(
            'missing canonical marker: fret::workspace::WorkspaceApp::new("workspace-shell-demo")',
            messages,
        )

    def test_workspace_policy_rejects_demo_owned_dispatch_diagnostics(self) -> None:
        policy = POLICY.WORKSPACE_SHELL_POLICY
        source = POLICY.read_source(policy.path) + "\nuse fret_runtime::WindowCommandDispatchDiagnosticsStore;\n"

        failures = POLICY.check_source_policy(policy, source)

        self.assertTrue(
            any(
                "forbidden legacy/raw marker: WindowCommandDispatchDiagnosticsStore"
                in failure.message
                for failure in failures
            )
        )

    def test_workspace_run_slice_rejects_manual_driver_plumbing(self) -> None:
        source = """
pub fn run() -> anyhow::Result<()> {
    fret::workspace::WorkspaceApp::new("workspace-shell-demo")
        .ui_with_hooks(init, render, configure)?
        .run()?;
    let _ = UiAppDriver::new();
    Ok(())
}

#[cfg(test)]
mod tests {}
"""

        failures = POLICY.check_workspace_run_slice(source)

        self.assertTrue(any("forbidden legacy/raw marker: UiAppDriver::new(" in f.message for f in failures))

    def test_workspace_command_authoring_rejects_raw_standard_command_ids(self) -> None:
        regressions = (
            'CommandId::new(Arc::<str>::from("workspace.tab.next"));',
            'CommandId::new("workspace.tab.next");',
            'CommandId::new(Arc::from("workspace.pane.next"));',
            'let id = "workspace.tab.next"; CommandId::new(id);',
            "CommandId::from(CMD_WORKSPACE_TAB_NEXT);",
        )
        for regression in regressions:
            with self.subTest(regression=regression):
                source = POLICY.read_source(POLICY.WORKSPACE_SHELL_POLICY.path).replace(
                    "#[cfg(test)]",
                    f"let _ = {regression}\n\n#[cfg(test)]",
                    1,
                )

                failures = POLICY.check_workspace_command_authoring_slice(source)

                self.assertTrue(
                    any("forbidden legacy/raw marker" in f.message for f in failures)
                )

    def test_workspace_command_authoring_rejects_raw_standard_ids_in_tests_too(self) -> None:
        source = POLICY.read_source(POLICY.WORKSPACE_SHELL_POLICY.path)
        source += '\nCommandId::from("workspace.tab.close.fixture");\n'

        failures = POLICY.check_workspace_command_authoring_slice(source)

        self.assertTrue(
            any(
                'forbidden legacy/raw marker: "workspace.tab.' in failure.message
                for failure in failures
            )
        )

    def test_workspace_command_authoring_scans_sibling_modules(self) -> None:
        sibling = POLICY.EXAMPLES_SRC / "workspace_shell_demo/state.rs"
        source = POLICY.read_source(sibling) + '\nCommandId::from("workspace.tab.close.fixture");\n'

        failures = POLICY.check_workspace_command_authoring_slice(source, sibling)

        self.assertTrue(
            any(
                failure.path == sibling
                and 'forbidden legacy/raw marker: "workspace.tab.' in failure.message
                for failure in failures
            )
        )

    def test_workspace_command_authoring_allows_demo_owned_commands(self) -> None:
        source = POLICY.read_source(POLICY.WORKSPACE_SHELL_POLICY.path).replace(
            "#[cfg(test)]",
            'let _ = CommandId::new(Arc::<str>::from("workspace.shell_demo.fixture"));\n'
            'let _ = CommandId::from("window.close");\n\n#[cfg(test)]',
            1,
        )

        failures = POLICY.check_workspace_command_authoring_slice(source)

        self.assertFalse(
            any("forbidden legacy/raw marker" in failure.message for failure in failures)
        )

    def test_datatable_probe_requires_recipe_and_caller_visible_state(self) -> None:
        policy = POLICY.DATATABLE_POLICY
        source = POLICY.read_source(policy.path).replace("shadcn::DataTableRecipe::new", "legacy_table")

        failures = POLICY.check_source_policy(policy, source)

        self.assertTrue(any("DataTableRecipe::new" in f.message for f in failures))

    def test_editor_probe_requires_inspector_binding(self) -> None:
        policy = POLICY.EDITOR_NOTES_POLICY
        source = POLICY.read_source(policy.path).replace(
            "notes: InspectorTextFieldBinding::new(",
            "notes: legacy_text_field_binding(",
        )

        failures = POLICY.check_source_policy(policy, source)

        self.assertTrue(any("notes: InspectorTextFieldBinding::new(" in f.message for f in failures))

    def test_direct_facade_import_is_curated_but_root_alias_is_not(self) -> None:
        path = POLICY.WORKSPACE_ROOT / POLICY.EXAMPLES_SRC / "fixture.rs"

        facade_failures = POLICY.check_global_source_tree_policy(
            path,
            "use fret_ui_shadcn::facade::{Button, Input};\n",
        )
        root_failures = POLICY.check_global_source_tree_policy(
            path,
            "use fret_ui_shadcn as shadcn;\n",
        )

        self.assertEqual([], facade_failures)
        self.assertTrue(any("root shadcn alias" in f.message for f in root_failures))

    def test_manual_ui_tree_wrapper_policy_rejects_mutations(self) -> None:
        fixtures = (
            (
                "cjk_conformance_demo.rs",
                "ui::children![cx; cjk_conformance_page(cx, theme, card)]",
            ),
            (
                "emoji_conformance_demo.rs",
                "ui::children![cx; emoji_conformance_page(cx, theme, card)]",
            ),
        )
        for source_name, marker in fixtures:
            with self.subTest(source_name=source_name):
                path = POLICY.EXAMPLES_SRC / source_name
                kwargs = {"examples_src": POLICY.EXAMPLES_SRC}
                self.assert_policy_checker_rejects_mutation(
                    POLICY.check_manual_ui_tree_source_policies,
                    path,
                    marker,
                    **kwargs,
                )
                self.assert_policy_checker_rejects_mutation(
                    POLICY.check_manual_ui_tree_source_policies,
                    path,
                    "cx: &mut fret_ui::ElementContext<'_, App>,",
                    forbidden=True,
                    **kwargs,
                )

    def test_low_level_interop_policy_rejects_mutations(self) -> None:
        for source_name in (
            "external_texture_imports_demo.rs",
            "external_video_imports_avf_demo.rs",
            "external_video_imports_mf_demo.rs",
        ):
            with self.subTest(source_name=source_name):
                path = POLICY.EXAMPLES_SRC / source_name
                kwargs = {"examples_src": POLICY.EXAMPLES_SRC}
                self.assert_policy_checker_rejects_mutation(
                    POLICY.check_low_level_interop_source_policies,
                    path,
                    "let show = cx.data().selector_model_layout(&st.show, |show| show);",
                    **kwargs,
                )
                self.assert_policy_checker_rejects_mutation(
                    POLICY.check_low_level_interop_source_policies,
                    path,
                    "cx.observe_model(&st.show, Invalidation::Layout);",
                    forbidden=True,
                    **kwargs,
                )

    def test_components_owner_split_policy_rejects_mutations(self) -> None:
        path = POLICY.EXAMPLES_SRC / "components_gallery.rs"
        kwargs = {
            "examples_src": POLICY.EXAMPLES_SRC,
            "imui_examples_src": POLICY.IMUI_EXAMPLES_SRC,
            "workspace_root": POLICY.WORKSPACE_ROOT,
        }
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_owner_split_source_policies,
            path,
            "fn selected_theme_preset(&self, app: &App) -> Option<Arc<str>> {",
            **kwargs,
        )
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_owner_split_source_policies,
            path,
            "cx.app.models().revision(&table_state).unwrap_or(0);",
            forbidden=True,
            **kwargs,
        )

    def test_advanced_helper_context_policy_rejects_mutations(self) -> None:
        path = POLICY.EXAMPLES_SRC / "assets_demo.rs"
        kwargs = {"examples_src": POLICY.EXAMPLES_SRC}
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_advanced_helper_context_source_policies,
            path,
            "fn assets_page<C>(cx: &mut AppComponentCx<'_>, theme: &ThemeSnapshot, card: C) -> Ui",
            **kwargs,
        )
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_advanced_helper_context_source_policies,
            path,
            "fn render_image_panel(cx: &mut ElementContext<'_, KernelApp>,",
            forbidden=True,
            **kwargs,
        )

    def test_app_facing_policy_rejects_mutations(self) -> None:
        path = POLICY.EXAMPLES_SRC / "query_demo.rs"
        kwargs = {
            "examples_src": POLICY.EXAMPLES_SRC,
            "default_app_surface_common_forbidden": list(POLICY.DEFAULT_APP_FORBIDDEN),
            "source_slice": POLICY.source_slice,
        }
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_app_facing_demo_source_policies,
            path,
            "cx.data().query(",
            **kwargs,
        )
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_app_facing_demo_source_policies,
            path,
            "cx.use_query(",
            forbidden=True,
            **kwargs,
        )

    def test_grouped_state_policy_rejects_mutations(self) -> None:
        path = POLICY.EXAMPLES_SRC / "hello_counter_demo.rs"
        kwargs = {"examples_src": POLICY.EXAMPLES_SRC}
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_selected_grouped_state_source_policies,
            path,
            ".locals_with((&count_state, &step_state))",
            **kwargs,
        )
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_selected_grouped_state_source_policies,
            path,
            "count_state.layout(cx).value_or",
            forbidden=True,
            **kwargs,
        )

    def test_theme_lane_policy_rejects_mutations(self) -> None:
        path = POLICY.EXAMPLES_SRC / "hello_counter_demo.rs"
        kwargs = {
            "examples_src": POLICY.EXAMPLES_SRC,
            "imui_examples_src": POLICY.IMUI_EXAMPLES_SRC,
        }
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_core_lane_source_policies,
            path,
            "let theme = cx.theme_snapshot();",
            **kwargs,
        )
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_core_lane_source_policies,
            path,
            "Theme::global(&*cx.app).snapshot()",
            forbidden=True,
            **kwargs,
        )

    def test_local_state_lane_policy_rejects_mutations(self) -> None:
        path = POLICY.EXAMPLES_SRC / "todo_demo.rs"
        kwargs = {
            "examples_src": POLICY.EXAMPLES_SRC,
            "imui_examples_src": POLICY.IMUI_EXAMPLES_SRC,
        }
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_core_lane_source_policies,
            path,
            "draft: cx.state().local::<String>()",
            **kwargs,
        )
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_core_lane_source_policies,
            path,
            "LocalState::from_model(",
            forbidden=True,
            **kwargs,
        )

    def test_asset_helper_policy_rejects_mutations(self) -> None:
        path = POLICY.EXAMPLES_SRC / "assets_demo.rs"
        kwargs = {
            "examples_src": POLICY.EXAMPLES_SRC,
            "imui_examples_src": POLICY.IMUI_EXAMPLES_SRC,
        }
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_core_lane_source_policies,
            path,
            "ui_assets::rgba8_image_state(",
            **kwargs,
        )
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_core_lane_source_policies,
            path,
            "image_asset_state::use_rgba8_image_state(cx.app",
            forbidden=True,
            **kwargs,
        )

    def test_structural_view_entry_policy_rejects_mutations(self) -> None:
        path = POLICY.EXAMPLES_SRC / "query_demo.rs"
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_structural_lane_source_policies,
            path,
            ".view::<",
        )
        self.assert_policy_checker_rejects_mutation(
            POLICY.check_structural_lane_source_policies,
            path,
            ".run_view::<",
            forbidden=True,
        )


if __name__ == "__main__":
    unittest.main()
