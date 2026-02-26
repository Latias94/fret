#!/usr/bin/env python3
"""
Flatten `kind: script_redirect` stubs by rewriting their `to` field to the final target.

This is useful after multi-hop migrations (e.g. top-level -> ui-gallery/misc -> ui-gallery/<bucket>),
to reduce redirect chain depth without breaking legacy paths.

Design goals:
- Safe by default (--check is the default mode).
- Dependency-free (stdlib only).
- Loop-safe (detect redirect cycles).
"""

from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT_SENTINEL = "Cargo.toml"
SCRIPTS_DIR = Path("tools/diag-scripts")


def find_repo_root(start: Path) -> Path:
    cur = start.resolve()
    for parent in [cur, *cur.parents]:
        if (parent / REPO_ROOT_SENTINEL).is_file():
            return parent
    raise SystemExit(f"error: failed to locate repo root (missing {REPO_ROOT_SENTINEL} in ancestors)")


@dataclass(frozen=True)
class FlattenOp:
    src: Path
    old_to: str
    new_to: str
    chain: tuple[str, ...]


def read_json(path: Path) -> object:
    return json.loads(path.read_text(encoding="utf-8"))


def is_redirect(obj: object) -> bool:
    return isinstance(obj, dict) and obj.get("kind") == "script_redirect"


def resolve_redirect_chain(repo_root: Path, start_to: str, max_hops: int) -> tuple[str, tuple[str, ...]]:
    """
    Returns (final_to, chain) where chain is the list of redirect "to" values traversed.
    The returned final_to is the last "to" value in the chain (which may be non-redirect JSON).
    """
    seen: set[str] = set()
    chain: list[str] = []
    cur_to = start_to

    for _ in range(max_hops):
        if cur_to in seen:
            raise SystemExit(f"error: redirect loop detected at: {cur_to} (chain={chain})")
        seen.add(cur_to)
        chain.append(cur_to)

        cur_path = repo_root / Path(cur_to)
        if not cur_path.is_file():
            # Missing targets should be surfaced, not silently rewritten.
            raise SystemExit(f"error: redirect target does not exist: {cur_to}")

        raw = read_json(cur_path)
        if not is_redirect(raw):
            return (cur_to, tuple(chain))

        nxt = raw.get("to")
        if not isinstance(nxt, str) or not nxt:
            raise SystemExit(f"error: invalid redirect stub (missing to): {cur_to}")
        cur_to = nxt

    raise SystemExit(f"error: exceeded max redirect hops ({max_hops}) starting from: {start_to}")


def plan_flatten(repo_root: Path, roots: list[Path], max_hops: int) -> list[FlattenOp]:
    ops: list[FlattenOp] = []
    for root in roots:
        base = repo_root / root
        if not base.exists():
            continue
        for p in sorted(base.glob("*.json")):
            try:
                raw = read_json(p)
            except Exception:
                continue
            if not is_redirect(raw):
                continue
            old_to = raw.get("to")
            if not isinstance(old_to, str) or not old_to:
                raise SystemExit(f"error: invalid redirect stub (missing to): {p}")
            final_to, chain = resolve_redirect_chain(repo_root, old_to, max_hops=max_hops)
            if final_to != old_to:
                ops.append(
                    FlattenOp(
                        src=p,
                        old_to=old_to,
                        new_to=final_to,
                        chain=chain,
                    )
                )
    return ops


def apply_flatten(repo_root: Path, ops: list[FlattenOp]) -> None:
    for op in ops:
        raw = read_json(op.src)
        if not is_redirect(raw):
            continue
        raw["to"] = op.new_to
        op.src.write_text(json.dumps(raw, indent=2) + "\n", encoding="utf-8")


def main() -> None:
    ap = argparse.ArgumentParser(description="Flatten script_redirect stubs by resolving multi-hop redirect chains.")
    ap.add_argument("--cwd", default=".", help="Starting directory used to locate repo root (default: .).")
    ap.add_argument(
        "--root",
        action="append",
        default=[],
        help=(
            "Repo-relative directory to scan for redirect stubs (non-recursive; *.json). "
            "Repeatable. Default: tools/diag-scripts (top-level only)."
        ),
    )
    ap.add_argument("--max-hops", type=int, default=8, help="Maximum redirect hops allowed (default: 8).")
    ap.add_argument("--write", action="store_true", help="Apply changes (default: dry-run).")
    ap.add_argument("--json", action="store_true", help="Emit JSON output (default: human-readable).")
    args = ap.parse_args()

    repo_root = find_repo_root(Path(args.cwd))
    roots = [Path(r) for r in args.root]
    if not roots:
        roots = [SCRIPTS_DIR]

    ops = plan_flatten(repo_root, roots, max_hops=args.max_hops)

    if args.json:
        payload = [
            {
                "src": str(op.src.relative_to(repo_root)).replace("\\", "/"),
                "old_to": op.old_to,
                "new_to": op.new_to,
                "chain": list(op.chain),
            }
            for op in ops
        ]
        print(json.dumps({"schema_version": 1, "ops": payload}, indent=2))
    else:
        print(f"planned rewrites: {len(ops)}")
        for op in ops[:25]:
            rel = str(op.src.relative_to(repo_root)).replace("\\", "/")
            print(f"- {rel}: {op.old_to} -> {op.new_to}")
        if len(ops) > 25:
            print(f"(+{len(ops) - 25} more)")

    if not args.write:
        if not args.json:
            print("dry-run (no writes). Use --write to apply.")
        return

    apply_flatten(repo_root, ops)
    if not args.json:
        print("applied rewrites.")


if __name__ == "__main__":
    main()

