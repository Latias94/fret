from __future__ import annotations

from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "imui shadcn adapter sortable table source"

DEMO_PATH = Path("apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs")
SCRIPT_PATH = Path("tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-sortable-table-gate.json")

DEMO_MARKERS = [
    "usefret::{FretApp,advanced::prelude::*,imui::prelude::*};",
    "imui_in(cx,|ui|{",
    "imui(cx,move|ui|{",
    "kit::ButtonOptions{",
    "kit::SwitchOptions{",
    "kit::SliderOptions{",
    "kit::ComboModelOptions{",
    "kit::InputTextOptions{",
    "enumInspectorSort{",
    "TableSortDirection::Ascending",
    'kit::TableColumn::fill("Signal###inspector-signal")',
    "fnsort_rows(self,rows:&mut[InspectorRow])",
    'kit::TableColumn::fill("Field###inspector-field")',
    ".sorted(inspector_sort.direction())",
    "kit::TableOptions{",
    "lettable_response=ui.table_with_options(",
    ".header(sort_column_id)",
    "kit::VirtualListOptions{",
    "kit::VirtualListMeasureMode::Fixed",
]

DEMO_FORBIDDEN_MARKERS = [
    "fret_imui::imui_in(cx,|ui|{",
    "fret_imui::imui(cx,move|ui|{",
    "usefret_ui_kit::imui::UiWriterImUiFacadeExtas_;",
    "usefret_ui_kit::imui::UiWriterUiKitExtas_;",
    "fret_ui_kit::imui::ButtonOptions",
    "fret_ui_kit::imui::SwitchOptions",
    "fret_ui_kit::imui::SliderOptions",
    "fret_ui_kit::imui::ComboModelOptions",
    "fret_ui_kit::imui::InputTextOptions",
    "fret_ui_kit::imui::TableColumn",
    "fret_ui_kit::imui::TableOptions",
    "fret_ui_kit::imui::VirtualListOptions",
]

SCRIPT_MARKERS = [
    '"name":"imui-shadcn-adapter-sortable-table-gate"',
    '"id":"imui-shadcn-demo.inspector.table"',
    '"id":"imui-shadcn-demo.inspector.table.header.cell.inspector-field"',
    '"text":"sortedascending"',
    '"type":"click_stable"',
    '"text":"sorteddescending"',
    '"type":"capture_layout_sidecar"',
    '"type":"capture_screenshot"',
]


def normalized_source(path: Path) -> str:
    full_path = WORKSPACE_ROOT / path
    try:
        return "".join(full_path.read_text(encoding="utf-8").split())
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.as_posix()}: {exc}")


def missing_markers(source: str, markers: list[str]) -> list[str]:
    return [marker for marker in markers if marker not in source]


def present_markers(source: str, markers: list[str]) -> list[str]:
    return [marker for marker in markers if marker in source]


def main() -> None:
    demo = normalized_source(DEMO_PATH)
    script = normalized_source(SCRIPT_PATH)

    missing: list[str] = []
    for marker in missing_markers(demo, DEMO_MARKERS):
        missing.append(f"{DEMO_PATH.as_posix()}: {marker}")
    for marker in missing_markers(script, SCRIPT_MARKERS):
        missing.append(f"{SCRIPT_PATH.as_posix()}: {marker}")

    forbidden = present_markers(demo, DEMO_FORBIDDEN_MARKERS)

    if missing or forbidden:
        details = []
        if missing:
            details.append("missing sortable table proof marker(s):\n  - " + "\n  - ".join(missing))
        if forbidden:
            details.append(
                "found forbidden adapter facade marker(s):\n  - " + "\n  - ".join(forbidden)
            )
        fail(
            GATE_NAME,
            "\n".join(details),
        )

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
