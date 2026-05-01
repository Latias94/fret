from __future__ import annotations

from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "imui shadcn adapter table column width diag source"

DEMO_PATH = Path("apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs")
SCRIPT_PATH = Path(
    "tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json"
)
SUITE_PATH = Path("tools/diag-scripts/suites/imui-table-column-width-diag-gate/suite.json")

DEMO_MARKERS = [
    "structInspectorColumnWidths",
    "letinspector_widths_state=cx.state().local_init(InspectorColumnWidths::default);",
    "letinspector_widths=inspector_widths_state.layout_value(cx);",
    '"Field###inspector-field",inspector_widths.field',
    ".resizable_with_limits(Some(Px(88.0)),Some(Px(180.0)))",
    "fnapply_inspector_width_delta(",
    "header.resize.drag_delta_x()",
    "header.resize.dragging()",
    "widths_state.update_in(ui.cx_mut().app.models_mut(),|widths|",
    "clamped_width_delta(*width,delta_x,min_width,max_width)",
    'constTEST_ID_TABLE_WIDTHS:&str="imui-shadcn-demo.inspector.widths";',
]

SCRIPT_MARKERS = [
    '"name":"imui-shadcn-adapter-table-column-width-resize"',
    '"type":"drag_pointer_until"',
    '"kind":"label_contains"',
    '"id":"imui-shadcn-demo.inspector.table"',
    '"id":"imui-shadcn-demo.inspector.widths"',
    '"id":"imui-shadcn-demo.inspector.table.header.cell.inspector-field.resize"',
    '"text":"Widths:field104px"',
    '"text":"field180px"',
    '"text":"Widths:field180px"',
    '"type":"capture_layout_sidecar"',
    '"type":"capture_screenshot"',
]

SUITE_MARKERS = [
    '"kind":"diag_script_suite_manifest"',
    '"tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-table-column-width-resize.json"',
]


def normalized_source(path: Path) -> str:
    full_path = WORKSPACE_ROOT / path
    try:
        return "".join(full_path.read_text(encoding="utf-8").split())
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.as_posix()}: {exc}")


def missing_markers(source: str, markers: list[str]) -> list[str]:
    return [marker for marker in markers if marker not in source]


def main() -> None:
    demo = normalized_source(DEMO_PATH)
    script = normalized_source(SCRIPT_PATH)
    suite = normalized_source(SUITE_PATH)

    missing: list[str] = []
    for marker in missing_markers(demo, DEMO_MARKERS):
        missing.append(f"{DEMO_PATH.as_posix()}: {marker}")
    for marker in missing_markers(script, SCRIPT_MARKERS):
        missing.append(f"{SCRIPT_PATH.as_posix()}: {marker}")
    for marker in missing_markers(suite, SUITE_MARKERS):
        missing.append(f"{SUITE_PATH.as_posix()}: {marker}")

    if missing:
        fail(
            GATE_NAME,
            "missing table column width diag proof marker(s):\n  - " + "\n  - ".join(missing),
        )

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
