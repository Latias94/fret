from __future__ import annotations

from pathlib import Path
from typing import Any, Callable


VIEW_RUNTIME_APP_UI_ALIAS_SOURCES = tuple(
    Path(path)
    for path in (
        "apps/fret-examples/src/assets_demo.rs",
        "apps/fret-examples/src/async_playground_demo.rs",
        "apps/fret-examples/src/chart_declarative_demo.rs",
        "apps/fret-examples/src/custom_effect_v1_demo.rs",
        "apps/fret-examples/src/custom_effect_v2_demo.rs",
        "apps/fret-examples/src/custom_effect_v3_demo.rs",
        "apps/fret-examples/src/drop_shadow_demo.rs",
        "apps/fret-examples/src/embedded_viewport_demo.rs",
        "apps/fret-examples/src/external_texture_imports_demo.rs",
        "apps/fret-examples/src/external_video_imports_avf_demo.rs",
        "apps/fret-examples/src/external_video_imports_mf_demo.rs",
        "apps/fret-examples/src/genui_demo.rs",
        "apps/fret-examples/src/hello_counter_demo.rs",
        "apps/fret-examples/src/hello_world_compare_demo.rs",
        "apps/fret-examples/src/image_heavy_memory_demo.rs",
        "apps/fret-examples/src/imui_editor_proof_demo.rs",
        "apps/fret-examples-imui/src/imui_floating_windows_demo.rs",
        "apps/fret-examples-imui/src/imui_hello_demo.rs",
        "apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs",
        "apps/fret-examples-imui/src/imui_response_signals_demo.rs",
        "apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs",
        "apps/fret-examples/src/liquid_glass_demo.rs",
        "apps/fret-examples/src/markdown_demo.rs",
        "apps/fret-examples/src/node_graph_demo.rs",
        "apps/fret-examples/src/postprocess_theme_demo.rs",
        "apps/fret-examples/src/query_async_tokio_demo.rs",
        "apps/fret-examples/src/query_demo.rs",
        "apps/fret-examples/src/todo_demo.rs",
    )
)

VIEW_ENTRY_BUILDER_THEN_RUN_SOURCES = tuple(
    Path(path)
    for path in (
        "apps/fret-examples/src/async_playground_demo.rs",
        "apps/fret-examples/src/chart_declarative_demo.rs",
        "apps/fret-examples/src/drop_shadow_demo.rs",
        "apps/fret-examples/src/genui_demo.rs",
        "apps/fret-examples/src/hello_counter_demo.rs",
        "apps/fret-examples-imui/src/imui_floating_windows_demo.rs",
        "apps/fret-examples-imui/src/imui_hello_demo.rs",
        "apps/fret-examples-imui/src/imui_interaction_showcase_demo.rs",
        "apps/fret-examples-imui/src/imui_response_signals_demo.rs",
        "apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs",
        "apps/fret-examples/src/markdown_demo.rs",
        "apps/fret-examples/src/node_graph_demo.rs",
        "apps/fret-examples/src/query_async_tokio_demo.rs",
        "apps/fret-examples/src/query_demo.rs",
        "apps/fret-examples/src/todo_demo.rs",
    )
)

GROUPED_DATA_SURFACE_SOURCES = tuple(
    Path(path)
    for path in (
        "apps/fret-examples/src/async_playground_demo.rs",
        "apps/fret-examples/src/markdown_demo.rs",
        "apps/fret-examples/src/query_async_tokio_demo.rs",
        "apps/fret-examples/src/query_demo.rs",
    )
)

FRET_QUERY_FACADE_SOURCES = GROUPED_DATA_SURFACE_SOURCES

FRET_DOCKING_OWNER_SOURCES = tuple(
    Path(path)
    for path in (
        "apps/fret-examples/src/container_queries_docking_demo.rs",
        "apps/fret-examples/src/docking_demo.rs",
        "apps/fret-examples/src/docking_arbitration_demo.rs",
        "apps/fret-examples/src/imui_editor_proof_demo.rs",
    )
)

WORKSPACE_SHELL_CAPABILITY_HELPER_REQUIRED = (
    "fn workspace_shell_command_button<'a, Cx>(",
    "Cx: fret::app::ElementContextAccess<'a, App>,",
    "let cx = cx.elements();",
    "workspace_shell_command_button(",
    "fn workspace_shell_editor_rail<'a, Cx>(",
    "workspace_shell_editor_rail(",
    "InspectorPanel::new(None)",
    ".into_element_in(cx,",
    "PropertyGrid::new().into_element_in(cx,",
)

WORKSPACE_SHELL_CAPABILITY_HELPER_FORBIDDEN = (
    "let button = |cx: &mut fret_ui::ElementContext<'_, App>,",
    "fn workspace_shell_editor_rail(cx: &mut fret_ui::ElementContext<'_, App>,",
)


CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]


def check_structural_lane_source_policies(
    failures: list[Any],
    *,
    read_source: ReadSource,
    check_required_forbidden_markers: CheckMarkers,
) -> None:
    render_signatures = (
        "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
        "fn render(&mut self, cx: &mut fret::AppUi<'_, '_, App>) -> fret::Ui",
    )
    for path in VIEW_RUNTIME_APP_UI_ALIAS_SOURCES:
        source = read_source(path)
        required = [] if any(marker in source for marker in render_signatures) else [render_signatures[0]]
        check_required_forbidden_markers(
            path,
            source,
            required=required,
            forbidden=[
                "ViewCx<'_, '_, KernelApp>",
                "ViewCx<'_, '_, App>",
            ],
            failures=failures,
        )

    for path in VIEW_ENTRY_BUILDER_THEN_RUN_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=[".view::<", ".run()"],
            forbidden=[".run_view::<"],
            failures=failures,
        )

    grouped_data_markers = (
        "cx.data().selector_layout(",
        "cx.data().selector(",
        "cx.data().query(",
        "cx.data().query_async(",
        "cx.data().query_async_local(",
    )
    for path in GROUPED_DATA_SURFACE_SOURCES:
        source = read_source(path)
        required = [] if any(marker in source for marker in grouped_data_markers) else [grouped_data_markers[0]]
        check_required_forbidden_markers(
            path,
            source,
            required=required,
            forbidden=[
                "fret_query::ui::QueryElementContextExt",
                "fret_selector::ui::SelectorElementContextExt",
                "cx.use_selector(",
                "cx.use_query(",
                "cx.use_query_async(",
                "cx.use_query_async_local(",
            ],
            failures=failures,
        )

    for path in FRET_QUERY_FACADE_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["use fret::query::{"],
            forbidden=["use fret_query::{"],
            failures=failures,
        )

    for path in FRET_DOCKING_OWNER_SOURCES:
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=["use fret_docking::{"],
            forbidden=["use fret::docking::{"],
            failures=failures,
        )

    workspace_path = Path("apps/fret-examples/src/workspace_shell_demo/driver.rs")
    check_required_forbidden_markers(
        workspace_path,
        read_source(workspace_path),
        required=list(WORKSPACE_SHELL_CAPABILITY_HELPER_REQUIRED),
        forbidden=list(WORKSPACE_SHELL_CAPABILITY_HELPER_FORBIDDEN),
        failures=failures,
    )
