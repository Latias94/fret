from __future__ import annotations

import re
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "fret builder-only surface"

LIB_RS = WORKSPACE_ROOT / "ecosystem/fret/src/lib.rs"
APP_ENTRY_RS = WORKSPACE_ROOT / "ecosystem/fret/src/app_entry.rs"
README_MD = WORKSPACE_ROOT / "ecosystem/fret/README.md"


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.relative_to(WORKSPACE_ROOT)}: {exc}")


def require_snippets(path: Path, text: str, snippets: list[str]) -> list[str]:
    missing: list[str] = []
    for snippet in snippets:
        if snippet not in text:
            missing.append(
                f"missing required snippet in {path.relative_to(WORKSPACE_ROOT)}: {snippet!r}"
            )
    return missing


def forbid_snippets(path: Path, text: str, snippets: list[str]) -> list[str]:
    violations: list[str] = []
    for snippet in snippets:
        if snippet in text:
            violations.append(
                f"found forbidden snippet in {path.relative_to(WORKSPACE_ROOT)}: {snippet!r}"
            )
    return violations


def require_regexes(path: Path, text: str, patterns: list[str]) -> list[str]:
    missing: list[str] = []
    for pattern in patterns:
        if re.search(pattern, text, flags=re.MULTILINE) is None:
            missing.append(
                f"missing required pattern in {path.relative_to(WORKSPACE_ROOT)}: {pattern!r}"
            )
    return missing


def forbid_regexes(path: Path, text: str, patterns: list[str]) -> list[str]:
    violations: list[str] = []
    for pattern in patterns:
        if re.search(pattern, text, flags=re.MULTILINE) is not None:
            violations.append(
                f"found forbidden pattern in {path.relative_to(WORKSPACE_ROOT)}: {pattern!r}"
            )
    return violations


def main() -> None:
    lib_text = read_text(LIB_RS)
    app_entry_text = read_text(APP_ENTRY_RS)
    readme_text = read_text(README_MD)

    problems: list[str] = []
    problems.extend(
        forbid_regexes(
            LIB_RS,
            lib_text,
            patterns=[
                r"\bpub\s+fn\s+app_with_hooks\s*<",
                r"\bpub\s+fn\s+app\s*<",
                r"\bpub\s+fn\s+run_with_hooks\s*<",
                r"\bpub\s+fn\s+run\s*<",
            ],
        )
    )
    problems.extend(
        require_regexes(
            APP_ENTRY_RS,
            app_entry_text,
            patterns=[
                r"\bpub\s+fn\s+ui_with_hooks\s*<",
                r"\bpub\s+fn\s+view_with_hooks\s*<",
                r"\bpub\s+fn\s+run_ui_with_hooks\s*<",
                r"\bfn\s+finish_builder\s*<",
            ],
        )
    )
    problems.extend(
        require_snippets(
            README_MD,
            readme_text,
            snippets=[
                'FretApp::new("hello")',
                "fret::App::new(...).window(...).ui(...)?",
                "fret::App::new(...).window(...).ui_with_hooks(...)?",
                "fret::App::new(...).window(...).view::<V>()?",
                "fret::App::new(...).window(...).view_with_hooks::<V>(...)?",
            ],
        )
    )
    problems.extend(
        forbid_snippets(
            README_MD,
            readme_text,
            snippets=[
                "fret::app(",
                "fret::app_with_hooks",
                "fret::run(",
                "fret::run_with_hooks",
            ],
        )
    )
    problems.extend(
        require_snippets(
            LIB_RS,
            lib_text,
            snippets=[
                "fret::App::new(...).window(...).ui(...)?",
                "fret::run_native_with_fn_driver(...)",
            ],
        )
    )
    problems.extend(
        forbid_snippets(
            LIB_RS,
            lib_text,
            snippets=[
                "fret::app(",
                "fret::app_with_hooks",
                "fret::run(",
                "fret::run_with_hooks",
            ],
        )
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
