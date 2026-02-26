#!/usr/bin/env python3
"""
Migrate `tools/diag-scripts/*.json` into a subfolder taxonomy.

Design goals:
- Safe by default (dry-run unless --apply).
- Produces an explicit JSON plan for review.
- Can optionally write legacy-path redirect stubs (tooling-resolved).
- Can optionally rewrite repo references (opt-in; exact-string replacement).

This script is intentionally dependency-free (stdlib only).
"""

from __future__ import annotations

import argparse
import json
import os
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


def find_repo_root(start: Path) -> Path:
    cur = start.resolve()
    for parent in [cur, *cur.parents]:
        if (parent / REPO_ROOT_SENTINEL).is_file():
            return parent
    raise SystemExit(f"error: failed to locate repo root (missing {REPO_ROOT_SENTINEL} in ancestors)")


def iter_top_level_scripts(repo_root: Path) -> Iterable[Path]:
    base = repo_root / SCRIPTS_DIR
    if not base.is_dir():
        raise SystemExit(f"error: scripts dir not found: {base}")
    for p in sorted(base.glob("*.json")):
        name = p.name
        if name in {"index.json"}:
            continue
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
        if n.startswith("ui-gallery-text-") or "-ime-" in n or "ime" in n:
            return ("ui_gallery.text_ime", "ui-gallery/text-ime")
        if "wrap" in n:
            return ("ui_gallery.text_wrap", "ui-gallery/text-wrap")
        if n.startswith("ui-gallery-combobox-"):
            return ("ui_gallery.combobox", "ui-gallery/combobox")
        if n.startswith("ui-gallery-select-"):
            return ("ui_gallery.select", "ui-gallery/select")
        if "shadcn" in n or "conformance" in n:
            return ("ui_gallery.shadcn_conformance", "ui-gallery/shadcn-conformance")
        if n.endswith("-steady.json") or "perf" in n or "resize" in n or "torture" in n:
            return ("ui_gallery.perf", "ui-gallery/perf")
        if "overlay" in n or "dialog" in n or "popover" in n or "tooltip" in n:
            return ("ui_gallery.overlay", "ui-gallery/overlay")
        return ("ui_gallery.misc", "ui-gallery")

    return ("tooling.misc", "tooling")


def build_move_plan(repo_root: Path, files: Iterable[Path]) -> list[MoveOp]:
    out: list[MoveOp] = []
    for p in files:
        category, subdir_rel = categorize_script(p.name)
        dst = repo_root / SCRIPTS_DIR / subdir_rel / p.name
        out.append(
            MoveOp(
                src=str(p.relative_to(repo_root)).replace("\\", "/"),
                dst=str(dst.relative_to(repo_root)).replace("\\", "/"),
                category=category,
            )
        )
    return out


def write_plan_json(repo_root: Path, plan_out: Path, ops: list[MoveOp]) -> None:
    payload = {
        "schema_version": 1,
        "kind": "diag_script_library_migration_plan",
        "scripts_dir": str(SCRIPTS_DIR).replace("\\", "/"),
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
    args = ap.parse_args()

    repo_root = find_repo_root(Path(args.cwd))
    files = list(iter_top_level_scripts(repo_root))
    ops = build_move_plan(repo_root, files)

    plan_out = repo_root / Path(args.plan_out)
    write_plan_json(repo_root, plan_out, ops)
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

