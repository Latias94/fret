from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok, strip_rust_comments
from examples_source_tree_policy.advanced_helpers import (
    ADVANCED_HELPER_CONTEXT_POLICIES,
    check_advanced_helper_context_source_policies,
)
from examples_source_tree_policy.app_facing import (
    APP_FACING_POLICIES,
    check_app_facing_demo_source_policies,
)
from examples_source_tree_policy.core_lanes import (
    ASSET_HELPER_POLICIES,
    LOCAL_STATE_LANE_POLICIES,
    THEME_LANE_POLICIES,
    check_core_lane_source_policies,
)
from examples_source_tree_policy.grouped_state import (
    GROUPED_STATE_POLICIES,
    check_selected_grouped_state_source_policies,
)
from examples_source_tree_policy.interop import (
    LOW_LEVEL_INTEROP_DIRECT_LEAF_SOURCES,
    check_low_level_interop_source_policies,
)
from examples_source_tree_policy.manual import (
    MANUAL_UI_TREE_ROOT_WRAPPER_SOURCES,
    check_manual_ui_tree_source_policies,
)
from examples_source_tree_policy.owner_split import (
    COMPONENTS_GALLERY_OWNER_SPLIT_FORBIDDEN,
    COMPONENTS_GALLERY_OWNER_SPLIT_REQUIRED,
    check_owner_split_source_policies,
)
from examples_source_tree_policy.structural_lanes import (
    GROUPED_DATA_SURFACE_SOURCES,
    VIEW_ENTRY_BUILDER_THEN_RUN_SOURCES,
    VIEW_RUNTIME_APP_UI_ALIAS_SOURCES,
    check_structural_lane_source_policies,
)


GATE_NAME = "examples source tree policy"

EXAMPLES_SRC = Path("apps/fret-examples/src")
IMUI_EXAMPLES_SRC = Path("apps/fret-examples-imui/src")
EXAMPLES_SOURCE_ROOTS = (EXAMPLES_SRC, IMUI_EXAMPLES_SRC)
EXCLUDED_SOURCES = {
    Path("apps/fret-examples/src/lib.rs"),
    Path("apps/fret-examples-imui/src/lib.rs"),
}

ALLOWED_RAW_SHADCN_ESCAPES = (
    "fret::shadcn::raw::prelude::",
    "shadcn::raw::advanced::sync_theme_from_environment(",
    "fret::shadcn::raw::advanced::sync_theme_from_environment(",
    "shadcn::raw::advanced::install_with_ui_services(",
    "fret::shadcn::raw::advanced::install_with_ui_services(",
)

RAW_ACTION_NOTIFY_MARKERS = (
    "use fret::advanced::raw::AppUiRawActionNotifyExt as _;",
    "cx.on_action_notify::<",
    "cx.on_payload_action_notify::<",
)

# These APIs have no authoring role left in the examples tree. Advanced probes may still use
# named `KernelApp`, driver APIs, or an explicitly classified advanced prelude.
REMOVED_AUTHORING_MARKERS = (
    "fret_bootstrap::ui_app(",
    "fret_bootstrap::ui_app_with_hooks(",
    ".run_view::<",
    "ViewCx<'_, '_, KernelApp>",
    "fn render(&mut self, cx: &mut ViewCx<",
    "ActionRegistry",
    "ActionMeta",
    "shadcn::shadcn_themes::",
)

DEFAULT_APP_FORBIDDEN = (
    "use fret::advanced::",
    "advanced::prelude",
    "KernelApp",
    "UiTree<",
    "ModelStore",
    "ui_app_driver::UiAppDriver::new(",
    "fret::advanced::view::view_init_window",
    "fret::advanced::view::view_view",
)


@dataclass(frozen=True)
class Failure:
    path: Path
    line_no: int | None
    message: str
    line: str | None = None


@dataclass(frozen=True)
class SourcePolicy:
    path: Path
    owner: str
    required: tuple[str, ...] = ()
    required_any: tuple[str, ...] = ()
    forbidden: tuple[str, ...] = ()
    include_rust_comments: bool = False


