#!/usr/bin/env python3
"""
Analyze Fret diagnostics bundles for scroll handle content-size jumps.

This is intended to debug issues like "scroll range suddenly becomes huge" by locating
`debug.scroll_handle_changes` entries where `content_h/content_w` changes sharply, and
optionally attributing those changes to a semantics subtree via `bound_nodes_sample`.

Usage:
  python tools/diag/analyze_scroll_handle_jumps.py <bundle_dir|bundle.json> --test-id ui-gallery-content-scroll
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from dataclasses import dataclass
from typing import Dict, Iterable, List, Optional, Set, Tuple


@dataclass(frozen=True)
class SemNode:
    node_id: int
    parent: Optional[int]
    role: Optional[str]
    test_id: Optional[str]


def _resolve_bundle_json_path(path: str) -> str:
    path = os.path.abspath(path)
    if os.path.isdir(path):
        candidate = os.path.join(path, "bundle.json")
        if os.path.isfile(candidate):
            return candidate
        raise FileNotFoundError(f"bundle.json not found under directory: {path}")
    if os.path.isfile(path):
        return path
    raise FileNotFoundError(f"path not found: {path}")


def _load_json(path: str) -> dict:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def _pick_semantics_entry(bundle: dict, window_id: int) -> Optional[dict]:
    tables = bundle.get("tables") or {}
    sem = tables.get("semantics") or {}
    entries = sem.get("entries") or []
    for entry in entries:
        if entry.get("window") == window_id:
            return entry.get("semantics")
    return None


def _build_semantics_index(semantics: dict) -> Dict[int, SemNode]:
    out: Dict[int, SemNode] = {}
    for n in semantics.get("nodes") or []:
        nid = n.get("id")
        if not isinstance(nid, int):
            continue
        out[nid] = SemNode(
            node_id=nid,
            parent=n.get("parent"),
            role=n.get("role"),
            test_id=n.get("test_id"),
        )
    return out


def _ancestor_test_ids(
    by_id: Dict[int, SemNode],
    node_id: int,
    memo: Dict[int, Set[str]],
    max_hops: int = 256,
) -> Set[str]:
    if node_id in memo:
        return memo[node_id]

    out: Set[str] = set()
    cur = by_id.get(node_id)
    hops = 0
    while cur is not None and hops < max_hops:
        if cur.test_id:
            out.add(cur.test_id)
        parent = cur.parent
        if parent is None:
            break
        if parent in memo:
            out |= memo[parent]
            break
        cur = by_id.get(parent)
        hops += 1

    memo[node_id] = out
    return out


def _format_px(v: Optional[float]) -> str:
    if v is None:
        return "?"
    return f"{v:.3f}"


def main(argv: List[str]) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("bundle", help="Path to a bundle directory or bundle.json")
    ap.add_argument(
        "--test-id",
        default=None,
        help="Only report scroll handle changes whose bound nodes are within this test_id subtree",
    )
    ap.add_argument(
        "--min-delta-px",
        type=float,
        default=500.0,
        help="Minimum absolute delta (in px) for content_w/content_h to report",
    )
    ap.add_argument(
        "--max",
        type=int,
        default=50,
        help="Maximum number of matching events to print",
    )
    args = ap.parse_args(argv)

    bundle_path = _resolve_bundle_json_path(args.bundle)
    bundle = _load_json(bundle_path)

    windows = bundle.get("windows") or []
    if not windows:
        print("No windows in bundle:", bundle_path, file=sys.stderr)
        return 2

    printed = 0
    for win in windows:
        window_id = win.get("window")
        if not isinstance(window_id, int):
            continue

        semantics = _pick_semantics_entry(bundle, window_id)
        sem_index: Dict[int, SemNode] = _build_semantics_index(semantics) if semantics else {}
        ancestor_memo: Dict[int, Set[str]] = {}

        snapshots = win.get("snapshots") or []
        for snap in snapshots:
            debug = snap.get("debug") or {}
            changes = debug.get("scroll_handle_changes") or []
            if not changes:
                continue

            frame_id = snap.get("frame_id")
            tick_id = snap.get("tick_id")
            ts = snap.get("timestamp_unix_ms")

            for ch in changes:
                if not ch.get("content_changed"):
                    continue

                content_w = ch.get("content_w")
                content_h = ch.get("content_h")
                prev_w = ch.get("prev_content_w")
                prev_h = ch.get("prev_content_h")

                dw = abs(content_w - prev_w) if isinstance(content_w, (int, float)) and isinstance(prev_w, (int, float)) else 0.0
                dh = abs(content_h - prev_h) if isinstance(content_h, (int, float)) and isinstance(prev_h, (int, float)) else 0.0
                if max(dw, dh) < args.min_delta_px:
                    continue

                bound_nodes = ch.get("bound_nodes_sample") or []
                bound_test_ids: Set[str] = set()
                if sem_index and bound_nodes:
                    for nid in bound_nodes:
                        if isinstance(nid, int):
                            bound_test_ids |= _ancestor_test_ids(sem_index, nid, ancestor_memo)

                if args.test_id and args.test_id not in bound_test_ids:
                    continue

                print(
                    f"window={window_id} frame={frame_id} tick={tick_id} ts={ts} "
                    f"handle_key={ch.get('handle_key')} kind={ch.get('kind')}\n"
                    f"  content_w: {_format_px(prev_w)} -> {_format_px(content_w)} (Δ={dw:.3f})\n"
                    f"  content_h: {_format_px(prev_h)} -> {_format_px(content_h)} (Δ={dh:.3f})\n"
                    f"  viewport: {ch.get('viewport_w')}x{ch.get('viewport_h')} "
                    f"offset: ({ch.get('offset_x')},{ch.get('offset_y')})\n"
                    f"  bound_test_ids: {sorted(bound_test_ids)[:12]}{' ...' if len(bound_test_ids) > 12 else ''}\n"
                )

                printed += 1
                if printed >= args.max:
                    break
            if printed >= args.max:
                break
        if printed >= args.max:
            break

    if printed == 0:
        print(
            f"No matching content-size jumps found (min_delta_px={args.min_delta_px}).\n"
            f"bundle={bundle_path}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

