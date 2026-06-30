#!/usr/bin/env python3
"""Responsibility source-policy checks for Fret UI architecture surfaces.

This gate complements dependency layering. It catches source-level drift that cargo metadata cannot
see: default authoring paths importing raw runtime seams, advanced/manual seams being used without
classification, and `fret-ui` root exports growing policy-coded names.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable, Sequence


WORKSPACE_ROOT = Path(__file__).resolve().parent.parent
GATE_NAME = "surface responsibility policy"


@dataclass(frozen=True)
class SurfacePath:
    path: str
    category: str
    reason: str


@dataclass(frozen=True)
class SurfaceViolation:
    rule: str
    path: Path
    line_no: int
    message: str
    source: str = ""


DEFAULT_AUTHORING_SURFACES: tuple[SurfacePath, ...] = (
    SurfacePath(
        "README.md",
        "default_app_clean",
        "repository first-contact snippets should teach the app facade",
    ),
    SurfacePath(
        "docs/first-hour.md",
        "default_app_clean",
        "first-hour docs are copied by app authors",
    ),
    SurfacePath(
        "docs/examples/README.md",
        "default_app_clean",
        "example taxonomy should keep default snippets on the app facade",
    ),
    SurfacePath(
        "docs/examples/todo-app-golden-path.md",
        "default_app_clean",
        "todo golden path is a default app tutorial",
    ),
    SurfacePath(
        "crates/fretboard/src/scaffold/contracts.rs",
        "default_app_clean",
        "public scaffold contracts should not expose runtime seams",
    ),
    SurfacePath(
        "crates/fretboard/src/scaffold/templates.rs",
        "default_app_clean",
        "generated app templates are copied verbatim",
    ),
)

POLICY_RECIPE_SURFACES: tuple[SurfacePath, ...] = (
    SurfacePath(
        "ecosystem/fret-ui-kit/src",
        "policy_recipe",
        "policy and headless infrastructure may consume generic fret-ui mechanisms",
    ),
    SurfacePath(
        "ecosystem/fret-ui-shadcn/src",
        "policy_recipe",
        "recipe crates may compose mechanism types into shadcn policy surfaces",
    ),
)

ADVANCED_MANUAL_SURFACES: tuple[SurfacePath, ...] = (
    SurfacePath(
        "apps/fret-examples/src/api_workbench_lite_demo.rs",
        "advanced_manual",
        "second-hour prototype with raw runtime imports until U3 adds a public workbench-lite scaffold",
    ),
    SurfacePath(
        "apps/fret-examples/src/workspace_shell_demo",
        "advanced_manual",
        "workspace shell proof currently owns FnDriver/UiTree seams directly",
    ),
    SurfacePath(
        "apps/fret-cookbook/examples/canvas_pan_zoom_basics.rs",
        "advanced_manual",
        "canvas starter still exposes pointer/canvas runtime seams until a public wrapper lands",
    ),
    SurfacePath(
        "apps/fret-examples/src/node_graph_demo.rs",
        "advanced_manual",
        "node graph remains an advanced proof until the public app ladder has a starter",
    ),
    SurfacePath(
        "crates/fret-framework/src/lib.rs",
        "advanced_manual",
        "fret-framework is the manual assembly facade, not the default app crate",
    ),
)

MECHANISM_ROOT_SURFACES: tuple[SurfacePath, ...] = (
    SurfacePath(
        "crates/fret-ui/src/lib.rs",
        "mechanism_crate_root",
        "fret-ui root exports are the public mechanism substrate",
    ),
)

DEFAULT_FORBIDDEN_PATTERNS: tuple[tuple[str, str], ...] = (
    (
        r"\buse\s+fret_ui::",
        "default app/tutorial surfaces must not import `fret_ui`; use `fret::app::prelude::*` or an explicit advanced/component lane",
    ),
    (
        r"\buse\s+fret_core::",
        "default app/tutorial surfaces must not import `fret_core`; use the curated `fret` facade exports",
    ),
    (
        r"\bFnDriver\b",
        "`FnDriver` is an advanced/manual assembly seam, not a default app authoring noun",
    ),
    (
        r"\bUiTree\b",
        "`UiTree` is a retained runtime mechanism, not a default app authoring noun",
    ),
    (
        r"\bElementContext\b",
        "`ElementContext` is a runtime/component seam; default app helpers should prefer `AppUi`, `AppRenderContext`, or typed children",
    ),
    (
        r"\bfret::advanced::prelude::\*",
        "`fret::advanced::prelude::*` must stay off default app/tutorial surfaces",
    ),
    (
        r"\badvanced::prelude::\*",
        "`advanced::prelude::*` must stay off default app/tutorial surfaces",
    ),
)

POLICY_CODED_EXPORT_TERMS: tuple[str, ...] = (
    "Dialog",
    "Popover",
    "Menu",
    "Tooltip",
    "Dismiss",
    "Dismissable",
    "AutoFocus",
    "FocusTrap",
    "RovingFocus",
    "Typeahead",
    "ScrollDismiss",
    "HoverIntent",
    "ResizablePanelGroup",
)

MECHANISM_ROOT_EXPORT_CLASSIFICATIONS: dict[str, str] = {}


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def _iter_source_files(path: Path) -> list[Path]:
    if path.is_file():
        return [path] if path.suffix in {".md", ".rs"} else []
    if not path.is_dir():
        return []
    return sorted(
        p
        for p in path.rglob("*")
        if p.is_file() and p.suffix in {".md", ".rs"} and ".git" not in p.parts
    )


def _code_lines_for_scan(path: Path, text: str) -> list[tuple[int, str]]:
    if path.suffix != ".md":
        return list(enumerate(text.splitlines(), start=1))

    lines: list[tuple[int, str]] = []
    in_fence = False
    include_fence = False
    fence_re = re.compile(r"^\s*```([A-Za-z0-9_-]*)")
    for line_no, line in enumerate(text.splitlines(), start=1):
        match = fence_re.match(line)
        if match:
            if in_fence:
                in_fence = False
                include_fence = False
            else:
                lang = match.group(1).lower()
                in_fence = True
                include_fence = lang in {"", "rust", "rs"}
            continue
        if in_fence and include_fence:
            lines.append((line_no, line))
    return lines


def _is_rust_source_line_ignorable(line: str) -> bool:
    stripped = line.strip()
    return stripped.startswith("//") or stripped.startswith("///") or stripped.startswith("//!")


def _scan_default_authoring_surface(root: Path, spec: SurfacePath) -> list[SurfaceViolation]:
    violations: list[SurfaceViolation] = []
    for path in _iter_source_files(root / spec.path):
        text = _read_text(path)
        for line_no, line in _code_lines_for_scan(path, text):
            if path.suffix == ".rs" and _is_rust_source_line_ignorable(line):
                continue
            for pattern, message in DEFAULT_FORBIDDEN_PATTERNS:
                if re.search(pattern, line):
                    violations.append(
                        SurfaceViolation(
                            rule="default-app-clean",
                            path=path,
                            line_no=line_no,
                            message=message,
                            source=line.strip(),
                        )
                    )
                    break
    return violations


def _collect_root_public_statements(text: str) -> list[tuple[int, str]]:
    public_text = text.split("#[cfg(test)]\nmod ", 1)[0]
    statements: list[tuple[int, str]] = []
    collecting = False
    start_line = 0
    parts: list[str] = []

    for line_no, line in enumerate(public_text.splitlines(), start=1):
        stripped = line.strip()
        if not collecting:
            if not re.match(r"^(#\[[^\]]+\]\s*)?pub\s+(use|type|struct|enum|trait|fn|mod)\b", stripped):
                continue
            collecting = True
            start_line = line_no
            parts = [stripped]
        else:
            parts.append(stripped)

        if ";" in stripped or stripped.endswith("{"):
            statements.append((start_line, " ".join(parts)))
            collecting = False
            start_line = 0
            parts = []

    if collecting and parts:
        statements.append((start_line, " ".join(parts)))
    return statements


def _exported_symbols(statement: str) -> set[str]:
    words = set(re.findall(r"\b[A-Z][A-Za-z0-9_]*\b", statement))
    module_match = re.search(r"\bpub\s+mod\s+([a-zA-Z0-9_]+)\b", statement)
    if module_match:
        words.add(module_match.group(1))
    return words


def _is_policy_coded_statement(statement: str) -> bool:
    return any(term in statement for term in POLICY_CODED_EXPORT_TERMS)


def _classification_reason(symbols: Iterable[str]) -> str | None:
    for symbol in symbols:
        reason = MECHANISM_ROOT_EXPORT_CLASSIFICATIONS.get(symbol)
        if reason:
            return reason
    return None


def _scan_mechanism_root(root: Path, spec: SurfacePath) -> list[SurfaceViolation]:
    path = root / spec.path
    if not path.exists():
        return []
    violations: list[SurfaceViolation] = []
    for line_no, statement in _collect_root_public_statements(_read_text(path)):
        if not _is_policy_coded_statement(statement):
            continue
        symbols = _exported_symbols(statement)
        if _classification_reason(symbols):
            continue
        term = next(term for term in POLICY_CODED_EXPORT_TERMS if term in statement)
        violations.append(
            SurfaceViolation(
                rule="mechanism-root-policy-vocabulary",
                path=path,
                line_no=line_no,
                message=(
                    f"`crates/fret-ui` root export contains policy-coded term `{term}` without "
                    "an explicit mechanism/compat classification"
                ),
                source=statement,
            )
        )
    return violations


def _validate_surface_specs(specs: Sequence[SurfacePath]) -> list[SurfaceViolation]:
    violations: list[SurfaceViolation] = []
    for spec in specs:
        if not spec.reason.strip():
            violations.append(
                SurfaceViolation(
                    rule="surface-classification-reason",
                    path=Path(spec.path),
                    line_no=1,
                    message=f"{spec.category} surface classification must include a reason",
                )
            )
    for symbol, reason in MECHANISM_ROOT_EXPORT_CLASSIFICATIONS.items():
        if not reason.strip():
            violations.append(
                SurfaceViolation(
                    rule="surface-classification-reason",
                    path=Path("crates/fret-ui/src/lib.rs"),
                    line_no=1,
                    message=f"compat export `{symbol}` must include a reason",
                )
            )
    return violations


def _validate_existing_classified_surfaces(
    root: Path,
    specs: Sequence[SurfacePath],
) -> list[SurfaceViolation]:
    violations: list[SurfaceViolation] = []
    for spec in specs:
        path = root / spec.path
        if not path.exists():
            violations.append(
                SurfaceViolation(
                    rule="classified-surface-exists",
                    path=path,
                    line_no=1,
                    message=(
                        f"{spec.category} surface classification points to a missing path; "
                        "remove stale classifications or update the path"
                    ),
                )
            )
    return violations


def check_surface_policy(
    root: Path,
    *,
    default_surfaces: Sequence[SurfacePath] = DEFAULT_AUTHORING_SURFACES,
    advanced_manual_surfaces: Sequence[SurfacePath] = ADVANCED_MANUAL_SURFACES,
    policy_recipe_surfaces: Sequence[SurfacePath] = POLICY_RECIPE_SURFACES,
    mechanism_root_surfaces: Sequence[SurfacePath] = MECHANISM_ROOT_SURFACES,
) -> list[SurfaceViolation]:
    specs = [
        *default_surfaces,
        *advanced_manual_surfaces,
        *policy_recipe_surfaces,
        *mechanism_root_surfaces,
    ]
    violations = _validate_surface_specs(specs)
    violations.extend(_validate_existing_classified_surfaces(root, specs))

    for spec in default_surfaces:
        violations.extend(_scan_default_authoring_surface(root, spec))

    # Advanced/manual surfaces are explicitly classified so their raw runtime imports are not
    # mistaken for default app authoring. They should move out of this list as U3/U9 add public
    # ladders or facade splits.
    _ = advanced_manual_surfaces

    # Policy/recipe surfaces are intentionally classified here. Dependency direction and backend
    # leakage remain owned by `tools/check_layering.py`; this checker only prevents treating their
    # mechanism consumption as a default-app violation.
    _ = policy_recipe_surfaces

    for spec in mechanism_root_surfaces:
        violations.extend(_scan_mechanism_root(root, spec))

    return violations


def _print_violations(root: Path, violations: Sequence[SurfaceViolation]) -> None:
    print(f"[gate] {GATE_NAME}")
    if not violations:
        print("[gate] ok")
        return

    print(f"[gate] FAIL: {len(violations)} violation(s)")
    for violation in violations[:80]:
        try:
            rel = violation.path.resolve().relative_to(root.resolve())
        except ValueError:
            rel = violation.path
        print(f"  - {rel}:{violation.line_no}: {violation.rule}")
        print(f"      {violation.message}")
        if violation.source:
            print(f"      {violation.source}")
    if len(violations) > 80:
        print(f"  ... and {len(violations) - 80} more")


def main(argv: Sequence[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--root",
        type=Path,
        default=WORKSPACE_ROOT,
        help="repository root to scan (default: current Fret workspace root)",
    )
    args = parser.parse_args(argv)

    root = args.root.resolve()
    violations = check_surface_policy(root)
    _print_violations(root, violations)
    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
