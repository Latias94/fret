from __future__ import annotations

from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok, scan_regexes


GATE_NAME = "compat runner default-surface policy"

FRET_README = WORKSPACE_ROOT / "ecosystem/fret/README.md"
FRET_LIB = WORKSPACE_ROOT / "ecosystem/fret/src/lib.rs"

DEFAULT_PATH_FILES = [
    WORKSPACE_ROOT / "README.md",
    WORKSPACE_ROOT / "docs/first-hour.md",
    WORKSPACE_ROOT / "docs/examples/README.md",
    WORKSPACE_ROOT / "docs/crate-usage-guide.md",
    WORKSPACE_ROOT / "docs/ui-ergonomics-and-interop.md",
    WORKSPACE_ROOT / "apps/fret-cookbook/README.md",
    WORKSPACE_ROOT / "apps/fret-cookbook/EXAMPLES.md",
]


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.relative_to(WORKSPACE_ROOT)}: {exc}")


def require_snippet(path: Path, snippet: str) -> str | None:
    text = read_text(path)
    if snippet not in text:
        return f"missing required snippet in {path.relative_to(WORKSPACE_ROOT)}: {snippet!r}"
    return None


def main() -> None:
    problems: list[str] = []

    for path, snippet in [
        (
            FRET_README,
            "Advanced low-level interop driver path (compat seam, non-default): `fret::run_native_with_compat_driver(...)`",
        ),
        (
            FRET_LIB,
            "`fret::run_native_with_compat_driver(...)` is an advanced low-level interop path (non-default)",
        ),
    ]:
        problem = require_snippet(path, snippet)
        if problem is not None:
            problems.append(problem)

    hits = scan_regexes(DEFAULT_PATH_FILES, [r"run_native_with_compat_driver\s*\("])
    for hit in hits:
        problems.append(
            "default-path surface unexpectedly mentions compat runner: "
            f"{hit.path.resolve().relative_to(WORKSPACE_ROOT)}:{hit.line_no}"
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
