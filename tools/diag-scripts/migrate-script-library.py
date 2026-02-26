#!/usr/bin/env python3
"""
Migrate `tools/diag-scripts/*.json` into a subfolder taxonomy.

Design goals:
- Safe by default (dry-run unless --apply).
- Produces an explicit JSON plan for review.
- Can optionally write legacy-path redirect stubs (tooling-resolved).
- Can optionally rewrite repo references (opt-in; exact-string replacement).
- Supports incremental migrations (filter which scripts are included in a plan).

This script is intentionally dependency-free (stdlib only).
"""

from __future__ import annotations

import argparse
import json
import os
import fnmatch
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


REPO_ROOT_SENTINEL = "Cargo.toml"
SCRIPTS_DIR = Path("tools/diag-scripts")


@dataclass(frozen=True)
class MoveOp:
    src: str
    dst: str
    category: str


@dataclass(frozen=True)
class FilterSpec:
    include_prefixes: tuple[str, ...]
    include_categories: tuple[str, ...]
    include_name_globs: tuple[str, ...]
    exclude_prefixes: tuple[str, ...]
    exclude_categories: tuple[str, ...]
    exclude_name_globs: tuple[str, ...]
    limit: int | None


def find_repo_root(start: Path) -> Path:
    cur = start.resolve()
    for parent in [cur, *cur.parents]:
        if (parent / REPO_ROOT_SENTINEL).is_file():
            return parent
    raise SystemExit(f"error: failed to locate repo root (missing {REPO_ROOT_SENTINEL} in ancestors)")


def iter_scripts_in_dir(repo_root: Path, scan_dir: Path) -> Iterable[Path]:
    base = repo_root / scan_dir
    if not base.is_dir():
        raise SystemExit(f"error: scan dir not found: {base}")
    for p in sorted(base.glob("*.json")):
        name = p.name
        if name in {"index.json"}:
            continue
        # Avoid re-migrating redirect stubs written by previous runs.
        # Redirect stubs are suite/tooling-only compatibility artifacts, not canonical script sources.
        try:
            raw = json.loads(p.read_text(encoding="utf-8"))
            if isinstance(raw, dict) and raw.get("kind") == "script_redirect":
                continue
        except Exception:
            # If it's not valid JSON, keep the previous behavior and include it in the plan.
            pass
        yield p


