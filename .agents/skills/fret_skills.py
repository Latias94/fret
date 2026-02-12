#!/usr/bin/env python3
"""
Fret Skills CLI

Cross-platform tool to manage the repo-local Agent Skills under `.agents/skills/`.

Goals:
- One Python entrypoint for listing, installing, validating, and packaging skills.
- No third-party dependencies (stdlib only).
- Friendly to both framework developers and app developers.

Note: This script is intentionally kept in `.agents/skills/` so it can be shipped with
skills-only bundles without requiring the full Fret repo.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import sys
from collections.abc import Iterable
from dataclasses import dataclass
from pathlib import Path
from typing import NoReturn
from zipfile import ZIP_DEFLATED, ZipFile


SKILL_DIR_PREFIX = "fret-"
SKILL_FILE_NAME = "SKILL.md"

RECOMMENDED_HEADINGS = (
    "## When to use",
    "## Quick start",
    "## Workflow",
    "## Evidence anchors",
    "## Common pitfalls",
    "## Related skills",
)

NAME_REGEX = re.compile(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")

# Evidence checks are intended for maintainers running inside the Fret mono-repo.
# Keep the list small and high-signal (avoid brittle exact-line matches).
SYMBOL_CHECKS: dict[str, list[tuple[str, str]]] = {
    # skill_name: [(relative_file_path, regex_pattern)]
    "fret-app-ui-builder": [
        (".agents/skills/fret-app-ui-builder/scripts/stylegen.py", r""),
        (".agents/skills/fret-app-ui-builder/references/recipes/INDEX.md", r""),
    ],
    "fret-diag-workflow": [
        ("apps/fretboard/src/cli.rs", r"\bfretboard\s+diag\s+run\b"),
        ("crates/fret-diag-protocol/src/lib.rs", r"\bClickStable\b"),
        ("crates/fret-diag-protocol/src/lib.rs", r"\bEnsureVisible\b"),
        ("crates/fret-diag-protocol/src/lib.rs", r"\bTypeTextInto\b"),
        ("crates/fret-diag-protocol/src/lib.rs", r"\bMenuSelectPath\b"),
        ("crates/fret-diag-protocol/src/lib.rs", r"\bselector_resolution_trace\b"),
        ("crates/fret-diag-protocol/src/lib.rs", r"\bhit_test_trace\b"),
        ("crates/fret-diag-protocol/src/lib.rs", r"\bfocus_trace\b"),
        ("crates/fret-diag-protocol/src/lib.rs", r"\boverlay_placement_trace\b"),
        ("ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs", r"diag\.script_v2"),
        ("ecosystem/fret-bootstrap/src/ui_diagnostics_ws_bridge.rs", r"diag\.screenshot_png"),
        ("ecosystem/fret-bootstrap/src/ui_diagnostics.rs", r"\bFRET_DIAG_DEBUG_CLICK_STABLE\b"),
        ("crates/fret-diag/src/lib.rs", r"\bFRET_DIAG_FIXED_FRAME_DELTA_MS\b"),
        (".agents/skills/fret-diag-workflow/scripts/triage_perf_gate.py", r""),
    ],
    "fret-framework-maintainer-guide": [
        ("docs/adr/IMPLEMENTATION_ALIGNMENT.md", r""),
        ("docs/dependency-policy.md", r""),
    ],
    "fret-repo-orientation": [
        ("docs/architecture.md", r""),
        ("docs/runtime-contract-matrix.md", r""),
    ],
}


def _info(msg: str) -> None:
    print(f"[INFO] {msg}")


def _warn(msg: str) -> None:
    print(f"[WARN] {msg}", file=sys.stderr)


def _ok(msg: str) -> None:
    print(f"[OK]   {msg}")


def _die(msg: str) -> NoReturn:
    print(f"[ERROR] {msg}", file=sys.stderr)
    raise SystemExit(1)


def _read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def _write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def _normalize_agent(agent: str) -> str:
    a = agent.strip().lower()
    if a in ("claude", "claude-code"):
        return "claude-code"
    if a in ("codex",):
        return "codex"
    if a in ("gemini",):
        return "gemini"
    _die(f"Unknown agent: {agent} (expected claude-code, codex, gemini)")


def _skills_dest_dir(agent: str, target_abs: Path) -> Path:
    if agent == "codex":
        return target_abs / ".agents" / "skills"
    if agent == "gemini":
        return target_abs / ".gemini" / "skills"
    if agent == "claude-code":
        return target_abs / ".claude" / "skills"
    _die(f"Unknown agent: {agent}")


def _skills_root(default_root: Path | None) -> Path:
    # Default: directory containing this script (i.e. `.agents/skills/`)
    if default_root is None:
        return Path(__file__).resolve().parent
    return default_root.expanduser().resolve()


def _iter_skill_dirs(root: Path) -> list[Path]:
    out: list[Path] = []
    if not root.is_dir():
        return out
    for entry in sorted(root.iterdir(), key=lambda p: p.name):
        if not entry.is_dir():
            continue
        if not entry.name.startswith(SKILL_DIR_PREFIX):
            continue
        if not (entry / SKILL_FILE_NAME).is_file():
            continue
        out.append(entry)
    return out


def _split_csv(values: list[str]) -> list[str]:
    out: list[str] = []
    for v in values:
        for part in v.split(","):
            p = part.strip()
            if p:
                out.append(p)
    return out


@dataclass(frozen=True)
class Frontmatter:
    name: str | None
    description: str | None


def _unquote(s: str) -> str:
    s = s.strip()
    if (s.startswith('"') and s.endswith('"')) or (s.startswith("'") and s.endswith("'")):
        return s[1:-1]
    return s


def _parse_frontmatter(frontmatter_lines: list[str]) -> Frontmatter:
    name: str | None = None
    description: str | None = None

    i = 0
    while i < len(frontmatter_lines):
        line = frontmatter_lines[i]

        m = re.match(r"^\s*name:\s*(.+)\s*$", line)
        if m:
            name = _unquote(m.group(1))
            i += 1
            continue

        m = re.match(r"^\s*description:\s*(.+)\s*$", line)
        if m:
            raw = m.group(1).strip()
            if raw in ("|", ">"):
                block: list[str] = []
                j = i + 1
                while j < len(frontmatter_lines):
                    l = frontmatter_lines[j]
                    # Stop at the next top-level key.
                    if re.match(r"^\s*[\w-]+\s*:\s*", l):
                        break
                    block.append(re.sub(r"^\s{0,2}", "", l))
                    j += 1
                description = "\n".join(block).strip()
                i = j
                continue
            description = _unquote(raw)
            i += 1
            continue

        i += 1

    return Frontmatter(name=name, description=description)


@dataclass(frozen=True)
class ValidationProblem:
    kind: str  # "error" | "warning"
    skill: str
    message: str


def _infer_repo_root(skills_root: Path) -> Path | None:
    """
    Best-effort repo root inference.

    - In the Fret mono-repo, skills root is `<repo>/.agents/skills`, so repo root is `skills_root/../..`.
    - Otherwise, walk upward and look for `Cargo.toml` + `crates/` as a heuristic.
    """
    if skills_root.name == "skills" and skills_root.parent.name == ".agents":
        return skills_root.parent.parent

    cur = skills_root
    for _ in range(6):
        cargo = cur / "Cargo.toml"
        crates = cur / "crates"
        if cargo.is_file() and crates.is_dir():
            return cur
        cur = cur.parent
    return None


def _iter_code_spans(markdown: str) -> Iterable[str]:
    # Keep it simple: skills use single-backtick spans for paths and commands.
    for m in re.finditer(r"`([^`\n]+)`", markdown):
        yield m.group(1).strip()


def _looks_like_repo_path(s: str) -> bool:
    if not s or " " in s:
        return False
    # Wildcards/globs are allowed in docs; they are not stable "path exists" anchors.
    if any(ch in s for ch in ("*", "?", "[", "]", "{", "}")):
        return False
    if s.startswith(("http://", "https://")):
        return False
    if "<" in s or ">" in s:
        return False
    if s.startswith(("cargo ", "python ", "python3 ", "rg ", "jq ", "awk ", "pwsh ", "bash ")):
        return False
    if s.startswith(("~", "/", "\\")):
        return False
    if s.startswith(("target/", ".fret/", ".venv", ".venv/")):
        return False
    # Only check explicit repo-relative anchors to reduce false positives.
    allowed_prefixes = (
        "docs/",
        "crates/",
        "ecosystem/",
        "apps/",
        "tools/",
        ".agents/",
        ".github/",
        "goldens/",
        "themes/",
        "assets/",
    )
    if not s.startswith(allowed_prefixes):
        return False
    # Avoid "directory anchors" without a specific file.
    if s.endswith("/"):
        return False
    # Heuristic: must look like a file path.
    return any(s.endswith(ext) for ext in (".md", ".rs", ".toml", ".json", ".yml", ".yaml", ".py", ".sh", ".ps1"))


def _validate_skill_dir(skill_dir: Path, strict_headings: bool) -> list[ValidationProblem]:
    problems: list[ValidationProblem] = []
    skill_name = skill_dir.name
    skill_file = skill_dir / SKILL_FILE_NAME

    if not skill_file.exists():
        problems.append(ValidationProblem("error", skill_name, "Missing SKILL.md"))
        return problems

    text = _read_text(skill_file)
    lines = text.splitlines()
    if len(lines) < 3 or lines[0].strip() != "---":
        problems.append(ValidationProblem("error", skill_name, "Invalid frontmatter (missing leading ---)"))
        return problems

    end_index = None
    for idx in range(1, len(lines)):
        if lines[idx].strip() == "---":
            end_index = idx
            break

    if end_index is None:
        problems.append(ValidationProblem("error", skill_name, "Invalid frontmatter (missing closing ---)"))
        return problems

    fm_lines = lines[1:end_index]
    fm = _parse_frontmatter(fm_lines)

    if not fm.name or not fm.name.strip():
        problems.append(ValidationProblem("error", skill_name, "Frontmatter missing name"))
    else:
        if fm.name != skill_name:
            problems.append(
                ValidationProblem("error", skill_name, f"Skill name mismatch: dir='{skill_name}' frontmatter.name='{fm.name}'")
            )
        if len(fm.name) > 64:
            problems.append(ValidationProblem("error", skill_name, "Skill name too long (>64)"))
        if not NAME_REGEX.match(fm.name):
            problems.append(ValidationProblem("error", skill_name, "Skill name invalid (expected lowercase-hyphen)"))

    if not fm.description or not fm.description.strip():
        problems.append(ValidationProblem("error", skill_name, "Frontmatter missing description"))
    else:
        if len(fm.description) > 1024:
            problems.append(ValidationProblem("error", skill_name, "Description too long (>1024 chars)"))

    body = "\n".join(lines[end_index + 1 :])
    for h in RECOMMENDED_HEADINGS:
        if h not in body:
            kind = "error" if strict_headings else "warning"
            problems.append(ValidationProblem(kind, skill_name, f"Missing recommended heading: {h}"))

    return problems


def _validate_anchors(skill_dir: Path, repo_root: Path, strict: bool) -> list[ValidationProblem]:
    skill_name = skill_dir.name
    skill_file = skill_dir / SKILL_FILE_NAME
    problems: list[ValidationProblem] = []

    text = _read_text(skill_file)
    for span in _iter_code_spans(text):
        if not _looks_like_repo_path(span):
            continue

        # repo-ref is explicitly optional for GitHub checkouts.
        if span.startswith("repo-ref/"):
            if not (repo_root / span).exists():
                problems.append(
                    ValidationProblem(
                        "warning",
                        skill_name,
                        f"Optional anchor missing (repo-ref not present): `{span}`",
                    )
                )
            continue

        p = repo_root / span
        if not p.exists():
            kind = "error" if strict else "warning"
            problems.append(ValidationProblem(kind, skill_name, f"Anchor path missing: `{span}`"))

    return problems


def _validate_symbols(skills: list[Path], repo_root: Path) -> list[ValidationProblem]:
    problems: list[ValidationProblem] = []
    available = {s.name for s in skills}

    for skill_name, checks in SYMBOL_CHECKS.items():
        if skill_name not in available:
            continue
        for rel_path, pat in checks:
            p = repo_root / rel_path
            if not p.exists():
                problems.append(ValidationProblem("error", skill_name, f"Evidence file missing: `{rel_path}`"))
                continue
            if not pat:
                continue
            try:
                content = _read_text(p)
            except Exception:
                problems.append(ValidationProblem("error", skill_name, f"Failed to read evidence file: `{rel_path}`"))
                continue
            if re.search(pat, content) is None:
                problems.append(
                    ValidationProblem(
                        "error",
                        skill_name,
                        f"Evidence symbol not found in `{rel_path}`: /{pat}/",
                    )
                )

    return problems


def cmd_list(args: argparse.Namespace) -> int:
    root = _skills_root(Path(args.root) if args.root else None)
    skills = _iter_skill_dirs(root)
    fmt = getattr(args, "format", "text")
    with_desc = bool(getattr(args, "with_descriptions", False))

    items: list[dict[str, str | None]] = []
    for s in skills:
        if with_desc or fmt == "json":
            skill_file = s / SKILL_FILE_NAME
            text = _read_text(skill_file)
            lines = text.splitlines()
            end_index = None
            for idx in range(1, len(lines)):
                if lines[idx].strip() == "---":
                    end_index = idx
                    break
            fm = _parse_frontmatter(lines[1:end_index] if end_index is not None else [])
            items.append({"name": s.name, "description": fm.description})
        else:
            items.append({"name": s.name, "description": None})

    if fmt == "json":
        print(json.dumps(items, indent=2, ensure_ascii=False))
        return 0

    for it in items:
        if with_desc:
            desc = (it.get("description") or "").strip()
            if desc:
                print(f"{it['name']} - {desc}")
            else:
                print(it["name"])
        else:
            print(it["name"])
    return 0


def cmd_validate(args: argparse.Namespace) -> int:
    root = _skills_root(Path(args.root) if args.root else None)
    strict = bool(args.strict)
    strict_headings = bool(args.strict_headings) if args.strict_headings is not None else strict
    check_anchors = bool(args.check_anchors)
    check_symbols = bool(args.check_symbols)
    repo_root = Path(args.repo_root).expanduser().resolve() if args.repo_root else _infer_repo_root(root)
    fmt = args.format

    skills = _iter_skill_dirs(root)
    if not skills:
        _warn(f"No skills found under {root}")
        return 0

    problems: list[ValidationProblem] = []
    for s in skills:
        problems.extend(_validate_skill_dir(s, strict_headings=strict_headings))

    if check_anchors or check_symbols:
        if repo_root is None:
            _die("Cannot infer repo root for anchor/symbol checks. Pass --repo-root <path>.")
        if not repo_root.exists():
            _die(f"--repo-root does not exist: {repo_root}")

    if check_anchors and repo_root is not None:
        for s in skills:
            problems.extend(_validate_anchors(s, repo_root=repo_root, strict=strict))

    if check_symbols and repo_root is not None:
        problems.extend(_validate_symbols(skills, repo_root=repo_root))

    errors = [p for p in problems if p.kind == "error"]
    warnings = [p for p in problems if p.kind == "warning"]

    if fmt == "json":
        payload = {
            "root": str(root),
            "skills_checked": len(skills),
            "errors": [{"skill": p.skill, "message": p.message} for p in errors],
            "warnings": [{"skill": p.skill, "message": p.message} for p in warnings],
        }
        print(json.dumps(payload, indent=2, ensure_ascii=False))
    else:
        if errors:
            print("Skill validation errors:", file=sys.stderr)
            for p in errors:
                print(f"  - [{p.skill}] {p.message}", file=sys.stderr)
        if warnings:
            print("Skill validation warnings:", file=sys.stderr)
            for p in warnings:
                print(f"  - [{p.skill}] {p.message}", file=sys.stderr)
        _ok(f"Skills checked: {len(skills)}")

    if errors:
        return 1
    if strict and warnings:
        return 1
    return 0


def cmd_install(args: argparse.Namespace) -> int:
    root = _skills_root(Path(args.root) if args.root else None)
    agent = _normalize_agent(args.agent)
    target_abs = Path(args.target).expanduser().resolve()
    dest_dir = _skills_dest_dir(agent, target_abs)
    force = bool(args.force)
    dry_run = bool(args.dry_run)

    available_dirs = _iter_skill_dirs(root)
    available = [p.name for p in available_dirs]
    if not available:
        _die(f"No skills found under {root} (expected {SKILL_DIR_PREFIX}*/{SKILL_FILE_NAME})")

    if args.profile and args.skills:
        _die("Use either --profile or --skills (not both)")

    if args.profile:
        profiles = _load_profiles(root)
        if args.profile not in profiles:
            _die(f"Unknown profile: {args.profile} (known: {', '.join(sorted(profiles.keys())) or '<none>'})")
        requested = profiles[args.profile]
    elif args.skills:
        requested = _split_csv(args.skills)
    else:
        requested = list(available)

    missing = [s for s in requested if s not in available]
    if missing:
        _die(f"Unknown skill(s): {', '.join(missing)} (use `list` to see available skills)")

    _info(f"Source: {root}")
    _info(f"Target: {target_abs}")
    _info(f"Agent:  {agent}")
    _info(f"Dest:   {dest_dir}")
    if args.profile:
        _info(f"Profile:{args.profile}")
    _info(f"Skills: {', '.join(requested)}")
    if dry_run:
        _info("Dry run: no files will be changed")

    if not dest_dir.exists():
        if dry_run:
            _info(f"Dry run: would create {dest_dir}")
        else:
            dest_dir.mkdir(parents=True, exist_ok=True)

    for name in requested:
        src = root / name
        dst = dest_dir / name

        if dst.exists():
            if force:
                if dry_run:
                    _warn(f"Dry run: would remove existing {dst}")
                else:
                    shutil.rmtree(dst)
            else:
                _warn(f"Skip (already exists): {dst} (use --force to overwrite)")
                continue

        if dry_run:
            _info(f"Dry run: would copy {src} -> {dst}")
        else:
            shutil.copytree(src, dst)
            _ok(f"Installed: {name}")

    _ok("Done.")
    return 0


def _load_profiles(root: Path) -> dict[str, list[str]]:
    meta = root / "metadata.json"
    if not meta.exists():
        return {}
    try:
        payload = json.loads(_read_text(meta))
    except Exception:
        return {}
    profiles = payload.get("profiles", {})
    if not isinstance(profiles, dict):
        return {}
    out: dict[str, list[str]] = {}
    for k, v in profiles.items():
        if isinstance(k, str) and isinstance(v, list) and all(isinstance(x, str) for x in v):
            out[k] = list(v)
    return out


def _zip_dir(src_dir: Path, zip_path: Path) -> None:
    zip_path.parent.mkdir(parents=True, exist_ok=True)
    with ZipFile(zip_path, "w", compression=ZIP_DEFLATED) as zf:
        for path in sorted(src_dir.rglob("*")):
            if path.is_dir():
                continue
            rel = path.relative_to(src_dir)
            zf.write(path, rel.as_posix())


def cmd_package(args: argparse.Namespace) -> int:
    root = _skills_root(Path(args.root) if args.root else None)
    out_dir = Path(args.out).expanduser().resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    available = [p.name for p in _iter_skill_dirs(root)]
    if not available:
        _die(f"No skills found under {root}")

    selected: list[str]
    if args.profile:
        profiles = _load_profiles(root)
        if args.profile not in profiles:
            _die(f"Unknown profile: {args.profile} (known: {', '.join(sorted(profiles.keys())) or '<none>'})")
        selected = profiles[args.profile]
    elif args.skills:
        selected = _split_csv(args.skills)
    else:
        selected = list(available)

    missing = [s for s in selected if s not in available]
    if missing:
        _die(f"Profile/selection references missing skills: {', '.join(missing)}")

    stage = out_dir / "skills"
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True, exist_ok=True)

    # Ship the management script + metadata with the bundle for convenience.
    shutil.copy2(root / "fret_skills.py", stage / "fret_skills.py")
    meta = root / "metadata.json"
    if meta.exists():
        shutil.copy2(meta, stage / "metadata.json")

    for name in selected:
        shutil.copytree(root / name, stage / name)

    profile_tag = args.profile or "custom"
    bundle_name = args.bundle_name or f"fret-skills-{profile_tag}"
    bundle_zip = out_dir / f"{bundle_name}.zip"
    _zip_dir(stage, bundle_zip)

    profile_install_block = ""
    if args.profile:
        profile_install_block = f"""
