from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail


GATE_NAME = "fret-launch runner scheduling source"

FIRST_FRAME_SMOKE_DEMO = WORKSPACE_ROOT / "apps/fret-examples/src/first_frame_smoke_demo.rs"
FRET_LAUNCH_DESKTOP_APP_HANDLER = (
    WORKSPACE_ROOT / "crates/fret-launch/src/runner/desktop/runner/app_handler.rs"
)
FRET_LAUNCH_DESKTOP_WINDOW_LIFECYCLE = (
    WORKSPACE_ROOT / "crates/fret-launch/src/runner/desktop/runner/window_lifecycle.rs"
)
FRET_LAUNCH_RUNNER_SCHEDULING_WORKSTREAM = (
    WORKSPACE_ROOT
    / "docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/WORKSTREAM.json"
)
FRET_LAUNCH_RUNNER_SCHEDULING_EVIDENCE = (
    WORKSPACE_ROOT
    / "docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/EVIDENCE_AND_GATES.md"
)
FRET_LAUNCH_RUNNER_SCHEDULING_FIRST_FRAME_NOTE = (
    WORKSPACE_ROOT
    / "docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1/M6_FIRST_FRAME_BOOTSTRAP_CLOSURE_2026-04-26.md"
)


@dataclass(frozen=True)
class Failure:
    path: Path
    message: str


def rel_path(path: Path) -> Path:
    return path.resolve().relative_to(WORKSPACE_ROOT)


def normalize(text: str) -> str:
    return "".join(text.split())


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {rel_path(path).as_posix()}: {exc}")


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


def check_required_forbidden_markers(
    failures: list[Failure],
    path: Path,
    source: str,
    *,
    required: list[str],
    forbidden: list[str],
) -> None:
    normalized = normalize(source)
    for marker in required:
        if normalize(marker) not in normalized:
            failures.append(Failure(rel_path(path), f"missing source marker: {marker}"))
    for marker in forbidden:
        if normalize(marker) in normalized:
            failures.append(Failure(rel_path(path), f"forbidden source marker: {marker}"))


def check_workstream_state(failures: list[Failure]) -> None:
    text = read_text(FRET_LAUNCH_RUNNER_SCHEDULING_WORKSTREAM)
    try:
        workstream = json.loads(text)
    except json.JSONDecodeError as exc:
        failures.append(
            Failure(
                rel_path(FRET_LAUNCH_RUNNER_SCHEDULING_WORKSTREAM),
                f"invalid workstream JSON: {exc}",
            )
        )
        return

    if workstream.get("status") != "maintenance":
        failures.append(
            Failure(
                rel_path(FRET_LAUNCH_RUNNER_SCHEDULING_WORKSTREAM),
                "workstream status should remain maintenance",
            )
        )
    if workstream.get("scope_kind") != "execution":
        failures.append(
            Failure(
                rel_path(FRET_LAUNCH_RUNNER_SCHEDULING_WORKSTREAM),
                "workstream scope_kind should remain execution",
            )
        )
    if "pointer movement or hover" not in workstream.get("problem", ""):
        failures.append(
            Failure(
                rel_path(FRET_LAUNCH_RUNNER_SCHEDULING_WORKSTREAM),
                "workstream problem should name the blank-until-hover invariant",
            )
        )