def categorize_script(filename: str) -> tuple[str, str]:
    """
    Returns (category, subdir_rel) where subdir_rel is relative to `tools/diag-scripts/`.
    """
    n = filename

    if n.startswith("tooling-suite-prelude-") or n.startswith("tooling-prelude-"):
        return ("prelude", "_prelude")

    if n.startswith("docking-arbitration-"):
        return ("docking.arbitration", "docking/arbitration")
    if n.startswith("docking-motion-pilot-"):
        return ("docking.motion_pilot", "docking/motion-pilot")

    if n.startswith("external-texture-imports-"):
        return ("tooling.external_texture_imports", "tooling/external-texture-imports")
    if n.startswith("todo-"):
        return ("tooling.todo", "tooling/todo")

    if n.startswith("ui-gallery-"):
        if n.startswith("ui-gallery-layout-") or "layout-sweep" in n:
            return ("ui_gallery.layout", "ui-gallery/layout")
        if n.startswith("ui-gallery-code-editor-") or "code-editor" in n:
            return ("ui_gallery.code_editor", "ui-gallery/code-editor")
        if n.startswith("ui-gallery-markdown-editor-") or "markdown-editor" in n:
            return ("ui_gallery.markdown_editor", "ui-gallery/markdown-editor")
        if n.startswith("ui-gallery-combobox-"):
            return ("ui_gallery.combobox", "ui-gallery/combobox")
        if n.startswith("ui-gallery-date-picker-") or "date-picker" in n:
            return ("ui_gallery.date_picker", "ui-gallery/date-picker")
        if n.startswith("ui-gallery-select-"):
            return ("ui_gallery.select", "ui-gallery/select")
        if "shadcn" in n or "conformance" in n:
            return ("ui_gallery.shadcn_conformance", "ui-gallery/shadcn-conformance")

        # Larger UI gallery buckets (break up ui_gallery.misc).
        if n.startswith("ui-gallery-material3-"):
            return ("ui_gallery.material3", "ui-gallery/material3")
        if n.startswith("ui-gallery-ai-"):
            return ("ui_gallery.ai", "ui-gallery/ai")
        if n.startswith("ui-gallery-menubar-"):
            return ("ui_gallery.menubar", "ui-gallery/menubar")
        if n.startswith("ui-gallery-command-"):
            return ("ui_gallery.command", "ui-gallery/command")
        if n.startswith("ui-gallery-data-table-"):
            return ("ui_gallery.data_table", "ui-gallery/data-table")
        if n.startswith("ui-gallery-context-menu-"):
            return ("ui_gallery.context_menu", "ui-gallery/context-menu")
        if n.startswith("ui-gallery-dropdown-menu-"):
            return ("ui_gallery.dropdown_menu", "ui-gallery/dropdown-menu")
        if n.startswith("ui-gallery-button-"):
            return ("ui_gallery.button", "ui-gallery/button")
        if n.startswith("ui-gallery-checkbox-"):
            return ("ui_gallery.checkbox", "ui-gallery/checkbox")
        if n.startswith("ui-gallery-sidebar-"):
            return ("ui_gallery.sidebar", "ui-gallery/sidebar")
        if n.startswith("ui-gallery-drawer-"):
            return ("ui_gallery.drawer", "ui-gallery/drawer")
        if n.startswith("ui-gallery-sonner-"):
            return ("ui_gallery.sonner", "ui-gallery/sonner")
        if n.startswith("ui-gallery-table-"):
            return ("ui_gallery.table", "ui-gallery/table")
        if n.startswith("ui-gallery-code-view-"):
            return ("ui_gallery.code_view", "ui-gallery/code-view")
        if n.startswith("ui-gallery-control-chrome-"):
            return ("ui_gallery.control_chrome", "ui-gallery/control-chrome")
        if n.startswith("ui-gallery-collapsible-"):
            return ("ui_gallery.collapsible", "ui-gallery/collapsible")
        if n.startswith("ui-gallery-dropdown-"):
            return ("ui_gallery.dropdown", "ui-gallery/dropdown")
        if n.startswith("ui-gallery-nav-") or n.startswith("ui-gallery-navigation-"):
            return ("ui_gallery.navigation", "ui-gallery/navigation")
        if n.startswith("ui-gallery-carousel-"):
            return ("ui_gallery.carousel", "ui-gallery/carousel")
        if n.startswith("ui-gallery-toggle-"):
            return ("ui_gallery.toggle", "ui-gallery/toggle")
        if n.startswith("ui-gallery-theme-"):
            return ("ui_gallery.theme", "ui-gallery/theme")
        if n.startswith("ui-gallery-typography-"):
            return ("ui_gallery.typography", "ui-gallery/typography")
        if n.startswith("ui-gallery-virtual-list-"):
            return ("ui_gallery.virtual_list", "ui-gallery/virtual-list")

        if n.startswith("ui-gallery-magic-"):
            return ("ui_gallery.magic", "ui-gallery/magic")
        if n.startswith("ui-gallery-accordion-"):
            return ("ui_gallery.accordion", "ui-gallery/accordion")
        if n.startswith("ui-gallery-hover-card-") or n.startswith("ui-gallery-hovercard-"):
            return ("ui_gallery.hover_card", "ui-gallery/hover-card")
        if n.startswith("ui-gallery-motion-presets-"):
            return ("ui_gallery.motion_presets", "ui-gallery/motion-presets")
        if n.startswith("ui-gallery-spinner-"):
            return ("ui_gallery.spinner", "ui-gallery/spinner")
        if n.startswith("ui-gallery-node-graph-"):
            return ("ui_gallery.node_graph", "ui-gallery/node-graph")
        if n.startswith("ui-gallery-native-select-"):
            return ("ui_gallery.native_select", "ui-gallery/native-select")
        if n.startswith("ui-gallery-intro-"):
            return ("ui_gallery.intro", "ui-gallery/intro")
        if n.startswith("ui-gallery-avatar-"):
            return ("ui_gallery.avatar", "ui-gallery/avatar")
        if n.startswith("ui-gallery-breadcrumb-"):
            return ("ui_gallery.breadcrumb", "ui-gallery/breadcrumb")
        if n.startswith("ui-gallery-calendar-"):
            return ("ui_gallery.calendar", "ui-gallery/calendar")
        if n.startswith("ui-gallery-card-"):
            return ("ui_gallery.card", "ui-gallery/card")
        if n.startswith("ui-gallery-alert-"):
            return ("ui_gallery.alert", "ui-gallery/alert")
        if n.startswith("ui-gallery-empty-"):
            return ("ui_gallery.empty", "ui-gallery/empty")
        if n.startswith("ui-gallery-image-"):
            return ("ui_gallery.image", "ui-gallery/image")
        if n.startswith("ui-gallery-item-"):
            return ("ui_gallery.item", "ui-gallery/item")
        if n.startswith("ui-gallery-chart-"):
            return ("ui_gallery.chart", "ui-gallery/chart")
        if n.startswith("ui-gallery-clipboard-"):
            return ("ui_gallery.clipboard", "ui-gallery/clipboard")
        if n.startswith("ui-gallery-field-"):
            return ("ui_gallery.field", "ui-gallery/field")
        if n.startswith("ui-gallery-form-"):
            return ("ui_gallery.form", "ui-gallery/form")
        if n.startswith("ui-gallery-kbd-"):
            return ("ui_gallery.kbd", "ui-gallery/kbd")
        if n.startswith("ui-gallery-label-"):
            return ("ui_gallery.label", "ui-gallery/label")
        if n.startswith("ui-gallery-badge-"):
            return ("ui_gallery.badge", "ui-gallery/badge")
        if n.startswith("ui-gallery-aspect-ratio-"):
            return ("ui_gallery.aspect_ratio", "ui-gallery/aspect-ratio")
        if n.startswith("ui-gallery-pagination-"):
            return ("ui_gallery.pagination", "ui-gallery/pagination")
        if n.startswith("ui-gallery-scroll-area-"):
            return ("ui_gallery.scroll_area", "ui-gallery/scroll-area")
        if n.startswith("ui-gallery-portal-"):
            return ("ui_gallery.portal", "ui-gallery/portal")
        if n.startswith("ui-gallery-progress-"):
            return ("ui_gallery.progress", "ui-gallery/progress")
        if n.startswith("ui-gallery-radio-group-"):
            return ("ui_gallery.radio_group", "ui-gallery/radio-group")
        if n.startswith("ui-gallery-resizable-"):
            return ("ui_gallery.resizable", "ui-gallery/resizable")
        if n.startswith("ui-gallery-settings-"):
            return ("ui_gallery.settings", "ui-gallery/settings")
        if n.startswith("ui-gallery-slider-"):
            return ("ui_gallery.slider", "ui-gallery/slider")
        if n.startswith("ui-gallery-switch-"):
            return ("ui_gallery.switch", "ui-gallery/switch")
        if n.startswith("ui-gallery-tabs-"):
            return ("ui_gallery.tabs", "ui-gallery/tabs")
        if n.startswith("ui-gallery-textarea-"):
            return ("ui_gallery.textarea", "ui-gallery/textarea")
        if n.startswith("ui-gallery-toast-"):
            return ("ui_gallery.toast", "ui-gallery/toast")
        if n.startswith("ui-gallery-topbar-"):
            return ("ui_gallery.topbar", "ui-gallery/topbar")
        if n.startswith("ui-gallery-tree-"):
            return ("ui_gallery.tree", "ui-gallery/tree")
        if n.startswith("ui-gallery-window-"):
            return ("ui_gallery.window", "ui-gallery/window")
        if n.startswith("ui-gallery-windowed-rows-"):
            return ("ui_gallery.windowed_rows", "ui-gallery/windowed-rows")
        if n.startswith("ui-gallery-ui-kit-"):
            return ("ui_gallery.ui_kit", "ui-gallery/ui-kit")
        if n.startswith("ui-gallery-diag-") or n.startswith("ui-gallery-hit-test-only-"):
            return ("ui_gallery.diag", "ui-gallery/diag")
        if n.startswith("ui-gallery-view-cache-"):
            return ("ui_gallery.diag", "ui-gallery/diag")
        if n.startswith("ui-gallery-incoming-"):
            return ("ui_gallery.incoming", "ui-gallery/incoming")
        if n.startswith("ui-gallery-contextmenu-"):
            return ("ui_gallery.context_menu", "ui-gallery/context-menu")
        if n.startswith("ui-gallery-centered-fixed-chrome-"):
            return ("ui_gallery.control_chrome", "ui-gallery/control-chrome")

        # Text buckets: keep IME matching token-based (avoid matching "time").
        if (
            n.startswith("ui-gallery-web-ime-")
            or n.startswith("ui-gallery-input-ime-")
            or "-ime-" in n
        ):
            return ("ui_gallery.text_ime", "ui-gallery/text-ime")
        if (
            n.startswith("ui-gallery-text-wrap-")
            or n.startswith("ui-gallery-text-measure-overlay-")
            or n.startswith("ui-gallery-markdown-wrap-")
            or n.startswith("ui-gallery-markdown-span-")
            or n.startswith("ui-gallery-tabs-wrap-")
        ):
            return ("ui_gallery.text_wrap", "ui-gallery/text-wrap")
        if n.startswith("ui-gallery-text-"):
            return ("ui_gallery.text", "ui-gallery/text")

        if n.startswith("ui-gallery-input-"):
            return ("ui_gallery.input", "ui-gallery/input")

        if n.endswith("-steady.json") or "perf" in n or "resize" in n or "torture" in n:
            return ("ui_gallery.perf", "ui-gallery/perf")
        if (
            "overlay" in n
            or "dialog" in n
            or "popover" in n
            or "tooltip" in n
            or "sheet" in n
            or "modal" in n
        ):
            return ("ui_gallery.overlay", "ui-gallery/overlay")
        return ("ui_gallery.misc", "ui-gallery/misc")

    return ("tooling.misc", "tooling")


