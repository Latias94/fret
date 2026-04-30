from __future__ import annotations

from pathlib import Path
from typing import Any, Callable


COMPONENTS_GALLERY_OWNER_SPLIT_REQUIRED = [
    "impl ComponentsGalleryWindowState {",
    "fn selected_theme_preset(&self, app: &App) -> Option<Arc<str>> {",
    "app.models().get_cloned(&self.theme_preset).flatten()",
    "fn overlays_open(&self, app: &App) -> bool {",
    "app.models().get_copied(&self.select_open).unwrap_or(false)",
    "app.models().get_copied(&self.cmdk_open).unwrap_or(false)",
    "let preset = state.selected_theme_preset(app);",
    "let state_revision = table_state.layout(cx).revision().unwrap_or(0);",
    "let items_revision = 1 ^ state_revision.rotate_left(17);",
    "let items_value = app.models().get_cloned(&items).unwrap_or_default();",
    "let tree_state_value = app.models().get_cloned(&state).unwrap_or_default();",
    "let overlays_open = state.overlays_open(app);",
]

COMPONENTS_GALLERY_OWNER_SPLIT_FORBIDDEN = [
    "cx.app.models().revision(&table_state).unwrap_or(0);",
]

COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT_REQUIRED = [
    "`components_gallery` is not one unresolved raw-model bucket anymore.",
    "retained render owner",
    "driver/event owner",
    "`table_state.layout(cx).revision()`",
    "`selected_theme_preset(app)`",
    "`overlays_open(app)`",
]

CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]


def check_owner_split_source_policies(
    failures: list[Any],
    *,
    examples_src: Path,
    workspace_root: Path,
    read_source: ReadSource,
    check_required_forbidden_markers: CheckMarkers,
) -> None:
    components_path = examples_src / "components_gallery.rs"
    check_required_forbidden_markers(
        components_path,
        read_source(components_path),
        required=COMPONENTS_GALLERY_OWNER_SPLIT_REQUIRED,
        forbidden=COMPONENTS_GALLERY_OWNER_SPLIT_FORBIDDEN,
        failures=failures,
    )

    audit_path = (
        workspace_root
        / "docs/workstreams/public-authoring-state-lanes-and-identity-fearless-refactor-v1/"
        "COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT_2026-04-16.md"
    )
    check_required_forbidden_markers(
        audit_path,
        read_source(audit_path),
        required=COMPONENTS_GALLERY_OWNER_SPLIT_AUDIT_REQUIRED,
        forbidden=[],
        failures=failures,
    )