DEFAULT_APP_POLICIES = (
    SourcePolicy(
        EXAMPLES_SRC / "simple_todo_demo.rs",
        "default View/AppUi tutorial",
        required=(
            "use fret::app::prelude::*;",
            "impl View for SimpleTodoView",
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "fret::actions!([",
            "fret::payload_actions!([",
            "cx.state().local",
            "cx.actions()",
            ".payload_update_if::<act::Toggle>",
            ".payload_update_if::<act::Remove>",
            ".action_payload(row.id)",
        ),
        forbidden=DEFAULT_APP_FORBIDDEN,
    ),
    SourcePolicy(
        EXAMPLES_SRC / "hello_counter_demo.rs",
        "default View/AppUi tutorial",
        required=(
            "use fret::app::prelude::*;",
            "impl View for HelloCounterView",
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "fret::actions!([",
            "cx.state().local_init",
            "cx.actions()",
            ".action(act::Inc)",
            ".view::<HelloCounterView>()?",
        ),
        forbidden=DEFAULT_APP_FORBIDDEN,
    ),
    SourcePolicy(
        EXAMPLES_SRC / "todo_demo.rs",
        "default View/AppUi product tutorial",
        required=(
            "use fret::app::prelude::*;",
            "impl View for TodoDemoView",
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "fret::actions!([",
            "fret::payload_actions!([",
            "cx.state().local",
            "cx.actions()",
            ".payload_update_if::<act::Toggle>",
            ".payload_update_if::<act::Remove>",
            ".view::<TodoDemoView>()?",
        ),
        forbidden=DEFAULT_APP_FORBIDDEN,
    ),
    SourcePolicy(
        EXAMPLES_SRC / "query_demo.rs",
        "default query capability tutorial",
        required=(
            "use fret::app::prelude::*;",
            "use fret::query::{",
            "impl View for QueryDemoView",
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "fret::actions!([",
            "cx.data().query(",
            "cx.actions()",
            ".view::<QueryDemoView>()?",
        ),
        forbidden=DEFAULT_APP_FORBIDDEN
        + (
            "use fret_query::{",
            "cx.use_query(",
        ),
    ),
    SourcePolicy(
        EXAMPLES_SRC / "query_async_tokio_demo.rs",
        "default async query capability tutorial",
        required=(
            "use fret::app::prelude::*;",
            "use fret::query::{",
            "impl View for QueryAsyncTokioDemoView",
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "fret::actions!([",
            "cx.data().query_async(",
            "cx.actions()",
            ".view::<QueryAsyncTokioDemoView>()?",
        ),
        forbidden=DEFAULT_APP_FORBIDDEN
        + (
            "use fret_query::{",
            "cx.use_query_async(",
        ),
    ),
)

EDITOR_NOTES_POLICY = SourcePolicy(
    EXAMPLES_SRC / "editor_notes_demo.rs",
    "app-facing editor inspector probe",
    required=(
        "use fret::app::editor::{",
        "use fret::app::prelude::*;",
        "InspectorTextFieldBinding",
        "InspectorTextFieldSnapshot",
        "notes: InspectorTextFieldBinding::new(",
        "asset.notes.paint_snapshot(cx)",
        "asset.notes.text_field(TextFieldOptions",
        "asset.notes.commit_activate()",
        "asset.notes.discard_activate()",
        "TextFieldBlurBehavior::PreserveDraft",
        "a11y_label: Some(Arc::from(\"Asset notes\"))",
        "WorkspaceFrame::new(center)",
        ".view::<EditorNotesDemoView>()?",
    ),
    forbidden=DEFAULT_APP_FORBIDDEN
    + (
        "TextFieldDraftController",
        "fret_ui_editor::controls::text_field::",
    ),
)