def matches_filter_spec(filename: str, category: str, spec: FilterSpec) -> bool:
    def any_glob_match(patterns: tuple[str, ...]) -> bool:
        return any(fnmatch.fnmatchcase(filename, pat) for pat in patterns)

    if spec.exclude_prefixes and any(filename.startswith(p) for p in spec.exclude_prefixes):
        return False
    if spec.exclude_categories and category in spec.exclude_categories:
        return False
    if spec.exclude_name_globs and any_glob_match(spec.exclude_name_globs):
        return False

    if spec.include_prefixes and not any(filename.startswith(p) for p in spec.include_prefixes):
        return False
    if spec.include_categories and category not in spec.include_categories:
        return False
    if spec.include_name_globs and not any_glob_match(spec.include_name_globs):
        return False

    return True


def build_move_plan(repo_root: Path, files: Iterable[Path], filters: FilterSpec) -> list[MoveOp]:
    out: list[MoveOp] = []
    for p in files:
        category, subdir_rel = categorize_script(p.name)
        if not matches_filter_spec(p.name, category, filters):
            continue
        dst = repo_root / SCRIPTS_DIR / subdir_rel / p.name
        if p.resolve() == dst.resolve():
            continue
        out.append(
            MoveOp(
                src=str(p.relative_to(repo_root)).replace("\\", "/"),
                dst=str(dst.relative_to(repo_root)).replace("\\", "/"),
                category=category,
            )
        )
        if filters.limit is not None and len(out) >= filters.limit:
            break
    return out