def check_first_frame_bootstrap_sources(failures: list[Failure]) -> None:
    smoke = read_text(FIRST_FRAME_SMOKE_DEMO)
    check_required_forbidden_markers(
        failures,
        FIRST_FRAME_SMOKE_DEMO,
        smoke,
        required=[
            "scene.push(SceneOp::Quad {",
            "state.frames_drawn = state.frames_drawn.saturating_add(1);",
            "if state.frames_drawn < 3 {",
            "app.push_effect(Effect::RequestAnimationFrame(window));",
            "app.push_effect(Effect::Window(WindowRequest::Close(window)));",
        ],
        forbidden=[],
    )

    window_lifecycle = read_text(FRET_LAUNCH_DESKTOP_WINDOW_LIFECYCLE)
    insert_window = source_slice(
        FRET_LAUNCH_DESKTOP_WINDOW_LIFECYCLE,
        window_lifecycle,
        "pub(super) fn insert_window(",
        "pub(super) fn close_window",
    )
    check_required_forbidden_markers(
        failures,
        FRET_LAUNCH_DESKTOP_WINDOW_LIFECYCLE,
        insert_window,
        required=[
            "self.window_registry.insert(winit_id, id);",
            "self.request_window_redraw_with_reason(",
            "fret_runtime::RunnerFrameDriveReason::SurfaceBootstrap,",
            "self.raf_windows.request(id);",
            "window may appear blank until another event arrives",
        ],
        forbidden=[],
    )

    app_handler = read_text(FRET_LAUNCH_DESKTOP_APP_HANDLER)
    deferred_surface = source_slice(
        FRET_LAUNCH_DESKTOP_APP_HANDLER,
        app_handler,
        "fn try_create_missing_surfaces(&mut self) {",
        "impl<D: WinitAppDriver> ApplicationHandler for WinitRunner<D> {",
    )
    check_required_forbidden_markers(
        failures,
        FRET_LAUNCH_DESKTOP_APP_HANDLER,
        deferred_surface,
        required=[
            "let mut redraw_bootstrap_windows: Vec<fret_core::AppWindowId> = Vec::new();",
            "redraw_bootstrap_windows.push(app_window);",
            "self.request_window_redraw_with_reason(",
            "fret_runtime::RunnerFrameDriveReason::SurfaceBootstrap,",
            "self.raf_windows.request(app_window);",
            "deferred surface creation also gets a one-shot RAF",
        ],
        forbidden=[
            "state.window.request_redraw();",
            "self.record_frame_drive_reason( app_window,",
        ],
    )

    about_to_wait = source_slice(
        FRET_LAUNCH_DESKTOP_APP_HANDLER,
        app_handler,
        "fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {",
        "fn resumed(&mut self, event_loop: &dyn ActiveEventLoop) {",
    )
    check_required_forbidden_markers(
        failures,
        FRET_LAUNCH_DESKTOP_APP_HANDLER,
        about_to_wait,
        required=[
            "next_raf_deadline.get_or_insert_with(|| now + self.config.frame_interval);",
            "let flushed_raf_this_turn = raf_deadline.is_some_and(|deadline| now >= deadline);",
            "self.flush_raf_redraw_requests();",
            "if wants_poll || flushed_raf_this_turn {",
            "event_loop.set_control_flow(ControlFlow::Poll);",
        ],
        forbidden=[],
    )


def check_workstream_docs(failures: list[Failure]) -> None:
    docs_text = "\n".join(
        [
            read_text(FRET_LAUNCH_RUNNER_SCHEDULING_EVIDENCE),
            read_text(FRET_LAUNCH_RUNNER_SCHEDULING_FIRST_FRAME_NOTE),
        ]
    )
    for marker in [
        "first_frame_smoke_demo",
        "SurfaceBootstrap",
        "request_window_redraw_with_reason",
        "one-shot RAF",
        "blank-start reports",
        "python tools/gate_fret_launch_runner_scheduling_source.py",
    ]:
        if marker not in docs_text:
            failures.append(
                Failure(
                    Path("docs/workstreams/fret-launch-runner-scheduling-fearless-refactor-v1"),
                    f"first-frame workstream docs should name marker: {marker}",
                )
            )


def print_failures(failures: list[Failure]) -> None:
    print(f"[gate] {GATE_NAME}")
    if not failures:
        print("[gate] ok")
        return

    print(f"[gate] FAIL: {len(failures)} source policy problem(s)")
    for failure in failures[:60]:
        print(f"  - {failure.path.as_posix()}: {failure.message}")
    if len(failures) > 60:
        print(f"  ... and {len(failures) - 60} more")


def main() -> None:
    failures: list[Failure] = []
    check_first_frame_bootstrap_sources(failures)
    check_workstream_state(failures)
    check_workstream_docs(failures)

    print_failures(failures)
    if failures:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
