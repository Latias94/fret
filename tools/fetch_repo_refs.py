#!/usr/bin/env python3
"""
Bootstrap helper for local `repo-ref/` checkouts.

Cross-platform replacement for `tools/fetch_repo_refs.ps1`.

Why this exists:
- `repo-ref/` is intentionally ignored by git in this repo (see `.gitignore`).
- Many docs reference paths under `repo-ref/` for upstream reading.
- This script makes those paths reproducible on a new machine without committing large upstream repos.

Usage:
  python3 tools/fetch_repo_refs.py                  # fetch the recommended set (ui + primitives)
  python3 tools/fetch_repo_refs.py --ui-only        # fetch only shadcn/ui
  python3 tools/fetch_repo_refs.py --primitives-only
  python3 tools/fetch_repo_refs.py --force          # re-point origin + checkout pinned commit

Proxy (optional):
  export HTTP_PROXY='http://127.0.0.1:10809'
  export HTTPS_PROXY='http://127.0.0.1:10809'
  export ALL_PROXY='http://127.0.0.1:10809'
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def _resolve_repo_root(start_dir: Path) -> Path:
    current = start_dir.resolve()
    while True:
        if (current / "Cargo.toml").is_file():
            return current
        parent = current.parent
        if parent == current:
            break
        current = parent
    raise RuntimeError(f"Unable to locate repo root from {start_dir} (expected Cargo.toml).")


def _run_git(args: list[str], *, cwd: Path | None = None, quiet: bool = False) -> str:
    proc = subprocess.run(
        ["git", *args],
        cwd=str(cwd) if cwd else None,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        msg = proc.stderr.strip() or proc.stdout.strip() or "git command failed"
        raise RuntimeError(f"{msg} (git {' '.join(args)})")
    if quiet:
        return ""
    return proc.stdout


def _ensure_repo_checkout(*, repo_root: Path, name: str, url: str, commit: str, force: bool) -> None:
    root = repo_root / "repo-ref"
    root.mkdir(parents=True, exist_ok=True)

    path = root / name
    if not path.exists():
        print(f"Cloning {name} -> {path}")
        _run_git(["clone", url, str(path)], quiet=True)

    git_dir = path / ".git"
    if not git_dir.exists():
        print(f"warning: Skipping {name}: {path} exists but is not a git repo.", file=sys.stderr)
        print(
            "warning: If this is a checkout you manage manually, keep it; otherwise delete the folder and re-run.",
            file=sys.stderr,
        )
        return

    origin = _run_git(["remote", "get-url", "origin"], cwd=path).strip()
    head = _run_git(["rev-parse", "--short=12", "HEAD"], cwd=path).strip()
    print(f"Found {name}: origin={origin} head={head}")

    if force and origin != url:
        print(f"Updating {name} origin -> {url}")
        _run_git(["remote", "set-url", "origin", url], cwd=path, quiet=True)

    print(f"Fetching {name}...")
    _run_git(["fetch", "--tags", "origin"], cwd=path, quiet=True)

    # Accept short SHAs (as recorded in docs) and full SHAs.
    target = commit
    print(f"Checking out {name} @ {target}")
    _run_git(["checkout", target], cwd=path, quiet=True)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ui-only", action="store_true")
    parser.add_argument("--primitives-only", action="store_true")
    parser.add_argument("--force", action="store_true")

    parser.add_argument("--repo-root", default="", help="Repo root (auto-detected if empty).")

    parser.add_argument("--ui-url", default="https://github.com/shadcn-ui/ui.git")
    parser.add_argument("--primitives-url", default="https://github.com/radix-ui/primitives.git")

    # Pinned commits recorded in `docs/repo-ref.md`.
    parser.add_argument("--ui-commit", default="d07a7af8")
    parser.add_argument("--primitives-commit", default="90751370")

    args = parser.parse_args(argv)

    repo_root = Path(args.repo_root) if args.repo_root else _resolve_repo_root(Path(__file__).parent.parent)

    want_ui = True
    want_primitives = True
    if args.ui_only:
        want_primitives = False
    if args.primitives_only:
        want_ui = False

    if want_ui:
        _ensure_repo_checkout(
            repo_root=repo_root,
            name="ui",
            url=args.ui_url,
            commit=args.ui_commit,
            force=args.force,
        )
    if want_primitives:
        _ensure_repo_checkout(
            repo_root=repo_root,
            name="primitives",
            url=args.primitives_url,
            commit=args.primitives_commit,
            force=args.force,
        )

    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
