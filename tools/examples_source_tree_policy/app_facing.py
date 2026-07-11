from __future__ import annotations

from pathlib import Path
from typing import Any, Callable


APP_FACING_POLICIES = (
    (
        "hello_counter_demo.rs",
        (
            "use fret::app::prelude::*;",
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "cx.state().local_init",
            "cx.data().selector_layout(",
            "cx.actions().locals_with(",
            "cx.actions().local(",
        ),
        (
            "KernelApp",
            "ModelStore",
            "cx.use_local_with(",
            "cx.on_action_notify",
        ),
    ),
    (
        "query_demo.rs",
        (
            "use fret::query::{",
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "cx.state().local_init",
            "cx.data().query(",
            "query_handle.read_layout(cx)",
            "cx.actions().local(",
        ),
        (
            "use fret_query::{",
            "cx.use_query(",
            "with_query_client(",
            "ModelStore",
        ),
    ),
    (
        "query_async_tokio_demo.rs",
        (
            "use fret::query::{",
            "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
            "cx.state().local_init",
            "cx.data().query_async(",
            "query_handle.read_layout(cx)",
            "cx.actions().local(",
        ),
        (
            "use fret_query::{",
            "cx.use_query_async(",
            "with_query_client(",
            "ModelStore",
        ),
    ),
    (
        "todo_demo.rs",
        (
            "use fret::app::prelude::*;",
            "struct TodoLocals {",
            "fn new(cx: &mut AppUi<'_, '_>) -> Self",
            "draft: cx.state().local::<String>()",
            "todos: cx.state().local_init",
            "cx.actions().locals_with(",
            "cx.actions().local(",
            ".payload_update_if::<act::Toggle>",
        ),
        (
            "KernelApp",
            "ModelStore",
            "TodoLocals::new(app)",
            "LocalState::from_model(",
            "cx.on_action_notify",
        ),
    ),
    (
        "editor_notes_demo.rs",
        (
            "use fret::app::editor::{",
            "InspectorTextFieldBinding",
            "InspectorTextFieldSnapshot",
            "notes: InspectorTextFieldBinding",
            "editor_asset_paint_snapshot(cx, &asset)",
            "asset.notes.text_field(TextFieldOptions",
            "asset.notes.commit_activate()",
            "asset.notes.discard_activate()",
            "WorkspaceFrame::new(center)",
        ),
        (
            "TextFieldDraftController",
            "EditorNotesModelOwner",
            "selector_model_paint(",
            "ModelStore",
        ),
    ),
    (
        "datatable_demo.rs",
        (
            "table_state: LocalState<shadcn::TableState>",
            "table_output: LocalState<shadcn::DataTableViewOutput>",
            "table_recipe: shadcn::DataTableRecipe<DemoRow>",
            "shadcn::DataTableRecipe::new(&table_state, &table_output, columns",
            "table_recipe.into_elements(cx, rows",
        ),
        (
            "fret_ui_kit::headless::table",
            "create_table(",
            ".view_parts(",
        ),
    ),
)


CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]


def check_app_facing_demo_source_policies(
    failures: list[Any],
    *,
    examples_src: Path,
    default_app_surface_common_forbidden: list[str],
    read_source: ReadSource,
    source_slice: Callable[..., str],
    check_required_forbidden_markers: CheckMarkers,
) -> None:
    del default_app_surface_common_forbidden, source_slice
    for source_name, required, forbidden in APP_FACING_POLICIES:
        path = examples_src / source_name
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=list(required),
            forbidden=list(forbidden),
            failures=failures,
        )
