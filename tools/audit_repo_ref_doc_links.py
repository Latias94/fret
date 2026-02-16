#!/usr/bin/env python3
"""
Audit docs for `repo-ref/<name>` anchors and whether each doc provides an upstream GitHub link.

Rationale:
- `repo-ref/` is intentionally ignored by git (local state).
- Public-facing docs should still be actionable for readers without local checkouts.

This script does not modify files. It writes a markdown report under `docs/audits/`.
"""

from __future__ import annotations

import re
import subprocess
import sys
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path


def _repo_root() -> Path:
    out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"])
    return Path(out.decode("utf-8").strip()).resolve()


RE_REPO_REF = re.compile(r"repo-ref/([A-Za-z0-9_.-]+)")
RE_GITHUB = re.compile(r"https?://github\.com/[^\s)>\"]+")


@dataclass(frozen=True)
class DocHit:
    path: Path
    repo_names: tuple[str, ...]
    has_github_link: bool


def main() -> int:
    try:
        root = _repo_root()
    except Exception as e:
        print(f"error: failed to locate repo root: {e}", file=sys.stderr)
        return 2

    docs_dir = root / "docs"
    if not docs_dir.is_dir():
        print("error: missing docs/ directory", file=sys.stderr)
        return 2

    hits: list[DocHit] = []

    for md in sorted(docs_dir.rglob("*.md")):
        text = md.read_text(encoding="utf-8", errors="replace")
        names = sorted(set(RE_REPO_REF.findall(text)))
        if not names:
            continue
        hits.append(
            DocHit(
                path=md.relative_to(root),
                repo_names=tuple(names),
                has_github_link=RE_GITHUB.search(text) is not None,
            )
        )

    report_path = root / "docs/audits/repo-ref-doc-links-audit.md"
    report_path.parent.mkdir(parents=True, exist_ok=True)

    by_repo: dict[str, list[DocHit]] = defaultdict(list)
    for h in hits:
        for n in h.repo_names:
            by_repo[n].append(h)

    lines: list[str] = []
    lines.append("# repo-ref doc link audit")
    lines.append("")
    lines.append("This report lists docs that reference `repo-ref/<name>` and whether they also include a GitHub link.")
    lines.append("`repo-ref/` is local state (ignored by git), so public docs should ideally include upstream URLs.")
    lines.append("")
    lines.append(f"- Total docs with `repo-ref/` anchors: **{len(hits)}**")
    lines.append("")
    lines.append("## By document")
    lines.append("")
    for h in hits:
        status = "OK" if h.has_github_link else "Missing GitHub link"
        names = ", ".join(h.repo_names)
        lines.append(f"- `{h.path.as_posix()}`: {status} (repos: {names})")
    lines.append("")
    lines.append("## By repo name")
    lines.append("")
    for repo_name in sorted(by_repo.keys()):
        docs = by_repo[repo_name]
        missing = sum(1 for d in docs if not d.has_github_link)
        lines.append(f"- `{repo_name}`: {len(docs)} docs ({missing} missing GitHub link)")

    report_path.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote: {report_path.relative_to(root).as_posix()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

