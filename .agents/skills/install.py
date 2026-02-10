#!/usr/bin/env python3
"""
Fret skills installer (repo-local source -> project-local agent directory).

Cross-platform replacement for `.agents/skills/install.ps1`.

This copies skill folders (each containing `SKILL.md`) from this repo's `.agents/skills/`
directory into a target project's agent skills directory:

- Claude Code: <project>/.claude/skills/
- Codex CLI:   <project>/.agents/skills/
- Gemini CLI:  <project>/.gemini/skills/
"""

from __future__ import annotations

import argparse
import os
import shutil
import sys
from pathlib import Path


def _info(msg: str) -> None:
    print(f"[INFO] {msg}")


def _warn(msg: str) -> None:
    print(f"[WARN] {msg}")


def _ok(msg: str) -> None:
    print(f"[OK]   {msg}")


def _die(msg: str) -> "NoReturn":
    print(f"[ERROR] {msg}", file=sys.stderr)
    raise SystemExit(1)


def _normalize_agent(agent: str) -> str:
    a = agent.strip().lower()
    if a in ("claude", "claude-code"):
        return "claude"
    if a == "codex":
        return "codex"
    if a == "gemini":
        return "gemini"
    _die(f"Unknown agent: {agent}")


def _skills_dest_dir(agent: str, target_abs: Path) -> Path:
    if agent == "codex":
        return target_abs / ".agents" / "skills"
    if agent == "gemini":
        return target_abs / ".gemini" / "skills"
    if agent == "claude":
        return target_abs / ".claude" / "skills"
    _die(f"Unknown agent: {agent}")


def _available_skills(source_root: Path) -> list[str]:
    out: list[str] = []
    if not source_root.is_dir():
        return out
    for entry in sorted(source_root.iterdir(), key=lambda p: p.name):
        if not entry.is_dir():
            continue
        if not entry.name.startswith("fret-"):
            continue
        if not (entry / "SKILL.md").is_file():
            continue
        out.append(entry.name)
    return out


def _split_skills(values: list[str]) -> list[str]:
    out: list[str] = []
    for v in values:
        for part in v.split(","):
            p = part.strip()
            if p:
                out.append(p)
    return out


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--agent", default="claude", choices=["claude", "claude-code", "codex", "gemini"])
    parser.add_argument("--target", default=".")
    parser.add_argument("--skills", action="append", default=[], help="Comma-separated skill names (repeatable).")
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args(argv)

    source_root = Path(__file__).parent.resolve()
    target_abs = Path(args.target).expanduser().resolve()
    agent_norm = _normalize_agent(args.agent)
    dest_dir = _skills_dest_dir(agent_norm, target_abs)

    if not source_root.exists():
        _die(f"Cannot find skills source directory: {source_root}")

    available = _available_skills(source_root)
    if args.list:
        _info(f"Available skills in {source_root}:")
        for s in available:
            print(f"  - {s}")
        return 0

    if not available:
        _die(f"No skills found under {source_root} (expected folders like fret-*/SKILL.md)")

    requested = _split_skills(args.skills) if args.skills else list(available)
    missing = [s for s in requested if s not in available]
    if missing:
        _die(f"Unknown skill(s): {', '.join(missing)}. Use --list to see available skills.")

    _info(f"Source: {source_root}")
    _info(f"Target: {target_abs}")
    _info(f"Agent:  {agent_norm}")
    _info(f"Dest:   {dest_dir}")
    _info(f"Skills: {', '.join(requested)}")

    if not dest_dir.exists():
        if args.dry_run:
            _info(f"Dry run: would create {dest_dir}")
        else:
            dest_dir.mkdir(parents=True, exist_ok=True)

    for name in requested:
        src = source_root / name
        dst = dest_dir / name

        if dst.exists():
            if args.force:
                if args.dry_run:
                    _warn(f"Dry run: would remove existing {dst}")
                else:
                    shutil.rmtree(dst)
            else:
                _warn(f"Skip (already exists): {dst} (use --force to overwrite)")
                continue

        if args.dry_run:
            _info(f"Dry run: would copy {src} -> {dst}")
        else:
            shutil.copytree(src, dst)
            _ok(f"Installed: {name}")

    _ok("Done.")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except BrokenPipeError:
        os._exit(0)