DATATABLE_POLICY = SourcePolicy(
    EXAMPLES_SRC / "datatable_demo.rs",
    "app-facing DataTable recipe probe",
    required=(
        "table_state: LocalState<shadcn::TableState>",
        "table_output: LocalState<shadcn::DataTableViewOutput>",
        "table_recipe: shadcn::DataTableRecipe<DemoRow>",
        "shadcn::DataTableRecipe::new(&table_state, &table_output, columns",
        "shadcn::RowKey(row.id)",
        ".column_labels(datatable_column_labels())",
        ".debug_ids(datatable_debug_ids())",
        ".toolbar_test_id_prefix(\"datatable-demo-toolbar\")",
        "table_recipe.into_elements(cx, rows",
        "impl fret::app::View for DemoWindowState",
        ".view::<DemoWindowState>()?",
    ),
    forbidden=DEFAULT_APP_FORBIDDEN
    + (
        "fret_ui_kit::headless::table",
        "create_table(",
        ".view_parts(",
    ),
)

WORKSPACE_SHELL_POLICY = SourcePolicy(
    EXAMPLES_SRC / "workspace_shell_demo/driver.rs",
    "WorkspaceApp real-app probe",
    required=(
        "WorkspaceCommandScope::new(window_layout.clone(), out)",
        "WorkspaceFrame::new(center)",
        "impl fret::workspace::WorkspaceWindowState for WorkspaceShellWindowState",
        "fn workspace_workbench(&self) -> &WorkspaceWorkbench",
        "WorkspaceWorkbench::new(app.models_mut(), window_layout.clone(), block_dirty_close)",
        "fn handle_workspace_command(",
        "record_pending_command_dispatch_source(acx, &cmd, reason)",
        "SemanticsRole",
        "fret::workspace::WorkspaceApp::new(\"workspace-shell-demo\")",
        ".ui(create_window_state, render_workspace_shell)",
        "workspace-shell-frame-stage-trace",
    ),
    forbidden=(
        "WindowCommandDispatchDiagnosticsStore",
        "CommandDispatchDecisionV1",
        "WindowPendingCommandDispatchSourceService",
        "UiTree<",
        ".on_command_before_ui(",
        "ui.dispatch_command(",
        "apply_workspace_model_commands(",
        "ModelStore",
    ),
)

WORKSPACE_FACADE_POLICY = SourcePolicy(
    Path("ecosystem/fret/src/workspace.rs"),
    "workspace startup facade",
    required=(
        "pub struct WorkspaceApp",
        "pub trait WorkspaceWindowState: crate::app::UiAppFrameStageSink",
        "fn workspace_workbench(&self) -> &WorkspaceWorkbench",
        "FretApp::new(root_name).setup(install)",
        "register_workspace_commands(app.commands_mut())",
        "pub fn ui<S: WorkspaceWindowState + 'static>(",
        ".on_app_command_before_ui(handle_workbench_command_from_context::<S>)",
        ".record_frame_stages()",
    ),
    forbidden=("pub fn ui_with_hooks<S:",),
)

WORKSPACE_TYPED_COMMAND_POLICY = SourcePolicy(
    Path("ecosystem/fret-workspace/src/commands.rs"),
    "typed workspace command owner",
    required=(
        "pub mod act {",
        "impl TypedAction for $name",
        "pub fn typed_command_id<A: TypedAction>() -> CommandId",
        "pub fn register_workspace_commands(registry: &mut CommandRegistry)",
        "typed_command_id::<act::WorkspaceTabNext>()",
        "typed_command_id::<act::WorkspacePaneToggleTabStripFocus>()",
    ),
    forbidden=(
        "ActionRegistry",
        "ActionMeta",
    ),
)

WORKSPACE_MENU_POLICY = SourcePolicy(
    Path("ecosystem/fret-workspace/src/menu.rs"),
    "typed workspace menu lowering",
    required=(
        "use crate::commands::{act, typed_command_id};",
        "typed_command_id::<act::WorkspaceTabNext>()",
        "typed_command_id::<act::WorkspacePaneSplitRight>()",
        "typed_command_id::<act::WorkspacePaneFocusRight>()",
    ),
)