If this bundle was built from a profile, install that profile explicitly:

```bash
python3 fret_skills.py install --agent codex --target <project> --profile {args.profile} --force
```
"""

    manifest = {
        "bundle": bundle_name,
        "profile": args.profile,
        "skills": selected,
        "source_root": str(root),
    }
    _write_text(out_dir / "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")

    readme = f"""# {bundle_name}

This bundle contains Fret Agent Skills (directories with `{SKILL_FILE_NAME}`).

## Install

Recommended (cross-platform, preserves updates cleanly):

```bash
# From the extracted bundle directory:
python3 fret_skills.py install --agent codex --target <project> --force
```

{profile_install_block}

Notes:
- `validate --check-anchors/--check-symbols` requires a full Fret source checkout (mono-repo). Bundles are skills-only.

Manual alternative (copy folders directly):

- Codex CLI: `<project>/.agents/skills/`
- Claude Code: `<project>/.claude/skills/`
- Gemini CLI: `<project>/.gemini/skills/`

If you have the Fret repo checkout available, you can also install via:

```bash
python3 .agents/skills/fret_skills.py install --agent codex --target <project> --force
```
"""
    _write_text(out_dir / "README.md", readme)

    _ok(f"Packaged {len(selected)} skills into {bundle_zip}")
    return 0


def cmd_prompt(args: argparse.Namespace) -> int:
    """
    Generate an <available_skills> XML block similar to the Agent Skills reference tool.
    """
    root = _skills_root(Path(args.root) if args.root else None)
    skills = _iter_skill_dirs(root)
    if not skills:
        _die(f"No skills found under {root}")

    items: list[tuple[str, str, str]] = []
    for s in skills:
        skill_md = s / SKILL_FILE_NAME
        text = _read_text(skill_md)
        lines = text.splitlines()
        if not lines or lines[0].strip() != "---":
            continue
        end_index = None
        for idx in range(1, len(lines)):
            if lines[idx].strip() == "---":
                end_index = idx
                break
        if end_index is None:
            continue
        fm = _parse_frontmatter(lines[1:end_index])
        if not fm.name or not fm.description:
            continue
        items.append((fm.name, fm.description, str(skill_md)))

    # Keep the output simple and stable (no pretty-print dependencies).
    print("<available_skills>")
    for name, desc, loc in items:
        print("<skill>")
        print("<name>")
        print(name)
        print("</name>")
        print("<description>")
        print(desc)
        print("</description>")
        print("<location>")
        print(loc)
        print("</location>")
        print("</skill>")
    print("</available_skills>")
    return 0


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(prog="fret_skills.py")
    parser.add_argument("--root", help="Skills root directory (default: directory containing this script)")

    sub = parser.add_subparsers(dest="cmd", required=True)

    p_list = sub.add_parser("list", help="List available skills")
    p_list.add_argument(
        "--with-descriptions",
        action="store_true",
        help="Include each skill's frontmatter description (more helpful for humans; names-only is the default)",
    )
    p_list.add_argument("--format", choices=["text", "json"], default="text")
    p_list.set_defaults(fn=cmd_list)

    p_validate = sub.add_parser("validate", help="Validate skills (frontmatter + recommended headings)")
    p_validate.add_argument("--strict", action="store_true", help="Treat warnings as errors")
    p_validate.add_argument(
        "--strict-headings",
        action="store_true",
        default=None,
        help="Treat missing recommended headings as errors (defaults to --strict)",
    )
    p_validate.add_argument(
        "--check-anchors",
        action="store_true",
        help="Validate repo-relative anchor paths referenced in SKILL.md backticks (maintainer mode)",
    )
    p_validate.add_argument(
        "--check-symbols",
        action="store_true",
        help="Validate a small set of high-signal evidence symbols exist in source files (maintainer mode)",
    )
    p_validate.add_argument(
        "--repo-root",
        help="Repo root for --check-anchors/--check-symbols (default: inferred from --root)",
    )
    p_validate.add_argument("--format", choices=["text", "json"], default="text")
    p_validate.set_defaults(fn=cmd_validate)

    p_install = sub.add_parser("install", help="Install skills into a target project for a specific agent")
    p_install.add_argument("--agent", default="claude-code", help="claude-code|codex|gemini")
    p_install.add_argument("--target", default=".", help="Target project directory")
    p_install.add_argument(
        "--profile", help="Profile name from metadata.json (e.g. consumer-app-dev, framework-dev)"
    )
    p_install.add_argument("--skills", action="append", default=[], help="Comma-separated skill names (repeatable)")
    p_install.add_argument("--force", action="store_true", help="Overwrite existing installed skills")
    p_install.add_argument("--dry-run", action="store_true", help="Print actions without changing files")
    p_install.set_defaults(fn=cmd_install)

    p_package = sub.add_parser("package", help="Package a skills bundle (for releases or distribution)")
    p_package.add_argument("--out", required=True, help="Output directory (will be created)")
    p_package.add_argument("--profile", help="Profile name from metadata.json (e.g. consumer-app-dev, framework-dev)")
    p_package.add_argument("--skills", action="append", default=[], help="Comma-separated skill names (repeatable)")
    p_package.add_argument("--bundle-name", help="Override default bundle name")
    p_package.set_defaults(fn=cmd_package)

    p_prompt = sub.add_parser("prompt", help="Generate <available_skills> XML for agent prompts")
    p_prompt.set_defaults(fn=cmd_prompt)

    args = parser.parse_args(argv)
    return int(args.fn(args))


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
