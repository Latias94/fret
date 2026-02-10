#!/usr/bin/env python3
"""
Material3 token import + audit check.

Cross-platform replacement for `tools/check_material3_tokens.ps1`.
"""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from pathlib import Path


def _repo_root() -> Path:
    proc = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        raise RuntimeError(proc.stderr.strip() or "git rev-parse --show-toplevel failed")
    return Path(proc.stdout.strip()).resolve()


def _default_material_web_dir(repo_root: Path) -> Path:
    common = subprocess.run(
        ["git", "rev-parse", "--git-common-dir"],
        cwd=str(repo_root),
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if common.returncode != 0:
        return repo_root / "repo-ref" / "material-web"
    common_dir = Path(common.stdout.strip())
    if not common_dir.is_absolute():
        common_dir = (repo_root / common_dir).resolve()
    repo_root_from_common = common_dir.parent
    return repo_root_from_common / "repo-ref" / "material-web"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--material-web-dir", default="")
    parser.add_argument("--update", action="store_true")
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    os.chdir(repo_root)
    print(f"Repo: {repo_root}")

    material_web = Path(args.material_web_dir).expanduser().resolve() if args.material_web_dir else _default_material_web_dir(repo_root)
    if material_web.exists():
        rev = subprocess.run(
            ["git", "-C", str(material_web), "rev-parse", "HEAD"],
            check=False,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
        if rev.returncode == 0:
            print(f"Material Web: {material_web}")
            print(f"Material Web rev: {rev.stdout.strip()}")
        else:
            print(f"Material Web: {material_web} (rev unknown)")
    else:
        print(
            f"Material Web: not found at {material_web} (token tools will attempt auto-discovery or env overrides)",
            file=sys.stderr,
        )

    print("\nToken import check...")
    import_check = subprocess.run(
        ["cargo", "run", "-p", "fret-ui-material3", "--bin", "material3_token_import", "--", "--check"],
        check=False,
    )
    import_code = import_check.returncode

    if import_code != 0 and args.update:
        print("\nUpdating generated token output...")
        update_run = subprocess.run(
            ["cargo", "run", "-p", "fret-ui-material3", "--bin", "material3_token_import"],
            check=False,
        )
        if update_run.returncode != 0:
            return update_run.returncode

        print("\nRe-checking token import...")
        recheck = subprocess.run(
            ["cargo", "run", "-p", "fret-ui-material3", "--bin", "material3_token_import", "--", "--check"],
            check=False,
        )
        if recheck.returncode != 0:
            return recheck.returncode
    else:
        if import_code != 0:
            return import_code

    print("\nToken audit check...")
    audit = subprocess.run(
        ["cargo", "run", "-p", "fret-ui-material3", "--bin", "material3_token_audit", "--", "--check"],
        check=False,
    )
    if audit.returncode != 0:
        return audit.returncode

    print("\nOK")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