ADVANCED_REFERENCE_CLASSIFICATIONS = (
    (
        EXAMPLES_SRC / "custom_effect_v1_demo.rs",
        ("effect/runtime ownership", "renderer/effect ABI"),
    ),
    (
        EXAMPLES_SRC / "custom_effect_v2_demo.rs",
        ("effect/runtime ownership", "renderer/effect ABI"),
    ),
    (
        EXAMPLES_SRC / "custom_effect_v3_demo.rs",
        ("effect/runtime ownership", "renderer/effect ABI and diagnostics pipeline"),
    ),
    (
        EXAMPLES_SRC / "postprocess_theme_demo.rs",
        ("renderer/theme bridge ownership", "high-ceiling post-process story"),
    ),
    (
        EXAMPLES_SRC / "liquid_glass_demo.rs",
        (
            "renderer capability and effect/control graph ownership",
            "glass/warp behavior ceilings",
        ),
    ),
    (
        EXAMPLES_SRC / "genui_demo.rs",
        ("explicit model ownership", "generator/editor integration"),
    ),
    (
        IMUI_EXAMPLES_SRC / "imui_floating_windows_demo.rs",
        ("immediate-mode overlap/floating proof", "IMUI interaction contracts"),
    ),
    (
        IMUI_EXAMPLES_SRC / "imui_interaction_showcase_demo.rs",
        ("product shell polish", "immediate-mode interaction affordances"),
    ),
)

EXAMPLES_DOC_POLICY = SourcePolicy(
    Path("docs/examples/README.md"),
    "examples taxonomy and real-probe guide",
    required=(
        "**Default**:",
        "**Second-hour**:",
        "**Comparison**:",
        "**Advanced**:",
        "`fret::workspace::WorkspaceApp`",
        "`InspectorTextFieldBinding`",
        "`DataTableRecipe`",
        "`datatable_demo` is now a default-clean recipe probe.",
        "`workspace_shell_demo` remains explicitly",
    ),
    forbidden=(
        "temporary advanced/raw allowances for `workspace_shell_demo` and `datatable_demo`",
        "route through `fret`",
    ),
)


def normalize(text: str) -> str:
    return "".join(text.split())


def rel_path(path: Path) -> Path:
    absolute = path if path.is_absolute() else WORKSPACE_ROOT / path
    return absolute.resolve().relative_to(WORKSPACE_ROOT)


def read_source(path: Path) -> str:
    full_path = path if path.is_absolute() else WORKSPACE_ROOT / path
    try:
        return full_path.read_text(encoding="utf-8")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {rel_path(full_path).as_posix()}: {exc}")


def source_slice(path: Path, source: str, start_marker: str, end_marker: str) -> str:
    try:
        start = source.index(start_marker)
    except ValueError:
        fail(
            GATE_NAME,
            f"missing start marker in {rel_path(path).as_posix()}: {start_marker}",
        )
    try:
        end = source.index(end_marker, start)
    except ValueError:
        fail(
            GATE_NAME,
            f"missing end marker in {rel_path(path).as_posix()}: {end_marker}",
        )
    return source[start:end]


def examples_rust_sources() -> list[Path]:
    paths: list[Path] = []
    for root in EXAMPLES_SOURCE_ROOTS:
        paths.extend((WORKSPACE_ROOT / root).rglob("*.rs"))
    return sorted(path for path in paths if rel_path(path) not in EXCLUDED_SOURCES)


def check_source_policy(policy: SourcePolicy, source: str) -> list[Failure]:
    failures: list[Failure] = []
    marker_source = (
        strip_rust_comments(source)
        if policy.path.suffix == ".rs" and not policy.include_rust_comments
        else source
    )
    normalized = normalize(marker_source)
    for marker in policy.required:
        if normalize(marker) not in normalized:
            failures.append(
                Failure(
                    policy.path,
                    None,
                    f"{policy.owner}: missing canonical marker: {marker}",
                )
            )
    if policy.required_any and not any(
        normalize(marker) in normalized for marker in policy.required_any
    ):
        failures.append(
            Failure(
                policy.path,
                None,
                f"{policy.owner}: missing one-of canonical markers: {', '.join(policy.required_any)}",
            )
        )
    for marker in policy.forbidden:
        if normalize(marker) in normalized:
            failures.append(
                Failure(
                    policy.path,
                    None,
                    f"{policy.owner}: forbidden legacy/raw marker: {marker}",
                )
            )
    return failures


