#!/usr/bin/env python3
"""Check crates.io visibility for the configured Fret release waves."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
import urllib.error
import urllib.request
from dataclasses import dataclass
from datetime import date
from pathlib import Path


USER_AGENT = "fret-release-wave-registry-status/1.0"


@dataclass
class RegistryStatus:
    crate: str
    visible: bool
    latest_seen: str | None
    error: str | None = None

    @property
    def summary(self) -> str:
        if self.error:
            return f"query failed ({self.error})"
        if self.visible:
            return "visible on crates.io"
        if self.latest_seen:
            return f"missing target version; latest visible is {self.latest_seen}"
        return "crate not visible on crates.io yet"


def _repo_root() -> Path:
    return (Path(__file__).resolve().parent / "..").resolve()


def _parse_waves(path: Path) -> dict[int, list[str]]:
    waves: dict[int, list[str]] = {}
    current_wave: int | None = None

    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line.startswith("## Wave "):
            current_wave = int(line.split("(", 1)[0].removeprefix("## Wave ").strip())
            waves[current_wave] = []
            continue
        if current_wave is not None and line.startswith("- "):
            waves[current_wave].append(line.removeprefix("- ").strip())

    return waves


def _workspace_version(path: Path) -> str:
    cargo = tomllib.loads(path.read_text(encoding="utf-8"))
    return cargo["workspace"]["package"]["version"]


def _crate_versions(crate: str, timeout: float) -> list[str]:
    request = urllib.request.Request(
        f"https://crates.io/api/v1/crates/{crate}",
        headers={"User-Agent": USER_AGENT},
    )
    with urllib.request.urlopen(request, timeout=timeout) as response:
        payload = json.load(response)
    return [str(version["num"]) for version in payload.get("versions", [])]


def _check_crate(crate: str, target_version: str, timeout: float) -> RegistryStatus:
    try:
        versions = _crate_versions(crate, timeout)
    except urllib.error.HTTPError as exc:
        if exc.code == 404:
            return RegistryStatus(crate=crate, visible=False, latest_seen=None)
        return RegistryStatus(crate=crate, visible=False, latest_seen=None, error=f"http {exc.code}")
    except urllib.error.URLError as exc:
        reason = getattr(exc, "reason", exc)
        return RegistryStatus(crate=crate, visible=False, latest_seen=None, error=str(reason))
    except TimeoutError:
        return RegistryStatus(crate=crate, visible=False, latest_seen=None, error="timeout")

    latest_seen = versions[0] if versions else None
    return RegistryStatus(
        crate=crate,
        visible=target_version in versions,
        latest_seen=latest_seen,
    )


def _selected_waves(all_waves: dict[int, list[str]], requested: list[int] | None) -> dict[int, list[str]]:
    if not requested:
        return all_waves

    missing = [wave for wave in requested if wave not in all_waves]
    if missing:
        joined = ", ".join(str(wave) for wave in missing)
        raise SystemExit(f"wave(s) not found in publish waves file: {joined}")

    return {wave: all_waves[wave] for wave in requested}


def _report(target_version: str, results: dict[int, list[RegistryStatus]]) -> str:
    lines: list[str] = [
        "# Fret crates.io registry status",
        "",
        f"Date verified: {date.today().isoformat()}.",
        f"Target version: {target_version}.",
        "",
    ]

    fully_visible_waves: list[int] = []
    pending_waves: list[int] = []
    errored_waves: list[int] = []

    for wave, statuses in results.items():
        visible = sum(1 for status in statuses if status.visible)
        errors = [status for status in statuses if status.error]
        pending = [status for status in statuses if not status.visible and not status.error]
        lines.append(f"## Wave {wave} ({visible}/{len(statuses)} visible)")
        lines.append("")
        for status in statuses:
            lines.append(f"- `{status.crate}`: {status.summary}")
        lines.append("")

        if errors:
            errored_waves.append(wave)
        elif not pending:
            fully_visible_waves.append(wave)
        else:
            pending_waves.append(wave)

    lines.extend(["Interpretation:", ""])
    if fully_visible_waves:
        joined = ", ".join(str(wave) for wave in fully_visible_waves)
        lines.append(f"- Fully visible waves: {joined}.")
    else:
        lines.append("- No selected wave is fully visible on crates.io yet.")

    if pending_waves:
        joined = ", ".join(str(wave) for wave in pending_waves)
        lines.append(f"- Waiting on crates.io visibility for wave(s): {joined}.")

    if errored_waves:
        joined = ", ".join(str(wave) for wave in errored_waves)
        lines.append(f"- Query errors occurred while checking wave(s): {joined}.")

    return "\n".join(lines) + "\n"


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--wave",
        dest="waves",
        type=int,
        action="append",
        help="Wave number from docs/release/v0.1.0-publish-waves.txt. Repeat to check multiple waves.",
    )
    parser.add_argument(
        "--waves-file",
        default="docs/release/v0.1.0-publish-waves.txt",
        help="Path to the publish waves markdown file.",
    )
    parser.add_argument(
        "--version",
        help="Target crate version to check. Defaults to workspace.package.version from Cargo.toml.",
    )
    parser.add_argument(
        "--timeout",
        type=float,
        default=10.0,
        help="Per-request timeout in seconds.",
    )
    parser.add_argument(
        "--output",
        help="Optional path to write the markdown report.",
    )
    parser.add_argument(
        "--require-visible",
        action="store_true",
        help="Exit non-zero when any selected crate is not visible on crates.io.",
    )
    args = parser.parse_args(argv)

    repo_root = _repo_root()
    target_version = args.version or _workspace_version(repo_root / "Cargo.toml")
    waves = _parse_waves(repo_root / args.waves_file)
    selected = _selected_waves(waves, args.waves)

    results: dict[int, list[RegistryStatus]] = {}
    for wave, crates in selected.items():
        results[wave] = [_check_crate(crate, target_version, args.timeout) for crate in crates]

    report = _report(target_version, results)
    if args.output:
        output_path = Path(args.output)
        if not output_path.is_absolute():
            output_path = repo_root / output_path
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(report, encoding="utf-8")
    print(report, end="")

    if args.require_visible:
        incomplete = any(not status.visible for statuses in results.values() for status in statuses)
        if incomplete:
            return 1

    if any(status.error for statuses in results.values() for status in statuses):
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
