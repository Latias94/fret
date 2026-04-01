#!/usr/bin/env python3
"""Run cargo publish dry-runs for a configured release wave and write a report."""

from __future__ import annotations

import argparse
import subprocess
import sys
from dataclasses import dataclass
from datetime import date
from pathlib import Path


@dataclass
class DryRunResult:
    crate: str
    command: list[str]
    returncode: int
    stdout: str
    stderr: str

    @property
    def passed(self) -> bool:
        return self.returncode == 0

    @property
    def summary(self) -> str:
        if self.passed:
            return "passed"
        if _looks_like_registry_gap(self.stderr):
            return "failed (awaiting earlier waves on crates.io)"
        return "failed (unexpected packaging blocker)"


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _looks_like_registry_gap(stderr: str) -> bool:
    markers = (
        "no matching package named",
        "failed to select a version for the requirement",
        "is not found in the package registry",
        "perhaps a crate was updated and forgotten to be re-vendored",
    )
    lowered = stderr.lower()
    return any(marker in lowered for marker in markers)


def _parse_waves(path: Path) -> dict[int, list[str]]:
    waves: dict[int, list[str]] = {}
    current_wave: int | None = None

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line.startswith("## Wave "):
            head = line.split("(", 1)[0]
            current_wave = int(head.removeprefix("## Wave ").strip())
            waves[current_wave] = []
            continue
        if current_wave is not None and line.startswith("- "):
            waves[current_wave].append(line.removeprefix("- ").strip())

    return waves


def _run_dry_run(repo_root: Path, crate: str) -> DryRunResult:
    command = ["cargo", "publish", "--dry-run", "-p", crate]
    proc = subprocess.run(
        command,
        cwd=repo_root,
        text=True,
        capture_output=True,
        check=False,
    )
    return DryRunResult(
        crate=crate,
        command=command,
        returncode=proc.returncode,
        stdout=_sanitize_output(proc.stdout, repo_root),
        stderr=_sanitize_output(proc.stderr, repo_root),
    )


def _tail(text: str, lines: int = 20) -> str:
    stripped = text.strip()
    if not stripped:
        return ""
    return "\n".join(stripped.splitlines()[-lines:])


def _sanitize_output(text: str, repo_root: Path) -> str:
    return text.replace(str(repo_root), ".")


def _report_for_wave(wave: int, results: list[DryRunResult]) -> str:
    lines: list[str] = [
        f"# Fret v0.1.0 Wave {wave} dry-run status",
        "",
        f"Date verified: {date.today().isoformat()}.",
        "",
        "Commands run:",
        "",
    ]

    for result in results:
        lines.append(f"- `{' '.join(result.command)}`")

    lines.extend(["", "Status:", ""])
    for result in results:
        lines.append(f"- `{result.crate}`: {result.summary}")

    failures = [result for result in results if not result.passed]
    if failures:
        lines.extend(["", "Failure excerpts:", ""])
        for result in failures:
            lines.append(f"## `{result.crate}`")
            lines.append("")
            lines.append("```text")
            lines.append(_tail(result.stderr) or "(no stderr output)")
            lines.append("```")
            lines.append("")

    lines.extend(["Interpretation:", ""])
    if all(result.passed for result in results):
        lines.append(
            f"- All wave {wave} crates passed `cargo publish --dry-run` from the current workspace state."
        )
    else:
        if all(_looks_like_registry_gap(result.stderr) for result in failures):
            lines.append(
                f"- Wave {wave} is packaging-clean, but local `cargo publish --dry-run` still depends on earlier waves being visible on crates.io."
            )
        else:
            lines.append(
                f"- Wave {wave} has at least one non-registry dry-run failure and needs follow-up before publish."
            )
        lines.append(
            "- Registry-gap failures are expected when later waves depend on crates that have not been published yet."
        )

    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--wave", type=int, required=True, help="Wave number from docs/release/v0.1.0-publish-waves.txt.")
    parser.add_argument(
        "--waves-file",
        default="docs/release/v0.1.0-publish-waves.txt",
        help="Path to the publish waves markdown file.",
    )
    parser.add_argument(
        "--output",
        help="Optional path to write the markdown report. Defaults to docs/release/v0.1.0-wave-<wave>-dry-run.txt.",
    )
    parser.add_argument(
        "--allow-registry-gap",
        action="store_true",
        help="Exit zero when every failure is an expected crates.io visibility gap.",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    waves = _parse_waves(repo_root / args.waves_file)
    crates = waves.get(args.wave)
    if not crates:
        raise SystemExit(f"wave {args.wave} not found in {args.waves_file}")

    results = [_run_dry_run(repo_root, crate) for crate in crates]
    report = _report_for_wave(args.wave, results)

    output_path = Path(args.output) if args.output else repo_root / f"docs/release/v0.1.0-wave-{args.wave}-dry-run.txt"
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(report, encoding="utf-8")
    print(report, end="")

    if all(result.passed for result in results):
        return 0

    failures = [result for result in results if not result.passed]
    if args.allow_registry_gap and all(_looks_like_registry_gap(result.stderr) for result in failures):
        return 0

    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