def write_plan_json(
    repo_root: Path,
    plan_out: Path,
    ops: list[MoveOp],
    filters: FilterSpec,
    scan_dirs: list[Path],
) -> None:
    payload = {
        "schema_version": 1,
        "kind": "diag_script_library_migration_plan",
        "scripts_dir": str(SCRIPTS_DIR).replace("\\", "/"),
        "scan_dirs": [str(p).replace("\\", "/") for p in scan_dirs],
        "filters": {
            "include_prefixes": list(filters.include_prefixes),
            "include_categories": list(filters.include_categories),
            "include_name_globs": list(filters.include_name_globs),
            "exclude_prefixes": list(filters.exclude_prefixes),
            "exclude_categories": list(filters.exclude_categories),
            "exclude_name_globs": list(filters.exclude_name_globs),
            "limit": filters.limit,
        },
        "ops": [op.__dict__ for op in ops],
    }
    plan_out.parent.mkdir(parents=True, exist_ok=True)
    plan_out.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def apply_moves(repo_root: Path, ops: list[MoveOp], write_redirects: bool) -> None:
    for op in ops:
        src = repo_root / Path(op.src)
        dst = repo_root / Path(op.dst)
        dst.parent.mkdir(parents=True, exist_ok=True)
        if dst.exists():
            raise SystemExit(f"error: destination already exists: {dst}")
        src.rename(dst)
        if write_redirects:
            redirect = {
                "schema_version": 1,
                "kind": "script_redirect",
                "to": op.dst,
            }
            src.write_text(json.dumps(redirect, indent=2) + "\n", encoding="utf-8")


