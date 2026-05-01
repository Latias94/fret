from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "imui workstream source"


@dataclass(frozen=True)
class SourceCheck:
    path: Path
    required: list[str]
    forbidden: list[str]


def read_source(path: Path) -> str:
    try:
        return (WORKSPACE_ROOT / path).read_text(encoding="utf-8")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.as_posix()}: {exc}")


def check_source(check: SourceCheck, failures: list[str]) -> None:
    source = read_source(check.path)
    for marker in check.required:
        if marker not in source:
            failures.append(f"{check.path.as_posix()}: missing {marker}")
    for marker in check.forbidden:
        if marker in source:
            failures.append(f"{check.path.as_posix()}: forbidden {marker}")


def main() -> None:
    checks = [
        SourceCheck(
            Path("docs/workstreams/imui-response-status-lifecycle-v1/DESIGN.md"),
            required=[
                "`fret-authoring::Response` must stay unchanged.",
                "Richer lifecycle status stays in `fret-ui-kit::imui::ResponseExt`.",
                "The initial quartet is:",
                "`activated`",
                "`deactivated`",
                "`edited`",
                "`deactivated_after_edit`",
                "Do not widen `crates/fret-ui` or invent a global key-owner model in this lane.",
                "`apps/fret-examples/src/imui_response_signals_demo.rs`",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-response-status-lifecycle-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-response-status-lifecycle-v1"',
                '"follow_on_of": "imui-editor-grade-product-closure-v1"',
                "python tools/gate_imui_workstream_source.py",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-editor-grade-product-closure-v1/TODO.md"),
            required=[
                "`docs/workstreams/imui-response-status-lifecycle-v1/` now proves this rule",
                "`docs/workstreams/imui-response-status-lifecycle-v1/` now owns the narrow",
                "`ResponseExt` lifecycle vocabulary slice",
                "`docs/workstreams/imui-key-owner-surface-v1/` now records the closed key-owner /",
                "item-local shortcut ownership follow-on",
                "the current helper-local",
                "first-party proof warrants a different narrow lane, and",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/DESIGN.md"),
            required=[
                "focused `activate_shortcut`",
                "`SetNextItemShortcut()` / `SetItemKeyOwner()`",
                "`crates/fret-ui` must remain unchanged unless stronger ADR-backed evidence appears.",
                "`apps/fret-examples/src/imui_response_signals_demo.rs`",
                "`ecosystem/fret-imui/src/tests/interaction.rs`",
                "Global keymap / command routing semantics remain fixed input, not negotiable scope here.",
                "Do not reopen `ResponseExt` lifecycle vocabulary, collection/pane proof breadth, or richer",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md"),
            required=[
                "Keep `apps/fret-examples/src/imui_response_signals_demo.rs` as the current proof/contract surface",
                "do not promote a new dedicated key-owner proof demo yet.",
                "menu_item_command_uses_command_metadata_shortcut_and_gating",
                "combo_model_activate_shortcut_is_scoped_to_focused_trigger",
                "runtime keymap / IME arbitration",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md"),
            required=[
                "M2 closes on a no-new-surface verdict.",
                "There is still no stronger first-party consumer pressure for a broader key-owner surface.",
                "Do not add a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade.",
                "Do not add a broader item-local shortcut registration seam.",
                "reopen this question only if stronger first-party proof exceeds the current demo/test",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md"),
            required=[
                "Status: closed closeout record",
                "The current helper-local shortcut seams already close the first-party key-owner demand for this cycle",
                "There is still no stronger first-party consumer pressure for a broader key-owner surface",
                "do not add a `SetNextItemShortcut()` / `SetItemKeyOwner()`-scale immediate facade here by",
                "start a different narrow lane with stronger first-party proof if future pressure still",
            ],
            forbidden=[],
        ),
        SourceCheck(
            Path("docs/workstreams/imui-key-owner-surface-v1/WORKSTREAM.json"),
            required=[
                '"slug": "imui-key-owner-surface-v1"',
                '"status": "closed"',
                '"follow_on_of": "imui-editor-grade-product-closure-v1"',
                '"path": "docs/workstreams/imui-key-owner-surface-v1/M1_PROOF_ROSTER_FREEZE_2026-04-21.md"',
                '"path": "docs/workstreams/imui-key-owner-surface-v1/M2_NO_NEW_SURFACE_VERDICT_2026-04-21.md"',
                '"path": "docs/workstreams/imui-key-owner-surface-v1/CLOSEOUT_AUDIT_2026-04-21.md"',
                '"default_action": "close"',
                "python tools/gate_imui_workstream_source.py",
                "imui_response_signals_demo",
            ],
            forbidden=[],
        ),
    ]

    failures: list[str] = []
    for check in checks:
        check_source(check, failures)

    if failures:
        fail(GATE_NAME, f"{len(failures)} source marker problem(s):\n  - " + "\n  - ".join(failures))

    ok(GATE_NAME)


if __name__ == "__main__":
    main()
