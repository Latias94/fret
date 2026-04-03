from __future__ import annotations

import re
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "window input context command-availability usage"

ROOTS = [
    WORKSPACE_ROOT / "crates",
    WORKSPACE_ROOT / "ecosystem",
    WORKSPACE_ROOT / "apps",
]

RAW_WINDOW_INPUT_CONTEXT_READ = re.compile(
    r"\.global::<(?:fret_runtime::)?WindowInputContextService>\(\)"
)

COMMAND_AVAILABILITY_FIELD_PATTERNS = [
    re.compile(r"\bedit_can_undo\b"),
    re.compile(r"\bedit_can_redo\b"),
    re.compile(r"\brouter_can_back\b"),
    re.compile(r"\brouter_can_forward\b"),
    re.compile(r'"edit\.can_[^"]*"'),
    re.compile(r'"router\.can_[^"]*"'),
]

COMMAND_SHORTCUT_CONSUMER_PATTERNS = [
    re.compile(r"display_shortcut_for_command_sequence(?:_with_key_contexts)?\("),
    re.compile(r"\.is_enabled_for_command\("),
    re.compile(r"\.is_enabled_for_meta\("),
]

APPROVED_WINDOW_INPUT_HELPER_PATTERNS = [
    re.compile(r"\bbest_effort_input_context_for_window(?:_with_fallback)?\("),
    re.compile(r"\bsnapshot_for_window_with_input_ctx_fallback\("),
    re.compile(r"\bbest_effort_snapshot_for_window_with_input_ctx_fallback\("),
]

DIRECT_FIELD_ALLOWLIST = {
    Path("crates/fret-runtime/src/window_input_context.rs"),
    Path("crates/fret-ui/src/tree/commands.rs"),
}


def iter_rust_sources() -> list[Path]:
    files: list[Path] = []
    for root in ROOTS:
        files.extend(path for path in root.rglob("*.rs") if path.is_file())
    return files


def main() -> None:
    problems: list[str] = []

    for path in iter_rust_sources():
        rel = path.relative_to(WORKSPACE_ROOT)
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            fail(GATE_NAME, f"failed to read {rel}: {exc}")

        if RAW_WINDOW_INPUT_CONTEXT_READ.search(text) is None:
            continue

        direct_field_hits = [
            pattern.pattern
            for pattern in COMMAND_AVAILABILITY_FIELD_PATTERNS
            if pattern.search(text) is not None
        ]
        shortcut_consumer_hits = [
            pattern.pattern
            for pattern in COMMAND_SHORTCUT_CONSUMER_PATTERNS
            if pattern.search(text) is not None
        ]
        has_approved_helper = any(
            pattern.search(text) is not None
            for pattern in APPROVED_WINDOW_INPUT_HELPER_PATTERNS
        )

        if direct_field_hits and rel not in DIRECT_FIELD_ALLOWLIST:
            problems.append(
                f"{rel}: raw WindowInputContextService reads must not own "
                f"edit/router command-availability truth; use runtime helpers instead "
                f"(matched: {', '.join(direct_field_hits)})"
            )

        if shortcut_consumer_hits and not has_approved_helper:
            problems.append(
                f"{rel}: raw WindowInputContextService reads must not feed command/shortcut "
                f"consumers without a runtime helper overlay "
                f"(matched: {', '.join(shortcut_consumer_hits)})"
            )

    if problems:
        print(f"[gate] {GATE_NAME}")
        print(f"[gate] FAIL: {len(problems)} problem(s)")
        for problem in problems:
            print(f"  - {problem}")
        raise SystemExit(1)

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
