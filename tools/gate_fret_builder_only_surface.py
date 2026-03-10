from __future__ import annotations

import re
from pathlib import Path

from _gate_lib import WORKSPACE_ROOT, fail, ok


GATE_NAME = "fret builder-only surface"

LIB_RS = WORKSPACE_ROOT / "ecosystem/fret/src/lib.rs"
APP_ENTRY_RS = WORKSPACE_ROOT / "ecosystem/fret/src/app_entry.rs"
README_MD = WORKSPACE_ROOT / "ecosystem/fret/README.md"
DOCS_README_MD = WORKSPACE_ROOT / "docs/README.md"
FIRST_HOUR_MD = WORKSPACE_ROOT / "docs/first-hour.md"
LAUNCH_SURFACE_DESIGN_MD = (
    WORKSPACE_ROOT / "docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/DESIGN.md"
)
LAUNCH_SURFACE_AUDIT_MD = (
    WORKSPACE_ROOT / "docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/SURFACE_AUDIT.md"
)
LAUNCH_SURFACE_TODO_MD = (
    WORKSPACE_ROOT / "docs/workstreams/fret-launch-app-surface-fearless-refactor-v1/TODO.md"
)
CUSTOM_EFFECT_AUTHORING_MD = (
    WORKSPACE_ROOT
    / "docs/workstreams/renderer-effects-semantics-and-extensibility-v1/custom-effect-v2/authoring-install-pattern.md"
)
UI_MEMORY_FOOTPRINT_README = (
    WORKSPACE_ROOT / "docs/workstreams/ui-memory-footprint-closure-v1/README.md"
)


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace")
    except OSError as exc:
        fail(GATE_NAME, f"failed to read {path.relative_to(WORKSPACE_ROOT)}: {exc}")


def rustdoc_only(text: str) -> str:
    return "\n".join(line for line in text.splitlines() if line.startswith("//!"))


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
    lib_rustdoc_text = rustdoc_only(lib_text)
    app_entry_text = read_text(APP_ENTRY_RS)
    readme_text = read_text(README_MD)
    docs_readme_text = read_text(DOCS_README_MD)
    first_hour_text = read_text(FIRST_HOUR_MD)
    launch_surface_design_text = read_text(LAUNCH_SURFACE_DESIGN_MD)
    launch_surface_audit_text = read_text(LAUNCH_SURFACE_AUDIT_MD)
    launch_surface_todo_text = read_text(LAUNCH_SURFACE_TODO_MD)
    custom_effect_authoring_text = read_text(CUSTOM_EFFECT_AUTHORING_MD)
    ui_memory_footprint_text = read_text(UI_MEMORY_FOOTPRINT_README)

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
                r"\bpub\s+struct\s+FretApp\b",
                r"\bpub\s+fn\s+view\s*<",
                r"\bpub\s+fn\s+view_with_hooks\s*<",
                r"\bfn\s+finish_builder\s*<",
            ],
        )
    )
    problems.extend(
        forbid_regexes(
            APP_ENTRY_RS,
            app_entry_text,
            patterns=[
                r"\bpub\s+struct\s+App\b",
                r"\bpub\s+fn\s+ui\s*<",
                r"\bpub\s+fn\s+ui_with_hooks\s*<",
                r"\bpub\s+fn\s+run_ui\s*<",
                r"\bpub\s+fn\s+run_ui_with_hooks\s*<",
                r"\bpub\s+fn\s+run_view\s*<",
                r"\bpub\s+fn\s+run_view_with_hooks\s*<",
            ],
        )
    )
    for current_doc, current_text in [
        (LAUNCH_SURFACE_DESIGN_MD, launch_surface_design_text),
        (LAUNCH_SURFACE_AUDIT_MD, launch_surface_audit_text),
        (LAUNCH_SURFACE_TODO_MD, launch_surface_todo_text),
        (CUSTOM_EFFECT_AUTHORING_MD, custom_effect_authoring_text),
        (UI_MEMORY_FOOTPRINT_README, ui_memory_footprint_text),
    ]:
        problems.extend(
            forbid_snippets(
                current_doc,
                current_text,
                snippets=[
                    "fret::App",
                    "fret::AppBuilder",
                ],
            )
        )
    problems.extend(
        require_snippets(
            README_MD,
            readme_text,
            snippets=[
                'FretApp::new("hello")',
                "fret::FretApp::new(...).window(...).view::<V>()?",
                "fret::FretApp::new(...).window(...).view_with_hooks::<V>(...)?",
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
                "fret::App::new(...).window(...).view::<V>()?",
                "fret::App::new(...).window(...).view_with_hooks::<V>(...)?",
                "fret::App::new(...).window(...).ui(...)?",
                "fret::App::new(...).window(...).ui_with_hooks(...)?",
            ],
        )
    )
    problems.extend(
        require_snippets(
            DOCS_README_MD,
            docs_readme_text,
            snippets=[
                "use fret::app::prelude::*;",
                "FretApp::new(...).window(...).view::<MyView>()?.run()",
                "cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()",
            ],
        )
    )
    problems.extend(
        forbid_snippets(
            DOCS_README_MD,
            docs_readme_text,
            snippets=[
                "run_view::<",
                "ViewCx::",
                "use fret::prelude::*;",
            ],
        )
    )
    problems.extend(
        require_snippets(
            FIRST_HOUR_MD,
            first_hour_text,
            snippets=[
                "use fret::app::prelude::*;",
                'FretApp::new("my-simple-todo").window("my-simple-todo", (...)).view::<TodoView>()?.run()',
                "fn render(&mut self, cx: &mut AppUi<'_, '_>) -> Ui",
                "cx.state()`, `cx.actions()`, `cx.data()`, `cx.effects()",
            ],
        )
    )
    problems.extend(
        forbid_snippets(
            FIRST_HOUR_MD,
            first_hour_text,
            snippets=[
                "run_view::<",
                "ViewCx::",
                "use fret::prelude::*;",
                "fret_ui_shadcn::prelude::*",
            ],
        )
    )
    problems.extend(
        require_snippets(
            LIB_RS,
            lib_rustdoc_text,
            snippets=[
                "fret::FretApp::new(...).window(...).view::<V>()?",
                "fret::run_native_with_fn_driver(...)",
            ],
        )
    )
    problems.extend(
        forbid_snippets(
            LIB_RS,
            lib_rustdoc_text,
            snippets=[
                "fret::app(",
                "fret::app_with_hooks",
                "fret::run(",
                "fret::run_with_hooks",
                "fret::App::new(...).window(...).view::<V>()?",
                "fret::App::new(...).window(...).view_with_hooks::<V>(...)?",
                "fret::App::new(...).window(...).ui(...)?",
                "fret::App::new(...).window(...).ui_with_hooks(...)?",
            ],
        )
    )
    problems.extend(
        forbid_regexes(
            LIB_RS,
            lib_text,
            patterns=[
                r"(?m)^\s*pub use app_entry::App;\s*$",
                r"(?m)^\s*pub use app_entry::App as AppBuilder;\s*$",
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
