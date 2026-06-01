from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "docking multiwindow workstream source"


COMMON_SUITE = Path("tools/diag-scripts/suites/docking-arbitration/common/suite.json")
WINDOWS_SUITE = Path("tools/diag-scripts/suites/docking-arbitration/windows/suite.json")
ROOT_SUITE = Path("tools/diag-scripts/suites/docking-arbitration/suite.json")
SMOKE_SUITE = Path("tools/diag-scripts/suites/diag-hardening-smoke-docking/suite.json")
WAYLAND_CAMPAIGN = Path("tools/diag-campaigns/imui-p3-wayland-real-host.json")
WAYLAND_SCRIPT = Path(
    "tools/diag-scripts/docking/arbitration/docking-arbitration-demo-wayland-degrade-no-os-tearoff.json"
)
WAYLAND_POLICY_SKIP_GATE = Path("tools/diag_gate_docking_wayland_policy_skip.py")

REQUIRED_COMMON_SCRIPTS = [
    "tools/diag-scripts/docking-arbitration-demo-multiwindow-drag-tab-back-to-main-large-outer-move.json",
    "tools/diag-scripts/docking-arbitration-demo-multiwindow-five-way-hints-sweep.json",
    "tools/diag-scripts/docking-arbitration-demo-multiwindow-overlap-zorder-switch.json",
    "tools/diag-scripts/docking-arbitration-demo-multiwindow-under-moving-window-tabs-group.json",
    "tools/diag-scripts/docking-arbitration-demo-tab-bar-drop-end-insert-index-overflow.json",
    "tools/diag-scripts/docking-arbitration-demo-tab-bar-edge-autoscroll.json",
]

REQUIRED_WINDOWS_SCRIPTS = [
    "tools/diag-scripts/docking-arbitration-demo-multiwindow-release-outside-windows-poll-up.json",
]

REQUIRED_SMOKE_SCRIPTS = [
    "tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-tearoff-tabs-group-two-tabs-merge-back.json",
    "tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-title-bar-drag-docks-to-main.json",
    "tools/diag-scripts/docking/arbitration/docking-arbitration-demo-multiwindow-under-moving-window-basic.json",
    "tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-overflow-menu-close-row-1-does-not-activate.json",
    "tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-overflow-menu-select-row-1-activates.json",
]

EXISTENCE_ONLY_SCRIPTS = [
    "tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-overflow-button-is-not-drop-surface.json",
]


def _read_text(path: Path, failures: list[str]) -> str:
    try:
        return (WORKSPACE_ROOT / path).read_text(encoding="utf-8")
    except OSError as exc:
        failures.append(f"{path.as_posix()}: failed to read: {exc}")
        return ""


def _read_json(path: Path, failures: list[str]) -> Any:
    source = _read_text(path, failures)
    if not source:
        return None
    try:
        return json.loads(source)
    except json.JSONDecodeError as exc:
        failures.append(f"{path.as_posix()}: invalid JSON: {exc}")
        return None


def _suite_scripts(path: Path, failures: list[str]) -> list[str]:
    payload = _read_json(path, failures)
    if not isinstance(payload, dict):
        return []
    if payload.get("kind") != "diag_script_suite_manifest":
        failures.append(f"{path.as_posix()}: expected diag_script_suite_manifest")
    scripts = payload.get("scripts")
    if not isinstance(scripts, list) or not all(isinstance(item, str) for item in scripts):
        failures.append(f"{path.as_posix()}: expected string scripts list")
        return []
    return scripts


def _require_markers(
    path: Path,
    *,
    required: list[str],
    forbidden: list[str] | None = None,
    failures: list[str],
) -> None:
    source = _read_text(path, failures)
    for marker in required:
        if marker not in source:
            failures.append(f"{path.as_posix()}: missing marker {marker}")
    for marker in forbidden or []:
        if marker in source:
            failures.append(f"{path.as_posix()}: forbidden stale marker {marker}")


def _require_suite_members(
    path: Path,
    *,
    required: list[str],
    failures: list[str],
    max_len: int | None = None,
) -> list[str]:
    scripts = _suite_scripts(path, failures)
    script_set = set(scripts)
    for script in required:
        if script not in script_set:
            failures.append(f"{path.as_posix()}: missing suite script {script}")
    if max_len is not None and len(scripts) > max_len:
        failures.append(f"{path.as_posix()}: expected at most {max_len} scripts, found {len(scripts)}")
    return scripts


def _require_script_paths(scripts: list[str], failures: list[str]) -> None:
    for script in scripts:
        path = Path(script)
        absolute = WORKSPACE_ROOT / path
        if not absolute.exists():
            failures.append(f"{script}: script path does not exist")
            continue
        payload = _read_json(path, failures)
        if isinstance(payload, dict) and set(payload.keys()) == {"to"}:
            target = payload.get("to")
            if not isinstance(target, str):
                failures.append(f"{script}: redirect target must be a string")
                continue
            if not (WORKSPACE_ROOT / target).exists():
                failures.append(f"{script}: redirect target does not exist: {target}")