def rewrite_references(repo_root: Path, ops: list[MoveOp], roots: list[Path]) -> int:
    """
    Performs exact-string replacement of old paths -> new paths.
    Returns number of files changed.
    """
    mapping = {op.src: op.dst for op in ops}
    changed_files = 0

    for root in roots:
        abs_root = (repo_root / root).resolve()
        if not abs_root.exists():
            continue
        for path in abs_root.rglob("*"):
            if path.is_dir():
                continue
            # Keep it conservative: only rewrite likely text files.
            if path.suffix.lower() not in {".rs", ".md", ".toml", ".json", ".ps1", ".py", ".sh"}:
                continue
            try:
                text = path.read_text(encoding="utf-8")
            except Exception:
                continue
            new_text = text
            for old, new in mapping.items():
                new_text = new_text.replace(old, new)
            if new_text != text:
                path.write_text(new_text, encoding="utf-8")
                changed_files += 1

    return changed_files


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Migrate tools/diag-scripts into a folder taxonomy (dry-run by default)."
    )
    ap.add_argument(
        "--plan-out",
        default=".fret/diag-script-library-migration.plan.json",
        help="Write a JSON plan to this path (repo-relative).",
    )
    ap.add_argument(
        "--apply",
        action="store_true",
        help="Apply the migration (moves files). Without this flag, only writes the plan.",
    )
    ap.add_argument(
        "--write-redirects",
        action="store_true",
        help="After moving, write legacy-path redirect stubs at the old locations (tooling-resolved).",
    )
    ap.add_argument(
        "--rewrite-references",
        choices=["off", "code", "all"],
        default="off",
        help="Rewrite repo references to moved paths (exact replacement).",
    )
    ap.add_argument(
        "--cwd",
        default=".",
        help="Starting directory used to locate repo root (default: .).",
    )
    ap.add_argument(
        "--scan-dir",
        action="append",
        default=[],
        help=(
            "Repo-relative directory to scan for scripts (non-recursive; *.json). "
            "Repeatable. Default: tools/diag-scripts (top-level only)."
        ),
    )

    # Incremental migration filters (optional).
    ap.add_argument(
        "--include-prefix",
        action="append",
        default=[],
        help="Only include scripts whose filenames start with this prefix (repeatable).",
    )
    ap.add_argument(
        "--include-category",
        action="append",
        default=[],
        help="Only include scripts whose inferred category matches exactly (repeatable).",
    )
    ap.add_argument(
        "--include-name-glob",
        action="append",
        default=[],
        help="Only include scripts whose filenames match this glob (repeatable). Example: ui-gallery-select-*.json",
    )
    ap.add_argument(
        "--exclude-prefix",
        action="append",
        default=[],
        help="Exclude scripts whose filenames start with this prefix (repeatable).",
    )
    ap.add_argument(
        "--exclude-category",
        action="append",
        default=[],
        help="Exclude scripts whose inferred category matches exactly (repeatable).",
    )
    ap.add_argument(
        "--exclude-name-glob",
        action="append",
        default=[],
        help="Exclude scripts whose filenames match this glob (repeatable).",
    )
    ap.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Stop after planning N moves (useful for reviewable batches).",
    )
    args = ap.parse_args()

    repo_root = find_repo_root(Path(args.cwd))
    scan_dirs = [Path(s) for s in args.scan_dir]
    if not scan_dirs:
        scan_dirs = [SCRIPTS_DIR]
    files: list[Path] = []
    for d in scan_dirs:
        files.extend(iter_scripts_in_dir(repo_root, d))

    filters = FilterSpec(
        include_prefixes=tuple(args.include_prefix),
        include_categories=tuple(args.include_category),
        include_name_globs=tuple(args.include_name_glob),
        exclude_prefixes=tuple(args.exclude_prefix),
        exclude_categories=tuple(args.exclude_category),
        exclude_name_globs=tuple(args.exclude_name_glob),
        limit=args.limit,
    )

    ops = build_move_plan(repo_root, files, filters)

    plan_out = repo_root / Path(args.plan_out)
    write_plan_json(repo_root, plan_out, ops, filters, scan_dirs)
    print(f"wrote plan: {plan_out}")
    print(f"planned moves: {len(ops)}")

    if not args.apply:
        print("dry-run (no moves applied). Use --apply to execute.")
        return

    apply_moves(repo_root, ops, write_redirects=args.write_redirects)
    print("applied moves.")

    if args.rewrite_references != "off":
        roots = [Path("crates"), Path("apps")]
        if args.rewrite_references == "all":
            roots += [Path("docs"), Path("tools")]
        n = rewrite_references(repo_root, ops, roots)
        print(f"rewrote references in {n} files (roots={','.join(str(r) for r in roots)})")


if __name__ == "__main__":
    # Avoid inheriting odd encodings in some Windows shells.
    os.environ.setdefault("PYTHONIOENCODING", "utf-8")
    main()