def check_required_forbidden_markers(
    path: Path,
    source: str,
    required: list[str],
    forbidden: list[str],
    failures: list[Failure],
) -> None:
    policy = SourcePolicy(
        path=path,
        owner="retained source policy",
        required=tuple(required),
        forbidden=tuple(forbidden),
    )
    failures.extend(check_source_policy(policy, source))


def check_global_source_tree_policy(path: Path, source: str) -> list[Failure]:
    failures: list[Failure] = []
    relative = rel_path(path)

    for line_no, line in enumerate(source.splitlines(), start=1):
        trimmed = line.strip()
        if "fret_ui_shadcn::" in trimmed and not (
            "fret_ui_shadcn::facade" in trimmed
            or "fret_ui_shadcn::{facade" in trimmed
        ):
            failures.append(
                Failure(relative, line_no, "non-curated fret_ui_shadcn import", trimmed)
            )

        if "shadcn::raw::" in trimmed or "fret::shadcn::raw::" in trimmed:
            if not any(marker in trimmed for marker in ALLOWED_RAW_SHADCN_ESCAPES):
                failures.append(
                    Failure(relative, line_no, "undocumented shadcn raw escape hatch", trimmed)
                )

        for marker in RAW_ACTION_NOTIFY_MARKERS:
            if marker in line:
                failures.append(
                    Failure(relative, line_no, f"raw action notify helper: {marker}", trimmed)
                )

        for marker in REMOVED_AUTHORING_MARKERS:
            if marker in line:
                failures.append(
                    Failure(relative, line_no, f"removed authoring surface: {marker}", trimmed)
                )

    normalized = normalize(source)
    if ".setup(|" in normalized or ".setup(move|" in normalized:
        failures.append(Failure(relative, None, "inline `.setup(...)` closure"))
    if "usefret_ui_shadcnasshadcn;" in normalized:
        failures.append(Failure(relative, None, "root shadcn alias bypasses the curated facade"))
    if "usefret_ui_shadcn::{selfasshadcn" in normalized:
        failures.append(Failure(relative, None, "root shadcn alias bypasses the curated facade"))

    return failures


def check_workspace_run_slice(source: str) -> list[Failure]:
    path = WORKSPACE_SHELL_POLICY.path
    start_marker = "pub fn run() -> anyhow::Result<()> {"
    end_marker = "#[cfg(test)]"
    try:
        start = source.index(start_marker)
        end = source.index(end_marker, start)
    except ValueError:
        return [Failure(path, None, "WorkspaceApp real-app probe: missing run-function boundary")]

    policy = SourcePolicy(
        path,
        "WorkspaceApp ordinary launch path",
        required=(
            "fret::workspace::WorkspaceApp::new(\"workspace-shell-demo\")",
            ".ui(create_window_state, render_workspace_shell)",
            ".run()",
        ),
        forbidden=(
            "FnDriver",
            "UiTree<",
            "UiAppDriver::new(",
            "run_native_with_driver(",
            "KernelApp::new()",
        ),
    )
    return check_source_policy(policy, source[start:end])


def check_workspace_command_authoring_slice(
    source: str, path: Path = WORKSPACE_SHELL_POLICY.path
) -> list[Failure]:
    policy = SourcePolicy(
        path,
        "typed workspace command authoring",
        forbidden=(
            '"workspace.tab.',
            '"workspace.pane.',
            "CMD_WORKSPACE_TAB_",
            "CMD_WORKSPACE_PANE_",
        ),
    )
    return check_source_policy(policy, source)


def workspace_shell_rust_sources() -> list[Path]:
    root = WORKSPACE_ROOT / EXAMPLES_SRC / "workspace_shell_demo"
    return sorted(path.relative_to(WORKSPACE_ROOT) for path in root.rglob("*.rs"))


