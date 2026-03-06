from __future__ import annotations

import re
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "WinitAppDriver example hook coverage"
EXAMPLES_DIR = WORKSPACE_ROOT / "apps/fret-examples/src"
ALLOWED_METHODS = {
    "init",
    "gpu_ready",
    "hot_reload_global",
    "hot_reload_window",
    "gpu_frame_prepare",
    "record_engine_frame",
    "viewport_input",
    "dock_op",
    "handle_command",
    "handle_global_command",
    "handle_model_changes",
    "handle_global_changes",
    "create_window_state",
    "handle_event",
    "render",
    "window_create_spec",
    "window_created",
    "before_close_window",
    "semantics_snapshot",
    "accessibility_focus",
    "accessibility_invoke",
    "accessibility_set_value_text",
    "accessibility_set_value_numeric",
    "accessibility_decrement",
    "accessibility_increment",
    "accessibility_scroll_by",
    "accessibility_set_text_selection",
    "accessibility_replace_selected_text",
}
IMPL_HEADER_RE = re.compile(r"impl\s+WinitAppDriver\s+for\s+[A-Za-z_][A-Za-z0-9_]*\s*\{")
METHOD_RE = re.compile(r"(?m)^\s{4}fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(")


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.relative_to(WORKSPACE_ROOT)}: {exc}")


def iter_impl_bodies(text: str) -> list[str]:
    bodies: list[str] = []
    for match in IMPL_HEADER_RE.finditer(text):
        body_start = match.end()
        depth = 1
        index = body_start
        state = "code"
        raw_hashes = 0

        while index < len(text):
            char = text[index]
            next_char = text[index + 1] if index + 1 < len(text) else ""

            if state == "code":
                if char == "/" and next_char == "/":
                    state = "line_comment"
                    index += 2
                    continue
                if char == "/" and next_char == "*":
                    state = "block_comment"
                    index += 2
                    continue
                if char == '"':
                    state = "string"
                    index += 1
                    continue
                if char == "r":
                    probe = index + 1
                    while probe < len(text) and text[probe] == "#":
                        probe += 1
                    if probe < len(text) and text[probe] == '"':
                        state = "raw_string"
                        raw_hashes = probe - (index + 1)
                        index = probe + 1
                        continue
                if char == "{":
                    depth += 1
                elif char == "}":
                    depth -= 1
                    if depth == 0:
                        bodies.append(text[body_start:index])
                        break
                index += 1
                continue

            if state == "line_comment":
                if char == "\n":
                    state = "code"
                index += 1
                continue

            if state == "block_comment":
                if char == "*" and next_char == "/":
                    state = "code"
                    index += 2
                    continue
                index += 1
                continue

            if state == "string":
                if char == "\\":
                    index += 2
                    continue
                if char == '"':
                    state = "code"
                index += 1
                continue

            if state == "char":
                if char == "\\":
                    index += 2
                    continue
                if char == "'":
                    state = "code"
                index += 1
                continue

            if state == "raw_string":
                if char == '"' and text[index + 1 : index + 1 + raw_hashes] == "#" * raw_hashes:
                    state = "code"
                    index += 1 + raw_hashes
                    continue
                index += 1
                continue
        else:
            fail(GATE_NAME, "failed to parse a `WinitAppDriver` impl block in examples")

    return bodies


def main() -> None:
    problems: list[str] = []

    for path in sorted(EXAMPLES_DIR.glob("*.rs")):
        text = read_text(path)
        if "fn build_driver(" not in text:
            continue
        bodies = iter_impl_bodies(text)
        if not bodies:
            continue

        for body in bodies:
            methods = set(METHOD_RE.findall(body))
            unsupported = sorted(methods - ALLOWED_METHODS)
            if unsupported:
                problems.append(
                    f"{path.relative_to(WORKSPACE_ROOT)} uses direct `WinitAppDriver` hook(s) not covered by `FnDriver`: {', '.join(unsupported)}"
                )

    if problems:
        print(f"[gate] {GATE_NAME}")
        print(f"[gate] FAIL: {len(problems)} issue(s)")
        for problem in problems:
            print(f"  - {problem}")
        raise SystemExit(1)

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
