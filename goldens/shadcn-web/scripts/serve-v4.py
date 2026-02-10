#!/usr/bin/env python3
"""
Serve the shadcn/ui v4 app in production mode (for golden extraction).

Cross-platform replacement for `goldens/shadcn-web/scripts/serve-v4.ps1`.
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
        if (current / "repo-ref" / "ui").exists():
            return current
        parent = current.parent
        if parent == current:
            break
        current = parent
    raise RuntimeError(f"Unable to locate repo root from {start_dir} (expected repo-ref/ui).")


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--port", type=int, default=4020)
    parser.add_argument("--app-url", default="")
    parser.add_argument("--v0-url", default="https://v0.dev")
    parser.add_argument("--skip-install", action="store_true")
    parser.add_argument("--skip-shadcn-build", action="store_true")
    parser.add_argument("--skip-build", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _resolve_repo_root(Path(__file__).parent / ".." / ".." / "..")
    ui_root = repo_root / "repo-ref" / "ui"
    v4_root = ui_root / "apps" / "v4"

    if not ui_root.exists():
        raise RuntimeError("Missing repo-ref/ui. Run: python3 tools/fetch_repo_refs.py --ui-only")

    app_url = args.app_url.strip() or f"http://localhost:{args.port}"

    env = os.environ.copy()
    env["NEXT_PUBLIC_APP_URL"] = app_url
    env["NEXT_PUBLIC_V0_URL"] = args.v0_url

    print("shadcn/ui v4 server (production)")
    print(f"  repoRoot: {repo_root}")
    print(f"  uiRoot:   {ui_root}")
    print(f"  v4Root:   {v4_root}")
    print(f"  appUrl:   {app_url}")
    print(f"  port:     {args.port}")

    if not args.skip_install:
        print("Installing deps (pnpm -C repo-ref/ui install)...")
        code = subprocess.run(["pnpm", "-C", str(ui_root), "install"], env=env).returncode
        if code != 0:
            return code

    if not args.skip_shadcn_build:
        print("Building shadcn package (pnpm -C repo-ref/ui --filter shadcn build)...")
        code = subprocess.run(["pnpm", "-C", str(ui_root), "--filter", "shadcn", "build"], env=env).returncode
        if code != 0:
            return code

    if not args.skip_build:
        print("Building v4 app with webpack (pnpm -C repo-ref/ui/apps/v4 exec next build --webpack)...")
        code = subprocess.run(["pnpm", "-C", str(v4_root), "exec", "next", "build", "--webpack"], env=env).returncode
        if code != 0:
            return code

    print(f"Starting server (pnpm -C repo-ref/ui/apps/v4 exec next start -p {args.port})...")
    return subprocess.run(["pnpm", "-C", str(v4_root), "exec", "next", "start", "-p", str(args.port)], env=env).returncode


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
