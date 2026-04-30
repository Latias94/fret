from __future__ import annotations

from pathlib import Path
from typing import Any, Callable, Literal


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

SourceRoot = Literal["examples", "imui_examples"]
RawOwnerSourcePolicy = tuple[SourceRoot, str, list[str], list[str]]


SELECTED_RAW_OWNER_SOURCE_POLICIES: list[RawOwnerSourcePolicy] = [
    (
        "examples",
        "components_gallery.rs",
        [
            "let cx = cx.elements();",
            "let theme = cx.theme_snapshot();",
            "let last_action_value = last_action.layout(cx).value_or_else(|| Arc::<str>::from(\"<none>\"));\n                let cx = cx.elements();\n                let theme_name = cx.theme().name.clone();",
        ],
        [],
    ),
    (
        "examples",
        "editor_notes_device_shell_demo.rs",
        [
            "let (name_value, committed_notes, notes_outcome, summary_status) =",
            "cx.data().selector_model_paint(",
            "&asset.summary_status_model",
        ],
        [
            ".watch_model(&asset.name_model)",
            ".watch_model(&asset.notes_model)",
            ".watch_model(&asset.notes_outcome_model)",
            ".watch_model(&asset.summary_status_model)",
        ],
    ),
    ("examples", "emoji_conformance_demo.rs", ["let cx = cx.elements();"], []),
    ("examples", "form_demo.rs", ["let cx = cx.elements();"], []),
    ("examples", "date_picker_demo.rs", ["let cx = cx.elements();"], []),
    (
        "imui_examples",
        "imui_interaction_showcase_demo.rs",
        ["let cx = cx.elements();"],
        [],
    ),
    (
        "examples",
        "postprocess_theme_demo.rs",
        [
            "let cx = cx.elements();",
            "shadcn::raw::typography::h3(\"Custom effects unavailable\").into_element_in(cx)",
        ],
        [],
    ),
    ("examples", "drop_shadow_demo.rs", ["let cx = cx.elements();"], []),
    ("examples", "ime_smoke_demo.rs", ["let cx = cx.elements();"], []),
    ("examples", "sonner_demo.rs", ["let cx = cx.elements();"], []),
    (
        "examples",
        "custom_effect_v1_demo.rs",
        ["view(cx.elements(), &mut st)"],
        ["view(cx, &mut st)"],
    ),
    (
        "examples",
        "custom_effect_v2_demo.rs",
        ["view(cx.elements(), &mut st)"],
        ["view(cx, &mut st)"],
    ),
    (
        "examples",
        "custom_effect_v3_demo.rs",
        ["view(cx.elements(), &mut st)"],
        ["view(cx, &mut st)"],
    ),
    (
        "examples",
        "liquid_glass_demo.rs",
        ["view(cx.elements(), &mut st)"],
        ["view(cx, &mut st)"],
    ),
    (
        "examples",
        "genui_demo.rs",
        ["view(cx.elements(), &mut self.st)"],
        ["view(cx, &mut self.st)"],
    ),
]

CheckMarkers = Callable[..., None]
ReadSource = Callable[[Path], str]


def check_owner_split_source_policies(
    failures: list[Any],
    *,
    examples_src: Path,
    imui_examples_src: Path,
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

    source_roots = {
        "examples": examples_src,
        "imui_examples": imui_examples_src,
    }
    for source_root, name, required, forbidden in SELECTED_RAW_OWNER_SOURCE_POLICIES:
        path = source_roots[source_root] / name
        check_required_forbidden_markers(
            path,
            read_source(path),
            required=required,
            forbidden=forbidden,
            failures=failures,
        )
