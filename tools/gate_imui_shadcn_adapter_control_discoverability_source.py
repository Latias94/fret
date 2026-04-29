from __future__ import annotations

from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "imui shadcn adapter control discoverability source"

DEMO_PATH = Path("apps/fret-examples/src/imui_shadcn_adapter_demo.rs")
SCRIPT_PATH = Path(
    "tools/diag-scripts/ui-editor/imui/imui-shadcn-adapter-control-discoverability.json"
)

DEMO_MARKERS = [
    'constTEST_ID_ROOT:&str="imui-shadcn-demo.root";',
    'constTEST_ID_CONTROL_CARD:&str="imui-shadcn-demo.controls.card";',
    'constTEST_ID_SUMMARY_CARD:&str="imui-shadcn-demo.summary.card";',
    'constTEST_ID_INSPECTOR_CARD:&str="imui-shadcn-demo.inspector.card";',
    'constTEST_ID_SUMMARY_COUNT:&str="imui-shadcn-demo.summary.count";',
    'constTEST_ID_SUMMARY_ENABLED:&str="imui-shadcn-demo.summary.enabled";',
    'constTEST_ID_SUMMARY_MODE:&str="imui-shadcn-demo.summary.mode";',
    'constTEST_ID_SUMMARY_DRAFT:&str="imui-shadcn-demo.summary.draft";',
    "test_id:Some(Arc::from(TEST_ID_INCREMENT))",
    "test_id:Some(Arc::from(TEST_ID_ENABLED))",
    "test_id:Some(Arc::from(TEST_ID_VALUE))",
    "test_id:Some(Arc::from(TEST_ID_MODE))",
    "test_id:Some(Arc::from(TEST_ID_DRAFT))",
    "summary_badge(",
]

SCRIPT_MARKERS = [
    '"name":"imui-shadcn-adapter-control-discoverability"',
    '"id":"imui-shadcn-demo.root"',
    '"id":"imui-shadcn-demo.controls.increment.chrome"',
    '"id":"imui-shadcn-demo.inspector.card"',
    '"id":"imui-shadcn-demo.controls.enabled.chrome"',
    '"id":"imui-shadcn-demo.controls.value.chrome"',
    '"id":"imui-shadcn-demo.controls.mode.chrome"',
    '"id":"imui-shadcn-demo.controls.draft"',
    '"kind":"bounds_min_size"',
    '"kind":"bounds_non_overlapping"',
    '"id":"imui-shadcn-demo.controls.mode.option.1"',
    '"text":"mode:Beta"',
    '"text":"draft:stagingreview"',
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


def main() -> None:
    demo = normalized_source(DEMO_PATH)
    script = normalized_source(SCRIPT_PATH)

    missing: list[str] = []
    for marker in missing_markers(demo, DEMO_MARKERS):
        missing.append(f"{DEMO_PATH.as_posix()}: {marker}")
    for marker in missing_markers(script, SCRIPT_MARKERS):
        missing.append(f"{SCRIPT_PATH.as_posix()}: {marker}")

    if missing:
        fail(
            GATE_NAME,
            "missing control discoverability proof marker(s):\n  - " + "\n  - ".join(missing),
        )

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
