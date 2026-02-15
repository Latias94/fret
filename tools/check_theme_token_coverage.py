#!/usr/bin/env python3
"""
Report required theme token usage and check coverage in theme JSON files.

This tool scans Rust source for calls that *require* a theme token:
  - color_token / color_required
  - metric_token / metric_required
  - corners_token / corners_required
  - number_token / number_required
  - duration_ms_token / duration_ms_required
  - easing_token / easing_required
  - text_style_token / text_style_required

It then verifies that each configured theme JSON provides the required keys under the
corresponding ThemeConfig maps (colors/metrics/..., etc).

Notes:
- This is a static check based on source strings and ThemeConfig JSON keys. It does not execute
  Theme::apply_config, so it intentionally treats keys as "present" only when they are explicitly
  configured by the theme JSON (or when an obvious alias/compat key is provided).
- If you want runtime-accurate validation ("after apply_config, key exists"), add a Rust gate.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


def _repo_root() -> Path:
    return Path(__file__).parent.parent.resolve()


def _load_json(path: Path) -> dict:
    with path.open("r", encoding="utf-8") as f:
        return json.load(f)


@dataclass(frozen=True)
class RequiredToken:
    kind: str
    key: str
    source: str


_CALL_KIND: dict[str, str] = {
    "color_token": "colors",
    "color_required": "colors",
    "metric_token": "metrics",
    "metric_required": "metrics",
    "corners_token": "corners",
    "corners_required": "corners",
    "number_token": "numbers",
    "number_required": "numbers",
    "duration_ms_token": "durations_ms",
    "duration_ms_required": "durations_ms",
    "easing_token": "easings",
    "easing_required": "easings",
    "text_style_token": "text_styles",
    "text_style_required": "text_styles",
}


_METHOD_RE = re.compile(
    r"""\.(?P<method>color_token|color_required|metric_token|metric_required|corners_token|corners_required|number_token|number_required|duration_ms_token|duration_ms_required|easing_token|easing_required|text_style_token|text_style_required)\(\s*(?P<arg>[^)]*?)\s*\)""",
)

# Normal string literal: "..."
_STR_RE = re.compile(r'"(?P<body>(?:\\.|[^"\\])*)"')

# Raw string literal: r#"..."# / r##"..."## / etc.
_RAW_STR_RE = re.compile(r'r(?P<hashes>#+)?"(?P<body>.*?)"(?P=hashes)', re.DOTALL)

# A simple constant-ish path expression.
_PATH_RE = re.compile(r"(?P<path>[A-Za-z_][A-Za-z0-9_:]*)")


def _unescape_rust_str(s: str) -> str:
    # Keep this conservative; theme token keys should never require complex escapes.
    return (
        s.replace(r"\\", "\\")
        .replace(r"\/", "/")
        .replace(r"\"", '"')
        .replace(r"\n", "\n")
        .replace(r"\t", "\t")
    )


def _canonicalize_color_key(key: str) -> str:
    key = key.strip()
    if not key:
        return key

    # Keep in sync with crates/fret-ui/src/theme/registry.rs (canonicalize_color_key).
    color_aliases = {
        "ring_offset_background": "ring-offset-background",
        "card.background": "card",
        "card.foreground": "card-foreground",
        "popover.background": "popover",
        "popover.foreground": "popover-foreground",
        "input.border": "input",
        "primary.background": "primary",
        "primary.foreground": "primary-foreground",
        "secondary.background": "secondary",
        "secondary.foreground": "secondary-foreground",
        "destructive.background": "destructive",
        "destructive.foreground": "destructive-foreground",
        "muted.background": "muted",
        "muted.foreground": "muted-foreground",
        "accent.background": "accent",
        "accent.foreground": "accent-foreground",
        "chart.1": "chart-1",
        "chart.2": "chart-2",
        "chart.3": "chart-3",
        "chart.4": "chart-4",
        "chart.5": "chart-5",
        "sidebar.background": "sidebar",
        "sidebar-background": "sidebar",
        "sidebar.foreground": "sidebar-foreground",
        "sidebar.border": "sidebar-border",
        "sidebar.ring": "sidebar-ring",
        "sidebar.primary": "sidebar-primary",
        "sidebar.primary.foreground": "sidebar-primary-foreground",
        "sidebar.accent": "sidebar-accent",
        "sidebar.accent.foreground": "sidebar-accent-foreground",
    }
    return color_aliases.get(key, key)


def _canonicalize_required_token(kind: str, key: str) -> str:
    if kind == "colors":
        return _canonicalize_color_key(key)
    return key.strip()


def _theme_token_equivalents(kind: str, key: str) -> set[str]:
    """
    Return a set of keys that we consider "equivalent enough" for JSON coverage checks.

    This is intentionally small and focused on migration/compat keys where runtime derives one
    token from another (typed baseline -> semantic keys) or where shadcn v4 uses multiple naming
    conventions.
    """
    key = key.strip()
    if not key:
        return {key}

    if kind == "colors":
        k = _canonicalize_color_key(key)
        eq = {k}
        # Baseline dotted keys drive typed fields, which drive semantic aliases.
        semantic_to_baseline = {
            "background": {"color.surface.background"},
            "foreground": {"color.text.primary"},
            "border": {"color.panel.border", "input"},
            "input": {"color.panel.border", "border"},
            "ring": {"color.focus.ring"},
            "ring-offset-background": {"background", "color.surface.background"},
            "card": {"color.panel.background"},
            "popover": {"color.menu.background"},
            "popover.border": {"color.menu.border"},
            "muted": {"color.panel.background"},
            "muted-foreground": {"color.text.muted"},
            "accent": {"color.hover.background"},
            "primary": {"color.accent"},
            "selection.background": {"color.selection.background"},
            "scrollbar.background": {"color.scrollbar.track"},
            "scrollbar.thumb.background": {"color.scrollbar.thumb"},
            "scrollbar.thumb.hover.background": {"color.scrollbar.thumb.hover"},
            "caret": {"color.text.primary"},
        }
        eq |= semantic_to_baseline.get(k, set())
        # Also accept the inverse direction in case code uses baseline but theme provides semantic.
        inverse: dict[str, set[str]] = {}
        for sem, baselines in semantic_to_baseline.items():
            for b in baselines:
                inverse.setdefault(b, set()).add(sem)
        eq |= inverse.get(k, set())
        return {_canonicalize_color_key(x) for x in eq}

    if kind == "metrics":
        k = key
        eq = {k}
        metric_aliases = {
            "font.size": {"metric.font.size"},
            "font.line_height": {"metric.font.line_height"},
            "mono_font.size": {"metric.font.mono_size"},
            "mono_font.line_height": {"metric.font.mono_line_height"},
            "radius": {"metric.radius.sm"},
            "radius.lg": {"metric.radius.md"},
        }
        eq |= metric_aliases.get(k, set())
        inverse: dict[str, set[str]] = {}
        for sem, baselines in metric_aliases.items():
            for b in baselines:
                inverse.setdefault(b, set()).add(sem)
        eq |= inverse.get(k, set())
        return eq

    return {key}


def _load_fret_ui_kit_theme_tokens(repo_root: Path) -> dict[str, str]:
    """
    Resolve `theme_tokens::metric::FOO` style constants used across shadcn ports.
    """
    path = repo_root / "ecosystem/fret-ui-kit/src/theme_tokens.rs"
    if not path.exists():
        return {}

    text = path.read_text(encoding="utf-8", errors="replace")
    const_re = re.compile(
        r"pub const\s+(?P<name>[A-Z0-9_]+)\s*:\s*&str\s*=\s*\"(?P<value>[^\"]+)\";"
    )

    by_module: dict[str, dict[str, str]] = {}
    current_mod: str | None = None
    for line in text.splitlines():
        m_mod = re.match(r"\s*pub mod\s+(?P<mod>[a-z0-9_]+)\s*\{", line)
        if m_mod:
            current_mod = m_mod.group("mod")
            by_module.setdefault(current_mod, {})
            continue
        m_const = const_re.search(line)
        if m_const and current_mod is not None:
            by_module[current_mod][m_const.group("name")] = m_const.group("value")

    resolved: dict[str, str] = {}
    for mod, consts in by_module.items():
        for name, value in consts.items():
            resolved[f"theme_tokens::{mod}::{name}"] = value
            resolved[f"fret_ui_kit::theme_tokens::{mod}::{name}"] = value
    return resolved


def _iter_rs_files(
    repo_root: Path, roots: Iterable[str], exclude_substrs: Iterable[str]
) -> Iterable[Path]:
    for raw in roots:
        root = (repo_root / raw).resolve()
        if not root.exists():
            continue
        for path in root.rglob("*.rs"):
            if path.is_file():
                rel = path.relative_to(repo_root).as_posix()
                if any(ex in rel for ex in exclude_substrs):
                    continue
                yield path


def _extract_required_tokens(
    repo_root: Path, roots: list[str], exclude_substrs: list[str]
) -> tuple[list[RequiredToken], list[str]]:
    token_consts = _load_fret_ui_kit_theme_tokens(repo_root)
    required: list[RequiredToken] = []
    unresolved: list[str] = []

    for path in _iter_rs_files(repo_root, roots, exclude_substrs):
        rel = path.relative_to(repo_root).as_posix()
        try:
            text = path.read_text(encoding="utf-8", errors="replace")
        except OSError as exc:
            unresolved.append(f"{rel}:0:{exc}")
            continue

        for line_no, line in enumerate(text.splitlines(), start=1):
            for m in _METHOD_RE.finditer(line):
                method = m.group("method")
                kind = _CALL_KIND.get(method)
                if kind is None:
                    continue
                arg = m.group("arg").strip()

                key: str | None = None
                m_raw = _RAW_STR_RE.search(arg)
                if m_raw:
                    key = m_raw.group("body")
                else:
                    m_str = _STR_RE.search(arg)
                    if m_str:
                        key = _unescape_rust_str(m_str.group("body"))
                    else:
                        m_path = _PATH_RE.match(arg)
                        if m_path:
                            p = m_path.group("path")
                            key = token_consts.get(p)
                            if key is None and "theme_tokens::" in p:
                                unresolved.append(f"{rel}:{line_no}:unresolved const {p}")
                        else:
                            unresolved.append(f"{rel}:{line_no}:unparsed arg {arg}")

                if key is None:
                    continue

                key = _canonicalize_required_token(kind, key)
                required.append(
                    RequiredToken(kind=kind, key=key, source=f"{rel}:{line_no}:{method}")
                )

    return required, unresolved


def _theme_maps(theme_json: dict) -> dict[str, set[str]]:
    out: dict[str, set[str]] = {}
    for kind in ("colors", "metrics", "corners", "numbers", "durations_ms", "easings", "text_styles"):
        raw = theme_json.get(kind, {})
        if isinstance(raw, dict):
            out[kind] = {str(k).strip() for k in raw.keys() if str(k).strip()}
        else:
            out[kind] = set()
    return out


def _extract_rust_fn_inserted_string_keys(text: str, fn_name: str) -> set[str]:
    """
    Best-effort extraction of `"key".to_string()` occurrences inside a specific function.
    """
    fn_re = re.compile(rf"\bfn\s+{re.escape(fn_name)}\b")
    m = fn_re.search(text)
    if not m:
        return set()

    i = m.start()
    brace = 0
    in_fn = False
    keys: set[str] = set()
    string_key_re = re.compile(r"\"(?P<k>[^\"]+)\"\.to_string\(\)")

    for line in text[i:].splitlines():
        if not in_fn:
            if "{" in line:
                brace += line.count("{") - line.count("}")
                in_fn = True
        else:
            brace += line.count("{") - line.count("}")
            for mm in string_key_re.finditer(line):
                k = mm.group("k").strip()
                if k:
                    keys.add(k)
            if brace <= 0:
                break
    return keys


def _load_default_theme_token_keys(repo_root: Path) -> dict[str, set[str]]:
    """
    Approximate the always-present default theme token keys by parsing fret-ui's theme module.

    This intentionally does not try to evaluate conditional logic; it just scrapes inserted token
    keys so we can avoid flagging "missing" tokens that are always seeded by default_theme().
    """
    path = repo_root / "crates/fret-ui/src/theme/mod.rs"
    if not path.exists():
        return {k: set() for k in _CALL_KIND.values()}

    text = path.read_text(encoding="utf-8", errors="replace")
    colors = _extract_rust_fn_inserted_string_keys(text, "default_color_tokens")
    metrics = _extract_rust_fn_inserted_string_keys(text, "default_metric_tokens")
    # The remaining kinds are not currently seeded in default_theme() via a helper function; keep
    # them empty to avoid hiding real gaps.
    return {
        "colors": {_canonicalize_color_key(k) for k in colors},
        "metrics": metrics,
        "corners": set(),
        "numbers": set(),
        "durations_ms": set(),
        "easings": set(),
        "text_styles": set(),
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--path",
        action="append",
        default=["crates", "ecosystem", "apps"],
        help="Search root (repeatable, repo-relative).",
    )
    parser.add_argument(
        "--exclude-substr",
        action="append",
        default=["/tests/", "\\tests\\"],
        help="Skip Rust files whose repo-relative path contains this substring (repeatable).",
    )
    parser.add_argument(
        "--theme",
        action="append",
        default=[
            "themes/fret-default-dark.json",
            "themes/godot-default-dark.json",
            "themes/hardhacker-dark.json",
        ],
        help="Theme JSON path to validate (repeatable, repo-relative).",
    )
    parser.add_argument(
        "--fail",
        action="store_true",
        help="Exit non-zero if any theme is missing required tokens.",
    )
    parser.add_argument(
        "--show-sources",
        action="store_true",
        help="Include one example source location per missing token.",
    )
    parser.add_argument(
        "--ignore-prefix",
        action="append",
        default=["missing.", "md.sys.", "chart.palette."],
        help="Ignore required tokens with this key prefix (repeatable).",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    required, unresolved = _extract_required_tokens(
        repo_root, args.path, args.exclude_substr
    )
    required = [
        t
        for t in required
        if not any(t.key.startswith(pfx) for pfx in args.ignore_prefix)
    ]

    default_keys = _load_default_theme_token_keys(repo_root)

    required_by_kind: dict[str, dict[str, str]] = {}
    for t in required:
        required_by_kind.setdefault(t.kind, {})
        required_by_kind[t.kind].setdefault(t.key, t.source)

    if unresolved:
        print("[WARN] unresolved token arguments (best-effort scan):", file=sys.stderr)
        for u in unresolved[:200]:
            print(f"  {u}", file=sys.stderr)
        if len(unresolved) > 200:
            print(f"  ... ({len(unresolved) - 200} more)", file=sys.stderr)

    any_missing = False
    for raw_theme in args.theme:
        theme_path = (repo_root / raw_theme).resolve()
        if not theme_path.exists():
            print(f"[FAIL] theme not found: {raw_theme}", file=sys.stderr)
            any_missing = True
            continue

        theme_json = _load_json(theme_path)
        available = _theme_maps(theme_json)

        # Union in always-present default keys, and canonicalize colors for matching.
        for kind in available.keys():
            available[kind] |= default_keys.get(kind, set())
        available["colors"] = {_canonicalize_color_key(k) for k in available["colors"]}

        missing: dict[str, list[str]] = {}
        for kind, required_map in sorted(required_by_kind.items()):
            missing_keys: list[str] = []
            for key in sorted(required_map.keys()):
                eq = _theme_token_equivalents(kind, key)
                if kind == "colors":
                    eq = {_canonicalize_color_key(k) for k in eq}
                if not (eq & available.get(kind, set())):
                    missing_keys.append(key)
            if missing_keys:
                missing[kind] = missing_keys

        if missing:
            any_missing = True
            print(f"[FAIL] {theme_path.relative_to(repo_root).as_posix()}", file=sys.stderr)
            for kind, keys in missing.items():
                print(f"  missing {kind}:", file=sys.stderr)
                for k in keys[:200]:
                    if args.show_sources:
                        src = required_by_kind[kind].get(k, "?")
                        print(f"    - {k}  ({src})", file=sys.stderr)
                    else:
                        print(f"    - {k}", file=sys.stderr)
                if len(keys) > 200:
                    print(f"    ... ({len(keys) - 200} more)", file=sys.stderr)
        else:
            print(f"[OK]   {theme_path.relative_to(repo_root).as_posix()}")

    return 1 if (args.fail and any_missing) else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