def _check_docs(failures: list[str]) -> None:
    _require_markers(
        Path("docs/workstreams/standalone/docking-multi-window-imgui-alignment-v1.md"),
        required=[
            "Status: Active reference (partially superseded by `docs/workstreams/docking-multiwindow-imgui-parity/`)",
            "Status note (2026-05-14): current execution state lives in `docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json`",
            "Tab overflow + scrolling: delivered for the current docking arbitration tab-bar scope",
            "`tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-bar-edge-autoscroll.json`",
            "`tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-overflow-menu-select-row-1-activates.json`",
            "`tools/diag-scripts/docking/arbitration/docking-arbitration-demo-tab-overflow-menu-close-row-1-does-not-activate.json`",
            "`tools/diag-scripts/suites/docking-arbitration/common/suite.json`",
            "`tools/diag-scripts/suites/diag-hardening-smoke-docking/suite.json`",
            "Keep the docking suite split honest",
        ],
        forbidden=[
            "Tab overflow + scrolling: ensure overflow behavior is predictable and stable under resize (and ideally gate it).",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M16_SOURCE_DRIFT_GUARD_2026-05-14.md"),
        required=[
            "Status: source drift guard; no behavior change.",
            "python tools/gate_docking_multiwindow_workstream_source.py",
            "The standalone behavior-first note no longer teaches tab overflow as an ungated gap.",
            "`tools/diag-scripts/suites/docking-arbitration/common/suite.json`",
            "`tools/diag-scripts/suites/docking-arbitration/windows/suite.json`",
            "`tools/diag-scripts/suites/diag-hardening-smoke-docking/suite.json`",
            "2026-05-15",
            "`tools/diag-campaigns/imui-p3-wayland-real-host.json`",
            "`tools/diag-scripts/docking/arbitration/docking-arbitration-demo-wayland-degrade-no-os-tearoff.json`",
            "`imui-p3-wayland-real-host` stays a manual, host-admitted campaign",
            "The canonical Wayland degradation script still waits for hover detection `none`",
            "asserts `known_window_count_is(n=1)`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M17_LOCAL_WAYLAND_POLICY_SKIP_GATE_2026-05-15.md"),
        required=[
            "Status: local policy-skip gate; no Wayland acceptance claim.",
            "python tools/diag_gate_docking_wayland_policy_skip.py",
            "`capabilities.json` with `diag.script_v2`",
            "`environment.requirement_unsatisfied`",
            "`environment.platform_capabilities.platform_ne`",
            "script item files are not",
            "produced under `script-results/` or `suite-results/`",
            "does not close `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md"),
        required=[
            "Status: local policy-skip matrix refresh; no Wayland acceptance claim.",
            "python tools/diag_gate_docking_wayland_policy_skip.py --reuse-built",
            "Windows sidecar",
            "Linux Wayland-style sidecar",
            "Linux/X11-style sidecar",
            "`environment.platform_capabilities.platform_ne`",
            "`environment.platform_capabilities.ui_multi_window_ne`",
            "`environment.platform_capabilities.ui_window_tear_off_ne`",
            "`environment.platform_capabilities.ui_window_hover_detection_ne`",
            "`environment.platform_capabilities.ui_window_z_level_ne`",
            "produced under `script-results/` or `suite-results/`",
            "does not close `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md"),
        required=[
            "Status: active capture runbook",
            "Run this only on a Linux native Wayland session.",
            "FRET_DOCK_TEAROFF_LOG=1 cargo run -p fretboard-dev -- diag campaign run",
            "imui-p3-wayland-real-host",
            "FRET_DOCK_TEAROFF_LOG=1 cargo run -p fretboard-dev -- diag run",
            "docking-arbitration-demo-wayland-degrade-no-os-tearoff.json",
            "known_window_count_is(n=1)",
            "diag windows <bundle_dir> --json",
            "diag dock-graph <bundle_dir> --json",
            "no matches for the acceptance run",
            "non-qualifying host should produce `check.environment.json`",
            "should not be counted as a compositor",
            "When a run satisfies the checklist above, record it in a new dated evidence note",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json"),
        required=[
            "\"updated\": \"2026-06-02\"",
            "M16_SOURCE_DRIFT_GUARD_2026-05-14.md",
            "M17_LOCAL_WAYLAND_POLICY_SKIP_GATE_2026-05-15.md",
            "M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md",
            "M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md",
            "M20_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-26.md",
            "M21_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-30.md",
            "M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md",
            "M23_DOCKING_RUNTIME_TEAR_OFF_OWNER_SPLIT_2026-05-31.md",
            "M24_DOCKING_RUNTIME_IN_WINDOW_OWNER_SPLIT_2026-05-31.md",
            "M25_DOCKING_RUNTIME_TEAR_OFF_CREATE_REQUEST_OWNER_SPLIT_2026-06-01.md",
            "M26_DOCKING_RUNTIME_TEAR_OFF_CANCELLATION_OWNER_SPLIT_2026-06-01.md",
            "M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md",
            "M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md",
            "M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md",
            "M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md",
            "M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md",
            "M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md",
            "M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md",
            "M34_DOCKING_DECLARATIVE_FRAME_STATE_OWNER_SPLIT_2026-06-02.md",
            "M35_DOCKING_PAINT_DROP_HINTS_OWNER_SPLIT_2026-06-02.md",
            "M36_DOCKING_PAINT_VIEWPORT_SURFACE_OWNER_SPLIT_2026-06-02.md",
            "M37_DOCKING_PAINT_FLOATING_CHROME_OWNER_SPLIT_2026-06-02.md",
            "M38_DOCKING_PAINT_SPLIT_HANDLE_OWNER_SPLIT_2026-06-02.md",
            "M39_DOCKING_PAINT_DROP_OVERLAY_OWNER_SPLIT_2026-06-02.md",
            "M40_DOCKING_PAINT_DRAG_GHOST_OWNER_SPLIT_2026-06-02.md",
            "M41_DOCKING_PAINT_TAB_CHROME_OWNER_SPLIT_2026-06-02.md",
            "python tools/gate_docking_multiwindow_workstream_source.py",
            "python tools/diag_gate_docking_wayland_policy_skip.py",
            "python tools/diag_gate_docking_wayland_policy_skip.py --reuse-built",
            "p3-wayland-policy-skip-local-cold-start",
            "p3-wayland-policy-skip-local-drift-reuse-built",
            "\"role\": \"next\"",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
            "Wayland compositor acceptance runbook as the next true closure path for `DW-P1-linux-003`",
            "local policy-skip evidence is not Wayland compositor acceptance",
            "2026-05-31 local guard refresh",
            "ecosystem/fret-docking/src/runtime/in_window.rs",
            "ecosystem/fret-docking/src/runtime/tear_off.rs",
            "ecosystem/fret-docking/src/runtime/window_created.rs",
            "ecosystem/fret-docking/src/runtime/before_close.rs",
            "ecosystem/fret-docking/src/runtime/auto_close.rs",
            "ecosystem/fret-docking/src/runtime/request.rs",
            "ecosystem/fret-docking/src/runtime/layout_invalidation.rs",
            "ecosystem/fret-docking/src/runtime/tests.rs",
            "ecosystem/fret-docking/src/dock/declarative.rs",
            "ecosystem/fret-docking/src/dock/declarative/frame_state.rs",
            "ecosystem/fret-docking/src/dock/declarative/tab_paint_state.rs",
            "ecosystem/fret-docking/src/dock/paint.rs",
            "ecosystem/fret-docking/src/dock/paint/drag_ghost.rs",
            "ecosystem/fret-docking/src/dock/paint/drop_hints.rs",
            "ecosystem/fret-docking/src/dock/paint/drop_overlay.rs",
            "ecosystem/fret-docking/src/dock/paint/floating_chrome.rs",
            "ecosystem/fret-docking/src/dock/paint/split_handle.rs",
            "ecosystem/fret-docking/src/dock/paint/tab_chrome.rs",
            "ecosystem/fret-docking/src/dock/paint/viewport_surface.rs",
            "`M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md` records DockFloating window-created completion split",
            "`M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md` records DockFloating before-close merge split",
            "`M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md` records DockFloating empty-window auto-close split",
            "`M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md` records DockFloating request-to-new-window split",
            "`M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md` records DockOp post-mutation layout invalidation ownership",
            "`M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md` records docking runtime regression test ownership",
            "`M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md` records docking declarative tab paint-state ownership",
            "`M34_DOCKING_DECLARATIVE_FRAME_STATE_OWNER_SPLIT_2026-06-02.md` records docking declarative frame paint/input-state aggregation ownership",
            "`M35_DOCKING_PAINT_DROP_HINTS_OWNER_SPLIT_2026-06-02.md` records docking drop-hint paint pad/icon ownership",
            "`M36_DOCKING_PAINT_VIEWPORT_SURFACE_OWNER_SPLIT_2026-06-02.md` records docking viewport surface paint ownership",
            "`M37_DOCKING_PAINT_FLOATING_CHROME_OWNER_SPLIT_2026-06-02.md` records DockFloating chrome paint ownership",
            "`M38_DOCKING_PAINT_SPLIT_HANDLE_OWNER_SPLIT_2026-06-02.md` records docking split handle paint ownership",
            "`M39_DOCKING_PAINT_DROP_OVERLAY_OWNER_SPLIT_2026-06-02.md` records docking complex drop overlay paint ownership",
            "`M40_DOCKING_PAINT_DRAG_GHOST_OWNER_SPLIT_2026-06-02.md` records docking drag payload ghost paint ownership",
            "`M41_DOCKING_PAINT_TAB_CHROME_OWNER_SPLIT_2026-06-02.md` records docking tab chrome paint ownership",
            "python -m json.tool docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json",
            "tools/gate_docking_multiwindow_workstream_source.py",
            "tools/diag_gate_docking_wayland_policy_skip.py",
        ],
        forbidden=[
            "python3 tools/check_workstream_catalog.py",
            "python3 .agents/skills/fret_skills.py",
            "python3 -m json.tool docs/workstreams/docking-multiwindow-imgui-parity/WORKSTREAM.json > /dev/null",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md"),
        required=[
            "Latest source-drift guard:",
            "M16_SOURCE_DRIFT_GUARD_2026-05-14.md",
            "Latest local Wayland policy-skip matrix:",
            "M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md",
            "Latest local Wayland guard refresh:",
            "M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity-todo.md"),
        required=[
            "[~] DW-P1-linux-003 Wayland-safe degradation policy for follow-mode.",
            "[ ] Manual Wayland compositor acceptance remains open.",
            "Acceptance (manual; Linux Wayland compositor):",
            "M16_SOURCE_DRIFT_GUARD_2026-05-14.md",
            "M17_LOCAL_WAYLAND_POLICY_SKIP_GATE_2026-05-15.md",
            "M18_LOCAL_WAYLAND_POLICY_SKIP_MATRIX_2026-05-16.md",
            "M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md",
            "M20_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-26.md",
            "M21_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-30.md",
            "M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md",
            "M23_DOCKING_RUNTIME_TEAR_OFF_OWNER_SPLIT_2026-05-31.md",
            "M24_DOCKING_RUNTIME_IN_WINDOW_OWNER_SPLIT_2026-05-31.md",
            "M25_DOCKING_RUNTIME_TEAR_OFF_CREATE_REQUEST_OWNER_SPLIT_2026-06-01.md",
            "M26_DOCKING_RUNTIME_TEAR_OFF_CANCELLATION_OWNER_SPLIT_2026-06-01.md",
            "M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md",
            "M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md",
            "M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md",
            "M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md",
            "M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md",
            "M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md",
            "M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md",
            "source drift guard now validates docking suite membership",
            "Local Wayland policy-skip gate now proves non-Wayland sidecars stop before script execution",
            "Local Wayland policy-skip matrix now covers each Wayland campaign admission predicate",
            "2026-05-26 local Wayland guard refresh reran source/policy/capability/fallback gates",
            "2026-05-30 local Wayland guard refresh reran the same local source/policy/capability/",
            "2026-05-31 local Wayland guard refresh reran the same local source/policy/capability/",
            "Workstream gate commands now expose both the cold-start policy-skip path",
            "Wayland acceptance-open source guard now prevents local policy-skip evidence from being",
            "2026-05-30 local Wayland guard refresh keeps the acceptance boundary current",
            "2026-05-31 local Wayland guard refresh keeps the acceptance boundary current",
            "2026-05-31 docking runtime tear-off owner split keeps the fallback boundary smaller",
            "2026-05-31 docking runtime in-window fallback owner split keeps recovery/fallback",
            "2026-06-02 docking runtime window-created owner split keeps created-window completion out",
            "2026-06-02 docking runtime before-close owner split keeps OS close merge policy out of the runtime shell",
            "2026-06-02 docking runtime auto-close owner split keeps empty DockFloating close effects out of the runtime shell",
            "2026-06-02 docking runtime request owner split keeps tear-off request/fallback policy out of the runtime shell",
            "2026-06-02 docking runtime layout invalidation owner split keeps DockOp post-mutation",
            "2026-06-02 docking runtime test owner split keeps focused regression bodies out of the",
            "2026-06-02 docking declarative tab paint-state owner split keeps tab-hover/menu paint",
        ],
        forbidden=[
            "[x] DW-P1-linux-003 Wayland-safe degradation policy for follow-mode.",
            "[x] Manual Wayland compositor acceptance remains open.",
        ],
        failures=failures,
    )
    _require_markers(
        WAYLAND_POLICY_SKIP_GATE,
        required=[
            "CAMPAIGN_ID = \"imui-p3-wayland-real-host\"",
            "\"capabilities\": [\"diag.script_v2\"]",
            "platform=\"windows\"",
            "linux-wayland-multi-window-mismatch",
            "\"environment.platform_capabilities.ui_multi_window_ne\"",
            "linux-x11-tear-off-mismatch",
            "platform=\"linux\"",
            "\"environment.platform_capabilities.ui_window_tear_off_ne\"",
            "linux-wayland-hover-detection-mismatch",
            "\"environment.platform_capabilities.ui_window_hover_detection_ne\"",
            "linux-wayland-z-level-mismatch",
            "\"environment.platform_capabilities.ui_window_z_level_ne\"",
            "\"availability\": \"launch_time\"",
            "\"environment.requirement_unsatisfied\"",
            "\"environment.platform_capabilities.platform_ne\"",
            "\"existing_filesystem\"",
            "\"script-results\"",
            "\"suite-results\"",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/docking-multiwindow-imgui-parity.md"),
        required=[
            "Latest acceptance-open source guard:",
            "M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md",
            "Latest local Wayland guard refresh:",
            "M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md",
            "Latest docking runtime owner split:",
            "M23_DOCKING_RUNTIME_TEAR_OFF_OWNER_SPLIT_2026-05-31.md",
            "Latest docking runtime fallback owner split:",
            "M24_DOCKING_RUNTIME_IN_WINDOW_OWNER_SPLIT_2026-05-31.md",
            "Latest docking runtime create-request owner split:",
            "M25_DOCKING_RUNTIME_TEAR_OFF_CREATE_REQUEST_OWNER_SPLIT_2026-06-01.md",
            "Latest docking runtime cancellation owner split:",
            "M26_DOCKING_RUNTIME_TEAR_OFF_CANCELLATION_OWNER_SPLIT_2026-06-01.md",
            "Latest docking runtime window-created owner split:",
            "M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md",
            "Latest docking runtime before-close owner split:",
            "M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md",
            "Latest docking runtime auto-close owner split:",
            "M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md",
            "Latest docking runtime request owner split:",
            "M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md",
            "Latest docking runtime layout invalidation owner split:",
            "M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md",
            "Latest docking runtime test owner split:",
            "M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md",
            "Latest docking declarative tab paint-state owner split:",
            "M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M32_DOCKING_RUNTIME_TEST_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/tests.rs",
            "#[cfg(test)] mod tests",
            "request_float_creates_window_and_window_created_moves_panel",
            "window_created_prefers_pending_pointer_id_over_drag_source_window_match",
            "before_close_window_merges_dock_floating_panels_into_target_window",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M33_DOCKING_DECLARATIVE_TAB_PAINT_STATE_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/dock/declarative.rs",
            "ecosystem/fret-docking/src/dock/declarative/tab_paint_state.rs",
            "mod tab_paint_state;",
            "declarative_tab_hover_for_window",
            "apply_declarative_tab_interaction_paint_state",
            "TabChromePaintInput",
            "TabDetailPaintInput",
            "TabOverflowMenuState",
            "cargo nextest run -p fret-docking --no-fail-fast",
            "gate_docking_multiwindow_workstream_source.py",
            "gate_imui_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M34_DOCKING_DECLARATIVE_FRAME_STATE_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/dock/declarative.rs",
            "ecosystem/fret-docking/src/dock/declarative/frame_state.rs",
            "mod frame_state;",
            "prepare_declarative_frame_paint_state",
            "DockSpaceElementFrame",
            "tab width/scroll projection",
            "floating chrome paint inputs",
            "complex drop overlay inputs",
            "cargo nextest run -p fret-docking --no-fail-fast",
            "gate_docking_multiwindow_workstream_source.py",
            "gate_imui_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M35_DOCKING_PAINT_DROP_HINTS_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/dock/paint.rs",
            "ecosystem/fret-docking/src/dock/paint/drop_hints.rs",
            "mod drop_hints;",
            "paint_drop_hints",
            "paint_drop_hint_icon",
            "dock_hint_rects_with_font",
            "DropZone::Center",
            "cargo nextest run -p fret-docking --no-fail-fast",
            "gate_docking_multiwindow_workstream_source.py",
            "gate_imui_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M36_DOCKING_PAINT_VIEWPORT_SURFACE_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/dock/paint.rs",
            "ecosystem/fret-docking/src/dock/paint/viewport_surface.rs",
            "mod viewport_surface;",
            "ViewportSurfacePaintInput",
            "viewport_surface_paint_inputs",
            "paint_viewport_surface_inputs",
            "DockViewportOverlayHooks",
            "SceneOp::ViewportSurface",
            "hooks.paint_with_layout",
            "cargo test -p fret-docking",
            "gate_docking_multiwindow_workstream_source.py",
            "gate_imui_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M37_DOCKING_PAINT_FLOATING_CHROME_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/dock/paint.rs",
            "ecosystem/fret-docking/src/dock/paint/floating_chrome.rs",
            "mod floating_chrome;",
            "FloatingChromePaintInput",
            "paint_floating_chrome_inputs",
            "DOCK_FLOATING_BORDER",
            "SceneOp::SvgMaskIcon",
            "tab_close_glyph",
            "cargo test -p fret-docking",
            "gate_docking_multiwindow_workstream_source.py",
            "gate_imui_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M38_DOCKING_PAINT_SPLIT_HANDLE_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/dock/paint.rs",
            "ecosystem/fret-docking/src/dock/paint/split_handle.rs",
            "mod split_handle;",
            "SplitHandlePaintInput",
            "split_handle_paint_inputs",
            "paint_split_handle_inputs",
            "split_geometry::compute_layout",
            "SplitHandle",
            "cargo test -p fret-docking",
            "gate_docking_multiwindow_workstream_source.py",
            "gate_imui_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M39_DOCKING_PAINT_DROP_OVERLAY_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/dock/paint.rs",
            "ecosystem/fret-docking/src/dock/paint/drop_overlay.rs",
            "mod drop_overlay;",
            "ComplexDropOverlayPaintInput",
            "complex_drop_overlay_paint_inputs",
            "paint_complex_drop_overlay_inputs",
            "paint_tab_insert_preview_title",
            "split_geometry::compute_layout",
            "drop_zone_rect",
            "tab_strip_rect_with_overflow_button",
            "cargo test -p fret-docking",
            "gate_docking_multiwindow_workstream_source.py",
            "gate_imui_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M40_DOCKING_PAINT_DRAG_GHOST_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/dock/paint.rs",
            "ecosystem/fret-docking/src/dock/paint/drag_ghost.rs",
            "mod drag_ghost;",
            "DockDragGhostPaint",
            "paint_drag_payload_ghost",
            "drag_ghost_title_clip_rect",
            "dock_tab_width_for_title",
            "DOCK_TAB_H",
            "fret_core::DrawOrder(10_020)",
            "fret_core::DrawOrder(10_021)",
            "cargo test -p fret-docking",
            "gate_docking_multiwindow_workstream_source.py",
            "gate_imui_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M41_DOCKING_PAINT_TAB_CHROME_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/dock/paint.rs",
            "ecosystem/fret-docking/src/dock/paint/tab_chrome.rs",
            "mod tab_chrome;",
            "TabChromePaintInput",
            "tab_chrome_paint_inputs",
            "paint_tab_chrome_inputs",
            "split_tab_bar",
            "tab_strip_rect_with_overflow_button",
            "TabBarGeometry",
            "fret_core::DrawOrder(3)",
            "cargo test -p fret-docking",
            "gate_docking_multiwindow_workstream_source.py",
            "gate_imui_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M31_DOCKING_RUNTIME_LAYOUT_INVALIDATION_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/layout_invalidation.rs",
            "ecosystem/fret-docking/src/runtime/before_close.rs",
            "ecosystem/fret-docking/src/runtime/window_created.rs",
            "invalidate_after_dock_op",
            "invalidate_windows",
            "DockInvalidationService::bump_windows",
            "clear_viewport_layout_for_window",
            "whole-graph",
            "request_float_creates_window_and_window_created_moves_panel",
            "before_close_window_merges_dock_floating_panels_into_target_window",
            "redock_from_dock_floating_window_auto_closes_empty_os_window",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M30_DOCKING_RUNTIME_REQUEST_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/request.rs",
            "ecosystem/fret-docking/src/runtime/in_window.rs",
            "ecosystem/fret-docking/src/runtime/tear_off.rs",
            "handle_request_float_to_new_window",
            "RequestFloatPanelToNewWindow",
            "RequestFloatTabsToNewWindow",
            "dock_tear_off_supported",
            "default_in_window_float_rect",
            "push_dock_floating_window_create",
            "request_float_creates_window_and_window_created_moves_panel",
            "request_float_degrades_to_in_window_when_window_hover_detection_is_none",
            "request_float_is_idempotent_until_window_created",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M29_DOCKING_RUNTIME_AUTO_CLOSE_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/auto_close.rs",
            "ecosystem/fret-docking/src/runtime/tear_off.rs",
            "collect_empty_dock_floating_windows",
            "close_empty_dock_floating_windows",
            "DockFloating registry scanning",
            "reg.windows()",
            "collect_panels_in_window(window)",
            "WindowRequest::Close(window)",
            "redock_from_dock_floating_window_auto_closes_empty_os_window",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M28_DOCKING_RUNTIME_BEFORE_CLOSE_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/before_close.rs",
            "ecosystem/fret-docking/src/runtime/tear_off.rs",
            "DockFloating registry removal",
            "window_root(closing_window)",
            "first_tabs_in_window(target_window)",
            "DockOp::MergeWindowInto",
            "clear_viewport_layout_for_window(closing_window)",
            "clear_viewport_layout_for_window(target_window)",
            "before_close_window_merges_dock_floating_panels_into_target_window",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M27_DOCKING_RUNTIME_WINDOW_CREATED_OWNER_SPLIT_2026-06-02.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/window_created.rs",
            "complete_for_create_request(request, now)",
            "DockTearOffCompletion::CancelAndCloseWindow",
            "float_panel_to_window",
            "float_tabs_to_window",
            "active drag `source_window`/`current_window`",
            "DockFloating registry registration",
            "cargo check -p fret-docking",
            "request_float_creates_window_and_window_created_moves_panel",
            "window_created_updates_drag_source_window_for_active_dock_drag",
            "window_created_updates_drag_source_window_for_active_dock_tabs_drag",
            "window_created_prefers_pending_pointer_id_over_drag_source_window_match",
            "window_created_does_not_update_drag_source_when_canceled",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "It does not close",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M26_DOCKING_RUNTIME_TEAR_OFF_CANCELLATION_OWNER_SPLIT_2026-06-01.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/tear_off.rs",
            "prune_and_cancel_for_op",
            "cancel_for_tabs_node",
            "request_float_canceled_by_close_panel_closes_created_window",
            "window_created_does_not_update_drag_source_when_canceled",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "does not close `DW-P1-linux-003`",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M25_DOCKING_RUNTIME_TEAR_OFF_CREATE_REQUEST_OWNER_SPLIT_2026-06-01.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/tear_off.rs",
            "dock_tear_off_supported",
            "push_dock_floating_window_create",
            "WindowRequest::Create",
            "CreateWindowKind::DockFloating",
            "request_float_degrades_to_in_window_when_multi_window_is_disabled",
            "request_float_degrades_to_in_window_when_tear_off_is_disabled",
            "request_float_degrades_to_in_window_when_window_hover_detection_is_none",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "does not close `DW-P1-linux-003`",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M24_DOCKING_RUNTIME_IN_WINDOW_OWNER_SPLIT_2026-05-31.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/in_window.rs",
            "ecosystem/fret-docking/src/runtime/tear_off.rs",
            "default_in_window_float_rect",
            "recenter_in_window_floatings",
            "visible-bounds fallback",
            "request_float_degrades_to_in_window_when_multi_window_is_disabled",
            "request_float_degrades_to_in_window_when_tear_off_is_disabled",
            "request_float_degrades_to_in_window_when_window_hover_detection_is_none",
            "cargo check -p fret-demo --bin imui_editor_proof_demo",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "does not close `DW-P1-linux-003`",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M23_DOCKING_RUNTIME_TEAR_OFF_OWNER_SPLIT_2026-05-31.md"),
        required=[
            "Status: local owner split; no Wayland acceptance claim.",
            "`DW-P1-linux-003` open",
            "ecosystem/fret-docking/src/runtime.rs",
            "ecosystem/fret-docking/src/runtime/tear_off.rs",
            "DockFloatingOsWindowRegistry",
            "DockTearOffMachine",
            "pending tear-off correlation",
            "request_float_degrades_to_in_window_when_window_hover_detection_is_none",
            "gate_docking_multiwindow_workstream_source.py",
            "WORKSTREAM.json",
            "git diff --check",
            "does not close `DW-P1-linux-003`",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/runtime.rs"),
        required=[
            "mod auto_close;",
            "mod before_close;",
            "mod in_window;",
            "mod layout_invalidation;",
            "mod request;",
            "mod tear_off;",
            "mod window_created;",
            "pub use in_window::recenter_in_window_floatings;",
            "pub(crate) use tear_off::is_dock_floating_os_window;",
            "#[cfg(test)]",
            "mod tests;",
            "layout_invalidation::invalidate_windows(app, windows);",
            "DockTearOffMachine",
            "prune_and_cancel_for_op",
            "handle_dock_op",
            "request::handle_request_float_to_new_window(app, op)",
            "auto_close::collect_empty_dock_floating_windows(app, dock, tearoff_log)",
            "layout_invalidation::invalidate_after_dock_op(app, dock, &op);",
            "auto_close::close_empty_dock_floating_windows(app, &op, windows_to_auto_close)",
            "handle_dock_window_created",
            "window_created::handle_dock_window_created(app, request, new_window)",
            "handle_dock_before_close_window",
            "before_close::handle_dock_before_close_window(app, closing_window, target_window)",
        ],
        forbidden=[
            "struct DockFloatingOsWindowRegistry",
            "struct DockTearOffMachine",
            "enum DockTearOffKind",
            "enum DockTearOffCompletion",
            "DockTearOffCompletion::CancelAndCloseWindow",
            "complete_for_create_request",
            "drag.source_window = new_window",
            "reg.register(new_window)",
            "fn default_in_window_float_rect",
            "fn clamp_rect_to_bounds",
            "pub fn recenter_in_window_floatings",
            "WindowMetricsService",
            "pending_by_panel: std::collections::HashMap",
            "app.push_effect(Effect::Window(WindowRequest::Create",
            "machine.cancel_for_panel",
            "machine.prune_expired",
            "fret_core::DockNode::Tabs",
            "reg.remove(closing_window)",
            "window_root(closing_window)",
            "first_tabs_in_window(target_window)",
            "source_window: closing_window",
            "clear_viewport_layout_for_window(closing_window)",
            "DockFloatingOsWindowRegistry",
            "reg.windows()",
            "reg.remove(window)",
            "WindowRequest::Close(window)",
            "dock tear-off: auto-close empty DockFloating window",
            "collect_panels_in_window(window).is_empty()",
            "DockPanelDragPayload",
            "DockTabsDragPayload",
            "dock_tear_off_supported",
            "push_dock_floating_window_create",
            "default_in_window_float_rect",
            "DockTearOffKind::Panel",
            "DockTearOffKind::Tabs",
            "DockInvalidationService",
            "clear_viewport_layout_for_window(*source_window)",
            "invalidate_windows(app, [*source_window, *target_window])",
            "use crate::test_host::TestHost",
            "use slotmap::KeyData",
            "#[test]",
            "fn request_float_creates_window_and_window_created_moves_panel",
            "fn window_created_prefers_pending_pointer_id_over_drag_source_window_match",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/runtime/tests.rs"),
        required=[
            "use super::*;",
            "use crate::test_host::TestHost;",
            "use fret_core::{DockNode, DropZone, PanelKey};",
            "use fret_runtime::{CreateWindowKind, Effect, PlatformCapabilities, WindowRequest};",
            "use slotmap::KeyData;",
            "fn request_float_creates_window_and_window_created_moves_panel",
            "fn request_float_degrades_to_in_window_when_multi_window_is_disabled",
            "fn request_float_degrades_to_in_window_when_tear_off_is_disabled",
            "fn request_float_degrades_to_in_window_when_window_hover_detection_is_none",
            "fn request_float_is_idempotent_until_window_created",
            "fn window_created_updates_drag_source_window_for_active_dock_drag",
            "fn window_created_updates_drag_source_window_for_active_dock_tabs_drag",
            "fn window_created_prefers_pending_pointer_id_over_drag_source_window_match",
            "fn redock_from_dock_floating_window_auto_closes_empty_os_window",
            "fn before_close_window_merges_dock_floating_panels_into_target_window",
            "fn request_float_canceled_by_close_panel_closes_created_window",
            "fn window_created_does_not_update_drag_source_when_canceled",
            "handle_dock_op(&mut app",
            "handle_dock_window_created(&mut app",
            "handle_dock_before_close_window(&mut app",
        ],
        forbidden=[
            "pub fn ",
            "mod tests {",
            "DockInvalidationService",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/declarative.rs"),
        required=[
            "mod frame_state;",
            "mod tab_paint_state;",
            "use frame_state::prepare_declarative_frame_paint_state;",
            "use tab_paint_state::{",
            "prepare_declarative_frame_paint_state(",
            "declarative_tab_hover_for_window(cx.app(), window)",
            "apply_declarative_tab_interaction_paint_state(",
        ],
        forbidden=[
            "fn prepare_declarative_frame_paint_state",
            "struct DeclarativeFramePaintState",
            "complex_drop_overlay_paint_inputs",
            "split_handle_paint_inputs",
            "tab_chrome_paint_inputs",
            "tab_detail_paint_inputs",
            "viewport_surface_paint_inputs",
            "fn declarative_tab_hover_for_window",
            "fn apply_declarative_tab_interaction_paint_state",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/declarative/frame_state.rs"),
        required=[
            "pub(super) struct DeclarativeFramePaintState",
            "pub(super) tab_scroll",
            "pub(super) fn into_frame",
            "pub(super) fn prepare_declarative_frame_paint_state",
            "DockSpaceElementFrame::from_snapshot",
            "declarative_tab_widths_for_layout",
            "declarative_tab_scroll_for_frame",
            "declarative_tab_overflow_menu_for_window",
            "declarative_tab_hover_for_window",
            "declarative_floating_hover_for_window",
            "declarative_pressed_floating_close_for_window",
            "floating_chrome_paint_inputs",
            "dock_drag_ghost_for_window",
            "tab_chrome_paint_inputs",
            "tab_detail_paint_inputs",
            "complex_drop_overlay_paint_inputs",
            "split_handle_paint_inputs",
            "viewport_surface_paint_inputs",
        ],
        forbidden=[
            "ManagedSurfaceProps",
            "layout_child",
            "layout_child_root",
            "paint_tab_chrome_inputs",
            "paint_tab_detail_inputs",
            "paint_complex_drop_overlay_inputs",
            "paint_split_handle_inputs",
            "fret_core::Event",
            "PointerEvent",
            "dock_space_element",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/declarative/tab_paint_state.rs"),
        required=[
            "pub(super) fn declarative_tab_hover_for_window",
            "pub(super) fn apply_declarative_tab_interaction_paint_state",
            "DeclarativeDockInteractionService",
            "DeclarativeTabHover",
            "TabChromePaintInput",
            "TabDetailPaintInput",
            "TabOverflowMenuState",
            "service.tab_hover(window)",
            "input.hovered_tab = None;",
            "input.hovered_tab_close = false;",
            "input.hovered_tab_overflow_button = false;",
            "input.tab_overflow_menu = None;",
            "input.tab_overflow_menu = Some(menu);",
        ],
        forbidden=[
            "DockManager",
            "ManagedSurfaceProps",
            "paint_tab_chrome_inputs",
            "paint_tab_detail_inputs",
            "declarative_open_tab_overflow_menu",
            "dock_space_element",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/paint.rs"),
        required=[
            "mod drag_ghost;",
            "mod drop_hints;",
            "mod drop_overlay;",
            "mod floating_chrome;",
            "mod split_handle;",
            "mod tab_chrome;",
            "mod viewport_surface;",
            "pub(super) use drag_ghost::{",
            "DockDragGhostPaint",
            "paint_drag_payload_ghost",
            "pub(super) use drop_hints::paint_drop_hints;",
            "pub(super) use drop_overlay::{",
            "ComplexDropOverlayPaintInput",
            "complex_drop_overlay_paint_inputs",
            "paint_complex_drop_overlay_inputs",
            "paint_tab_insert_preview_title",
            "pub(super) use floating_chrome::{",
            "FloatingChromePaintInput",
            "paint_floating_chrome_inputs",
            "pub(super) use split_handle::{",
            "SplitHandlePaintInput",
            "paint_split_handle_inputs",
            "split_handle_paint_inputs",
            "pub(super) use tab_chrome::{",
            "TabChromePaintInput",
            "paint_tab_chrome_inputs",
            "tab_chrome_paint_inputs",
            "pub(super) use viewport_surface::{",
            "ViewportSurfacePaintInput",
            "paint_viewport_surface_inputs",
            "viewport_surface_paint_inputs",
            "pub(super) fn tab_detail_paint_inputs",
            "pub(super) fn paint_basic_drop_overlay",
            "paint_drop_hints",
        ],
        forbidden=[
            "fn paint_drop_hints(",
            "fn paint_drop_hint_icon(",
            "struct ViewportSurfacePaintInput",
            "fn viewport_surface_paint_input",
            "fn viewport_surface_paint_inputs",
            "fn paint_viewport_surface_input",
            "fn paint_viewport_surface_inputs",
            "struct FloatingChromePaintInput",
            "fn paint_floating_chrome_input",
            "fn paint_floating_chrome_inputs",
            "pub(super) struct SplitHandlePaintInput",
            "pub(super) fn split_handle_paint_inputs",
            "pub(super) fn paint_split_handle_inputs",
            "pub(super) enum ComplexDropOverlayPaintInput",
            "pub(super) fn complex_drop_overlay_paint_inputs",
            "pub(super) fn paint_complex_drop_overlay_inputs",
            "pub(super) fn paint_tab_insert_preview_title",
            "pub(super) struct DockDragGhostPaint",
            "pub(super) fn paint_drag_payload_ghost",
            "split_geometry::compute_layout",
            "dock_tab_width_for_title",
            "pub(super) struct TabChromePaintInput",
            "pub(super) fn tab_chrome_paint_inputs",
            "pub(super) fn paint_tab_chrome_inputs",
            "dock_hint_rects_with_font",
            "ViewportMapping",
            "DockViewportOverlayHooks",
            "DockManager",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/paint/tab_chrome.rs"),
        required=[
            "pub(in crate::dock) struct TabChromePaintInput",
            "pub(in crate::dock) rect: Rect",
            "pub(in crate::dock) tabs_len: usize",
            "pub(in crate::dock) active: usize",
            "pub(in crate::dock) hovered_tab: Option<usize>",
            "pub(in crate::dock) fn tab_chrome_paint_inputs",
            "fn paint_tab_chrome_input",
            "pub(in crate::dock) fn paint_tab_chrome_inputs",
            "split_tab_bar",
            "tab_strip_rect_with_overflow_button",
            "TabBarGeometry",
            "theme.color_token(\"card\")",
            "theme.color_token(\"background\")",
            "theme.color_token(\"accent\")",
            "theme.color_token(\"primary\")",
            "fret_core::DrawOrder(3)",
        ],
        forbidden=[
            "TabDetailPaintInput",
            "ComplexDropOverlayPaintInput",
            "ViewportSurfacePaintInput",
            "FloatingChromePaintInput",
            "SplitHandlePaintInput",
            "DockDragGhostPaint",
            "DockDropHints",
            "DockDropTarget",
            "DockViewportOverlayHooks",
            "DockManager",
            "paint_drop_hints",
            "paint_tab_detail_input",
            "paint_complex_drop_overlay_inputs",
            "paint_viewport_surface_input",
            "paint_floating_chrome_input",
            "paint_split_handle_inputs",
            "paint_drag_payload_ghost",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/paint/drag_ghost.rs"),
        required=[
            "pub(in crate::dock) struct DockDragGhostPaint",
            "pub(in crate::dock) position: Point",
            "pub(in crate::dock) grab_offset: Point",
            "pub(in crate::dock) title: PreparedTabTitle",
            "fn drag_ghost_title_clip_rect",
            "pub(in crate::dock) fn paint_drag_payload_ghost",
            "dock_tab_width_for_title",
            "DOCK_TAB_H",
            "DOCK_TAB_CLOSE_SIZE",
            "DOCK_TAB_CLOSE_GAP",
            "SceneOp::Quad",
            "SceneOp::Text",
            "fret_core::DrawOrder(10_020)",
            "fret_core::DrawOrder(10_021)",
            "theme.color_token(\"card\")",
            "theme.color_token(\"border\")",
            "theme.color_token(\"foreground\")",
        ],
        forbidden=[
            "TabChromePaintInput",
            "TabDetailPaintInput",
            "ComplexDropOverlayPaintInput",
            "ViewportSurfacePaintInput",
            "FloatingChromePaintInput",
            "SplitHandlePaintInput",
            "DockDropHints",
            "DockDropTarget",
            "DockViewportOverlayHooks",
            "DockManager",
            "paint_drop_hints",
            "paint_tab_chrome_input",
            "paint_tab_detail_input",
            "paint_complex_drop_overlay_inputs",
            "paint_viewport_surface_input",
            "paint_floating_chrome_input",
            "paint_split_handle_inputs",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/paint/drop_overlay.rs"),
        required=[
            "pub(in crate::dock) enum ComplexDropOverlayPaintInput",
            "pub(in crate::dock) fn complex_drop_overlay_paint_inputs",
            "pub(in crate::dock) fn paint_complex_drop_overlay_inputs",
            "pub(in crate::dock) fn paint_tab_insert_preview_title",
            "split_geometry::compute_layout",
            "drop_zone_rect",
            "tab_strip_rect_with_overflow_button",
            "dock_tab_width_for_title",
            "ComplexDropOverlayPaintInput::TabInsertMarker",
            "ComplexDropOverlayPaintInput::EdgeZone",
            "theme.color_token(\"primary\")",
            "component.docking.drop_overlay.zone.bg",
            "component.docking.tab_insert.preview.bg",
        ],
        forbidden=[
            "TabChromePaintInput",
            "TabDetailPaintInput",
            "ViewportSurfacePaintInput",
            "FloatingChromePaintInput",
            "SplitHandlePaintInput",
            "DockDragGhostPaint",
            "DockDropHints",
            "DockViewportOverlayHooks",
            "DockManager",
            "paint_drop_hints",
            "paint_tab_chrome_input",
            "paint_tab_detail_input",
            "paint_viewport_surface_input",
            "paint_floating_chrome_input",
            "paint_split_handle_inputs",
            "paint_drag_payload_ghost",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/paint/split_handle.rs"),
        required=[
            "pub(in crate::dock) struct SplitHandlePaintInput",
            "pub(in crate::dock) fn split_handle_paint_inputs",
            "pub(in crate::dock) fn paint_split_handle_inputs",
            "split_geometry::compute_layout",
            "SplitHandle",
            "handle.paint(",
            "theme.color_token(\"ring\")",
            "theme.color_token(\"border\")",
            "paint_device_px: 1.0",
        ],
        forbidden=[
            "TabChromePaintInput",
            "TabDetailPaintInput",
            "ViewportSurfacePaintInput",
            "FloatingChromePaintInput",
            "DockDragGhostPaint",
            "DockDropHints",
            "DockDropTarget",
            "ViewportMapping",
            "DockViewportOverlayHooks",
            "paint_drop_hints",
            "paint_tab_chrome_input",
            "paint_tab_detail_input",
            "paint_viewport_surface_input",
            "paint_floating_chrome_input",
            "paint_complex_drop_overlay_inputs",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/paint/drop_hints.rs"),
        required=[
            "pub(in crate::dock) fn paint_drop_hints",
            "fn paint_drop_hint_icon",
            "dock_hint_rects_with_font",
            "DockDropHints",
            "DockDropTarget::Dock",
            "DropZone::Center",
            "SceneOp::Quad",
            "fret_core::DrawOrder(10_100)",
            "theme.metric_token(\"metric.radius.sm\")",
        ],
        forbidden=[
            "TabChromePaintInput",
            "TabDetailPaintInput",
            "ViewportSurfacePaintInput",
            "FloatingChromePaintInput",
            "SplitHandlePaintInput",
            "DockDragGhostPaint",
            "paint_tab_chrome_input",
            "paint_tab_detail_input",
            "paint_viewport_surface_input",
            "paint_floating_chrome_input",
            "paint_drag_payload_ghost",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/paint/floating_chrome.rs"),
        required=[
            "pub(in crate::dock) struct FloatingChromePaintInput",
            "pub(in crate::dock) outer: Rect",
            "pub(in crate::dock) close_pressed: bool",
            "fn paint_floating_chrome_input",
            "pub(in crate::dock) fn paint_floating_chrome_inputs",
            "DOCK_FLOATING_BORDER",
            "SceneOp::SvgMaskIcon",
            "tab_close_glyph",
            "tab_close_svg",
            "theme.metric_token(\"metric.radius.md\")",
        ],
        forbidden=[
            "TabChromePaintInput",
            "TabDetailPaintInput",
            "ViewportSurfacePaintInput",
            "SplitHandlePaintInput",
            "DockDragGhostPaint",
            "DockDropHints",
            "DockDropTarget",
            "ViewportMapping",
            "DockViewportOverlayHooks",
            "paint_drop_hints",
            "paint_tab_chrome_input",
            "paint_tab_detail_input",
            "paint_viewport_surface_input",
            "paint_split_handle_inputs",
            "paint_complex_drop_overlay_inputs",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/dock/paint/viewport_surface.rs"),
        required=[
            "pub(in crate::dock) struct ViewportSurfacePaintInput",
            "fn viewport_surface_paint_input",
            "pub(in crate::dock) fn viewport_surface_paint_inputs",
            "fn paint_viewport_surface_input",
            "pub(in crate::dock) fn paint_viewport_surface_inputs",
            "DockViewportLayout",
            "DockViewportOverlayHooks",
            "ViewportMapping",
            "SceneOp::ViewportSurface",
            "hooks.paint_with_layout",
            "split_tab_bar",
        ],
        forbidden=[
            "TabChromePaintInput",
            "TabDetailPaintInput",
            "FloatingChromePaintInput",
            "SplitHandlePaintInput",
            "DockDragGhostPaint",
            "DockDropHints",
            "DockDropTarget",
            "paint_drop_hints",
            "paint_tab_chrome_input",
            "paint_tab_detail_input",
            "paint_floating_chrome_input",
            "paint_split_handle_inputs",
            "paint_complex_drop_overlay_inputs",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/runtime/layout_invalidation.rs"),
        required=[
            "pub(super) fn invalidate_windows",
            "DockInvalidationService::bump_windows(app, windows)",
            "pub(super) fn invalidate_after_dock_op",
            "DockOp::MovePanel",
            "DockOp::MovePanelToEmptyDockSpace",
            "DockOp::MoveTabs",
            "DockOp::MoveTabsToEmptyDockSpace",
            "DockOp::FloatPanelToWindow",
            "DockOp::FloatPanelInWindow",
            "DockOp::FloatTabsInWindow",
            "DockOp::SetFloatingRect",
            "DockOp::SetActiveTab",
            "DockOp::RequestFloatPanelToNewWindow",
            "clear_viewport_layout_for_window",
            "invalidate_windows(app, [*source_window, *target_window])",
            "invalidate_windows(app, dock.graph.windows())",
        ],
        forbidden=[
            "pub fn ",
            "handle_dock_op",
            "WindowRequest::Create",
            "DockTearOffMachine",
            "DockFloatingOsWindowRegistry",
            "default_in_window_float_rect",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/runtime/auto_close.rs"),
        required=[
            "pub(super) fn collect_empty_dock_floating_windows",
            "pub(super) fn close_empty_dock_floating_windows",
            "DockFloatingOsWindowRegistry",
            "let Some(reg) = app.global::<DockFloatingOsWindowRegistry>()",
            "reg.windows()",
            "collect_panels_in_window(window)",
            "windows.push(window)",
            "FRET_DOCK_TEAROFF_LOG",
            "dock tear-off: scan dock-floating window panels",
            "dock tear-off: auto-close empty DockFloating window",
            "reg.remove(window)",
            "WindowRequest::Close(window)",
        ],
        forbidden=[
            "pub fn ",
            "handle_dock_op",
            "handle_dock_window_created",
            "handle_dock_before_close_window",
            "DockOp::MergeWindowInto",
            "complete_for_create_request",
            "push_dock_floating_window_create",
            "default_in_window_float_rect",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/runtime/request.rs"),
        required=[
            "pub(super) fn handle_request_float_to_new_window",
            "DockOp::RequestFloatPanelToNewWindow",
            "DockOp::RequestFloatTabsToNewWindow",
            "dock_tear_off_supported",
            "PlatformCapabilities",
            "default_in_window_float_rect",
            "DockOp::FloatPanelInWindow",
            "DockOp::FloatTabsInWindow",
            "super::handle_dock_op",
            "DockTearOffMachine::default",
            "register_request",
            "DRAG_KIND_DOCK_PANEL",
            "DockPanelDragPayload",
            "DRAG_KIND_DOCK_TABS",
            "DockTabsDragPayload",
            "DockTearOffKind::Panel",
            "DockTearOffKind::Tabs",
            "push_dock_floating_window_create",
        ],
        forbidden=[
            "pub fn ",
            "WindowRequest::Close",
            "handle_dock_window_created",
            "handle_dock_before_close_window",
            "collect_empty_dock_floating_windows",
            "complete_for_create_request",
            "reg.remove",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/runtime/in_window.rs"),
        required=[
            "pub fn recenter_in_window_floatings<H: UiHost>",
            "pub(super) fn default_in_window_float_rect<H: UiHost>",
            "fn visible_bounds<H: UiHost>",
            "fn clamp_rect_to_bounds(rect: Rect, bounds: Rect) -> Rect",
            "WindowMetricsService",
            "Size::new(Px(480.0), Px(360.0))",
            "Size::new(Px(960.0), Px(720.0))",
            "super::request_dock_invalidation(app, [window]);",
            "floating.rect = clamp_rect_to_bounds",
        ],
        forbidden=[
            "DockOp::",
            "WindowRequest::Create",
            "DockTearOffMachine",
            "DockFloatingOsWindowRegistry",
            "CreateWindowKind",
            "handle_dock_op",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/runtime/window_created.rs"),
        required=[
            "pub(super) fn handle_dock_window_created",
            "DockTearOffMachine::default",
            "complete_for_create_request(request, now)",
            "DockTearOffCompletion::CancelAndCloseWindow",
            "WindowRequest::Close(new_window)",
            "CreateWindowKind::DockFloating",
            "DockTearOffKind::Panel",
            "DockTearOffKind::Tabs",
            "float_panel_to_window",
            "float_tabs_to_window",
            "DRAG_KIND_DOCK_PANEL",
            "DRAG_KIND_DOCK_TABS",
            "drag.source_window = new_window",
            "reg.register(new_window)",
            "invalidate_windows(app, [*source_window, new_window]);",
        ],
        forbidden=[
            "pub fn ",
            "handle_dock_op",
            "RequestFloatPanelToNewWindow",
            "push_dock_floating_window_create",
            "default_in_window_float_rect",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/runtime/before_close.rs"),
        required=[
            "pub(super) fn handle_dock_before_close_window",
            "DockFloatingOsWindowRegistry::default",
            "reg.remove(closing_window)",
            "DockManager::default",
            "window_root(closing_window)",
            "first_tabs_in_window(target_window)",
            "DockOp::MergeWindowInto",
            "source_window: closing_window",
            "target_window,",
            "target_tabs,",
            "clear_viewport_layout_for_window(closing_window)",
            "clear_viewport_layout_for_window(target_window)",
            "invalidate_windows(app, [target_window]);",
        ],
        forbidden=[
            "pub fn ",
            "handle_dock_op",
            "handle_dock_window_created",
            "WindowRequest::Close",
            "complete_for_create_request",
            "push_dock_floating_window_create",
            "default_in_window_float_rect",
        ],
        failures=failures,
    )
    _require_markers(
        Path("ecosystem/fret-docking/src/runtime/tear_off.rs"),
        required=[
            "pub(super) struct DockFloatingOsWindowRegistry",
            "pub(super) fn register(&mut self, window: AppWindowId)",
            "pub(super) fn remove(&mut self, window: AppWindowId)",
            "pub(super) fn windows(&self) -> impl Iterator<Item = AppWindowId> + '_",
            "pub(crate) fn is_dock_floating_os_window",
            "pub(super) struct DockTearOffMachine",
            "pending_by_panel: HashMap<PanelKey, DockTearOffPending>",
            "pub(super) fn register_request(",
            "fn cancel_for_panel",
            "pub(super) fn complete_for_create_request(",
            "DockTearOffCompletion::CancelAndCloseWindow",
            "pub(super) fn dock_tear_off_supported",
            "pub(super) fn push_dock_floating_window_create",
            "pub(super) fn prune_and_cancel_for_op",
            "fn cancel_for_tabs_node",
            "DockOp::ClosePanel",
            "DockOp::MovePanel",
            "DockOp::FloatPanelInWindow",
            "DockOp::MoveTabs",
            "DockOp::MoveTabsToEmptyDockSpace",
            "DockOp::FloatTabsInWindow",
            "DockNode::Tabs",
            "WindowRequest::Create(CreateWindowRequest",
            "CreateWindowKind::DockFloating",
            "WindowRole::Auxiliary",
            "fret_window_style_profiles::tool_window_profile_v1",
            "caps.ui.multi_window",
            "caps.ui.window_tear_off",
        ],
        forbidden=[
            "handle_dock_op",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M22_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-31.md"),
        required=[
            "Status: local guard refresh; no Wayland acceptance claim.",
            "2026-05-31",
            "`DW-P1-linux-003` remains `[~]`",
            "Manual Wayland compositor acceptance",
            "remains open",
            "python -m py_compile tools\\gate_docking_multiwindow_workstream_source.py tools\\diag_gate_docking_wayland_policy_skip.py",
            "python tools\\gate_docking_multiwindow_workstream_source.py",
            "python tools\\diag_gate_docking_wayland_policy_skip.py --reuse-built",
            "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-wayland-real-host.json --json",
            "cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast",
            "cargo nextest run -p fret-docking --lib request_float_degrades_to_in_window_when_window_hover_detection_is_none --no-fail-fast",
            "python tools\\gate_imui_workstream_source.py",
            "python -m json.tool docs\\workstreams\\docking-multiwindow-imgui-parity\\WORKSTREAM.json",
            "python tools\\check_workstream_catalog.py",
            "git diff --check",
            "five policy-skip cases",
            "windows-platform-mismatch",
            "linux-wayland-multi-window-mismatch",
            "linux-x11-tear-off-mismatch",
            "linux-wayland-hover-detection-mismatch",
            "linux-wayland-z-level-mismatch",
            "2 tests passed, 94 skipped",
            "1 test passed, 86 skipped",
            "510 dedicated directories and 47 standalone markdown files",
            "does not close `DW-P1-linux-003`",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M21_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-30.md"),
        required=[
            "Status: local guard refresh; no Wayland acceptance claim.",
            "2026-05-30",
            "`DW-P1-linux-003` remains `[~]`",
            "Manual Wayland compositor acceptance",
            "remains open",
            "python -m py_compile tools\\gate_docking_multiwindow_workstream_source.py tools\\diag_gate_docking_wayland_policy_skip.py",
            "python tools\\gate_docking_multiwindow_workstream_source.py",
            "python tools\\diag_gate_docking_wayland_policy_skip.py --reuse-built",
            "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-wayland-real-host.json --json",
            "cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast",
            "cargo nextest run -p fret-docking --lib request_float_degrades_to_in_window_when_window_hover_detection_is_none --no-fail-fast",
            "python tools\\gate_imui_workstream_source.py",
            "python -m json.tool docs\\workstreams\\docking-multiwindow-imgui-parity\\WORKSTREAM.json",
            "python tools\\check_workstream_catalog.py",
            "git diff --check",
            "five policy-skip cases",
            "windows-platform-mismatch",
            "linux-wayland-multi-window-mismatch",
            "linux-x11-tear-off-mismatch",
            "linux-wayland-hover-detection-mismatch",
            "linux-wayland-z-level-mismatch",
            "does not close `DW-P1-linux-003`",
            "M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M20_LOCAL_WAYLAND_GUARD_REFRESH_2026-05-26.md"),
        required=[
            "Status: local guard refresh; no Wayland acceptance claim.",
            "`DW-P1-linux-003` remains `[~]`",
            "Manual Wayland compositor acceptance",
            "remains open",
            "python tools\\gate_docking_multiwindow_workstream_source.py",
            "python tools\\gate_imui_workstream_source.py",
            "cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast",
            "python tools\\diag_gate_docking_wayland_policy_skip.py",
            "cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-wayland-real-host.json --json",
            "cargo nextest run -p fret-docking --lib request_float_degrades_to_in_window_when_window_hover_detection_is_none --no-fail-fast",
            "does not close `DW-P1-linux-003`",
            "The next true closure event remains a dated real Linux",
            "Wayland compositor acceptance note produced from",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )
    _require_markers(
        Path("docs/workstreams/docking-multiwindow-imgui-parity/M19_WAYLAND_ACCEPTANCE_OPEN_GUARD_2026-05-17.md"),
        required=[
            "Status: source guard refresh; no Wayland acceptance claim.",
            "`DW-P1-linux-003` remains `[~]`",
            "\"Manual Wayland compositor acceptance remains open\" to remain unchecked",
            "`WORKSTREAM.json` to keep the M5 runbook as the `role: next` closure path",
            "does not close `DW-P1-linux-003`",
            "python tools/gate_docking_multiwindow_workstream_source.py",
            "python tools/gate_imui_workstream_source.py",
        ],
        forbidden=[
            "Status: accepted",
            "Status: closed",
            "closes `DW-P1-linux-003`",
        ],
        failures=failures,
    )


def _check_suites(failures: list[str]) -> None:
    common = _require_suite_members(
        COMMON_SUITE,
        required=REQUIRED_COMMON_SCRIPTS,
        failures=failures,
    )
    windows = _require_suite_members(
        WINDOWS_SUITE,
        required=REQUIRED_WINDOWS_SCRIPTS,
        failures=failures,
    )
    root = _require_suite_members(
        ROOT_SUITE,
        required=REQUIRED_COMMON_SCRIPTS + REQUIRED_WINDOWS_SCRIPTS,
        failures=failures,
    )
    _require_suite_members(
        SMOKE_SUITE,
        required=REQUIRED_SMOKE_SCRIPTS,
        failures=failures,
        max_len=12,
    )
    if root and common and windows and root != common + windows:
        failures.append(
            f"{ROOT_SUITE.as_posix()}: expected root scripts to equal common + windows suite scripts"
        )
    _require_script_paths(
        REQUIRED_COMMON_SCRIPTS
        + REQUIRED_WINDOWS_SCRIPTS
        + REQUIRED_SMOKE_SCRIPTS
        + EXISTENCE_ONLY_SCRIPTS,
        failures,
    )


def _step_predicate(step: Any) -> dict[str, Any]:
    if not isinstance(step, dict):
        return {}
    predicate = step.get("predicate")
    return predicate if isinstance(predicate, dict) else {}


def _check_wayland_admission(failures: list[str]) -> None:
    campaign = _read_json(WAYLAND_CAMPAIGN, failures)
    if not isinstance(campaign, dict):
        return
    if campaign.get("kind") != "diag_campaign_manifest":
        failures.append(f"{WAYLAND_CAMPAIGN.as_posix()}: expected diag_campaign_manifest")
    if campaign.get("id") != "imui-p3-wayland-real-host":
        failures.append(f"{WAYLAND_CAMPAIGN.as_posix()}: expected id imui-p3-wayland-real-host")
    if campaign.get("tier") != "manual":
        failures.append(f"{WAYLAND_CAMPAIGN.as_posix()}: expected manual tier")
    if campaign.get("expected_duration_ms") != 180000:
        failures.append(f"{WAYLAND_CAMPAIGN.as_posix()}: expected 180000ms duration")

    items = campaign.get("items")
    if not isinstance(items, list) or not any(
        isinstance(item, dict)
        and item.get("kind") == "script"
        and item.get("value") == WAYLAND_SCRIPT.as_posix()
        for item in items
    ):
        failures.append(f"{WAYLAND_CAMPAIGN.as_posix()}: missing canonical Wayland script item")

    requires_environment = campaign.get("requires_environment")
    if not isinstance(requires_environment, list) or len(requires_environment) != 1:
        failures.append(f"{WAYLAND_CAMPAIGN.as_posix()}: expected one environment admission rule")
        return

    requirement = requires_environment[0]
    if not isinstance(requirement, dict):
        failures.append(f"{WAYLAND_CAMPAIGN.as_posix()}: environment rule must be an object")
        return
    if requirement.get("source_id") != "platform.capabilities":
        failures.append(f"{WAYLAND_CAMPAIGN.as_posix()}: expected platform.capabilities source")

    predicate = requirement.get("predicate")
    if not isinstance(predicate, dict):
        failures.append(f"{WAYLAND_CAMPAIGN.as_posix()}: missing platform predicate")
        return
    expected_predicate = {
        "kind": "platform_capabilities",
        "platform_is": "linux",
        "ui_multi_window_is": True,
        "ui_window_tear_off_is": False,
        "ui_window_hover_detection_is": "none",
        "ui_window_z_level_is": "none",
    }
    for key, expected in expected_predicate.items():
        if predicate.get(key) != expected:
            failures.append(
                f"{WAYLAND_CAMPAIGN.as_posix()}: expected predicate {key}={expected!r}"
            )

    script = _read_json(WAYLAND_SCRIPT, failures)
    if not isinstance(script, dict):
        return
    if script.get("schema_version") != 2:
        failures.append(f"{WAYLAND_SCRIPT.as_posix()}: expected schema_version=2")
    meta = script.get("meta")
    env_defaults = meta.get("env_defaults") if isinstance(meta, dict) else None
    if (
        not isinstance(env_defaults, dict)
        or env_defaults.get("FRET_DOCK_ALLOW_MULTI_WINDOW_TEAR_OFF") != "1"
    ):
        failures.append(
            f"{WAYLAND_SCRIPT.as_posix()}: expected FRET_DOCK_ALLOW_MULTI_WINDOW_TEAR_OFF=1"
        )

    steps = script.get("steps")
    if not isinstance(steps, list):
        failures.append(f"{WAYLAND_SCRIPT.as_posix()}: expected steps list")
        return

    if not any(
        step.get("type") == "wait_until"
        and _step_predicate(step).get("kind") == "platform_ui_window_hover_detection_is"
        and _step_predicate(step).get("quality") == "none"
        for step in steps
        if isinstance(step, dict)
    ):
        failures.append(f"{WAYLAND_SCRIPT.as_posix()}: missing hover-detection-none wait")

    if not any(
        step.get("type") == "drag_pointer"
        and isinstance(step.get("target"), dict)
        and step["target"].get("id") == "dock-arb-tab-drag-anchor-right"
        and isinstance(step.get("delta_x"), (int, float))
        and step["delta_x"] >= 2000.0
        for step in steps
        if isinstance(step, dict)
    ):
        failures.append(f"{WAYLAND_SCRIPT.as_posix()}: missing long tear-off drag gesture")

    if not any(
        step.get("type") == "assert"
        and _step_predicate(step).get("kind") == "known_window_count_is"
        and _step_predicate(step).get("n") == 1
        for step in steps
        if isinstance(step, dict)
    ):
        failures.append(f"{WAYLAND_SCRIPT.as_posix()}: missing one-window fallback assertion")

    if not any(
        step.get("type") == "capture_bundle"
        and step.get("label") == "docking-arbitration-demo-wayland-degrade-no-os-tearoff"
        for step in steps
        if isinstance(step, dict)
    ):
        failures.append(f"{WAYLAND_SCRIPT.as_posix()}: missing canonical evidence bundle")


def collect_failures() -> list[str]:
    failures: list[str] = []
    _check_docs(failures)
    _check_suites(failures)
    _check_wayland_admission(failures)
    return failures


def main() -> None:
    failures = collect_failures()
    if failures:
        fail(GATE_NAME, f"{len(failures)} source marker problem(s):\n  - " + "\n  - ".join(failures))
    ok(GATE_NAME)


if __name__ == "__main__":
    main()