def check_advanced_reference_classifications() -> list[Failure]:
    failures: list[Failure] = []
    for path, reasons in ADVANCED_REFERENCE_CLASSIFICATIONS:
        policy = SourcePolicy(
            path,
            "advanced/reference classification",
            required=(
                "Advanced/reference demo:",
                "Why advanced:",
                "Not a first-contact teaching surface:",
                "reference/product-validation",
                *reasons,
            ),
            include_rust_comments=True,
        )
        failures.extend(check_source_policy(policy, read_source(path)))
    return failures


def collect_failures() -> list[Failure]:
    failures: list[Failure] = []
    for path in examples_rust_sources():
        failures.extend(check_global_source_tree_policy(path, read_source(path)))

    for policy in (
        *DEFAULT_APP_POLICIES,
        EDITOR_NOTES_POLICY,
        DATATABLE_POLICY,
        WORKSPACE_SHELL_POLICY,
        WORKSPACE_FACADE_POLICY,
        WORKSPACE_TYPED_COMMAND_POLICY,
        WORKSPACE_MENU_POLICY,
        EXAMPLES_DOC_POLICY,
    ):
        failures.extend(check_source_policy(policy, read_source(policy.path)))

    workspace_source = read_source(WORKSPACE_SHELL_POLICY.path)
    failures.extend(check_workspace_run_slice(workspace_source))
    for path in workspace_shell_rust_sources():
        failures.extend(check_workspace_command_authoring_slice(read_source(path), path))
    failures.extend(check_advanced_reference_classifications())
    check_app_facing_demo_source_policies(
        failures,
        examples_src=EXAMPLES_SRC,
        default_app_surface_common_forbidden=list(DEFAULT_APP_FORBIDDEN),
        read_source=read_source,
        source_slice=source_slice,
        check_required_forbidden_markers=check_required_forbidden_markers,
    )
    check_advanced_helper_context_source_policies(
        failures,
        examples_src=EXAMPLES_SRC,
        read_source=read_source,
        check_required_forbidden_markers=check_required_forbidden_markers,
    )
    check_selected_grouped_state_source_policies(
        failures,
        examples_src=EXAMPLES_SRC,
        read_source=read_source,
        check_required_forbidden_markers=check_required_forbidden_markers,
    )
    check_low_level_interop_source_policies(
        failures,
        examples_src=EXAMPLES_SRC,
        read_source=read_source,
        check_required_forbidden_markers=check_required_forbidden_markers,
    )
    check_manual_ui_tree_source_policies(
        failures,
        examples_src=EXAMPLES_SRC,
        read_source=read_source,
        check_required_forbidden_markers=check_required_forbidden_markers,
    )
    check_owner_split_source_policies(
        failures,
        examples_src=EXAMPLES_SRC,
        imui_examples_src=IMUI_EXAMPLES_SRC,
        workspace_root=WORKSPACE_ROOT,
        read_source=read_source,
        check_required_forbidden_markers=check_required_forbidden_markers,
    )
    check_core_lane_source_policies(
        failures,
        examples_src=EXAMPLES_SRC,
        imui_examples_src=IMUI_EXAMPLES_SRC,
        read_source=read_source,
        check_required_forbidden_markers=check_required_forbidden_markers,
    )
    check_structural_lane_source_policies(
        failures,
        read_source=read_source,
        check_required_forbidden_markers=check_required_forbidden_markers,
    )
    return failures


def print_failures(failures: list[Failure]) -> None:
    if not failures:
        return

    print(f"[gate] {GATE_NAME}")
    print(f"[gate] FAIL: {len(failures)} source policy problem(s)")
    for failure in failures[:60]:
        location = failure.path.as_posix()
        if failure.line_no is not None:
            location = f"{location}:{failure.line_no}"
        print(f"  - {location}: {failure.message}")
        if failure.line is not None:
            print(f"      {failure.line}")
    if len(failures) > 60:
        print(f"  ... and {len(failures) - 60} more")


def main() -> None:
    failures = collect_failures()
    print_failures(failures)
    if failures:
        raise SystemExit(1)
    ok(GATE_NAME)


if __name__ == "__main__":
    main()
