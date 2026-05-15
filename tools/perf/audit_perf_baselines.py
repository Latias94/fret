#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import pathlib
import re
import sys
from dataclasses import dataclass


LEGACY_HINTS = (
    "ui-gallery-steady.windows-rtx4090.v1.json",
    "ui-gallery-complex-steady.windows-rtx4090.v1.json",
    "ui-gallery-complex-typical.windows-rtx4090.v1.json",
)

PAYLOAD_THRESHOLD_SURFACES = {
    "ui-renderer-payload",
    "renderer-payload",
    "renderer",
    "all",
}

PAYLOAD_MEASURED_FIELDS = (
    "renderer_instance_bytes",
    "renderer_encode_scene_text_ops",
)

PAYLOAD_THRESHOLD_FIELDS = (
    "max_renderer_instance_bytes",
    "max_renderer_encode_scene_text_ops",
)


@dataclass(frozen=True)
class BaselineReport:
    path: pathlib.Path
    rows: int
    repeat: int | None
    threshold_surface: str | None
    legacy: bool
    missing_fields: tuple[str, ...]
    missing_payload_fields: tuple[str, ...]


def repo_root_from(path: pathlib.Path) -> pathlib.Path:
    resolved = path.resolve()
    parts = resolved.parts
    if len(parts) >= 4 and parts[-4].lower() == "docs":
        return resolved.parents[3]
    return resolved.parent


def discover_from_matrix(matrix_path: pathlib.Path) -> list[pathlib.Path]:
    text = matrix_path.read_text(encoding="utf-8")
    repo_root = repo_root_from(matrix_path)
    paths: list[pathlib.Path] = []
    seen: set[str] = set()
    for match in re.finditer(r"docs/workstreams/perf-baselines/[^`\s)]+\.json", text):
        rel = match.group(0)
        if "/policies/" in rel.replace("\\", "/"):
            continue
        if rel in seen:
            continue
        seen.add(rel)
        paths.append(repo_root / rel)
    return paths


def classify_legacy(path: pathlib.Path) -> bool:
    name = path.name
    return "windows-rtx4090" not in name or any(hint in name for hint in LEGACY_HINTS)


def requires_renderer_payload_contract(threshold_surface: str | None) -> bool:
    if threshold_surface is None:
        return False
    return threshold_surface in PAYLOAD_THRESHOLD_SURFACES


def missing_payload_contract_fields(rows: list[object]) -> tuple[str, ...]:
    missing: list[str] = []
    for index, row in enumerate(rows):
        if not isinstance(row, dict):
            missing.append(f"row[{index}]")
            continue
        script = row.get("script")
        row_label = str(script) if isinstance(script, str) and script else str(index)

        for measured_key in ("measured_p50", "measured_p90", "measured_p95", "measured_max"):
            measured = row.get(measured_key)
            if not isinstance(measured, dict):
                for field in PAYLOAD_MEASURED_FIELDS:
                    missing.append(f"{row_label}.{measured_key}.{field}")
                continue
            for field in PAYLOAD_MEASURED_FIELDS:
                if field not in measured:
                    missing.append(f"{row_label}.{measured_key}.{field}")

        seed = row.get("threshold_seed")
        if not isinstance(seed, dict):
            for field in PAYLOAD_MEASURED_FIELDS:
                missing.append(f"{row_label}.threshold_seed.{field}")
        else:
            for field in PAYLOAD_MEASURED_FIELDS:
                if field not in seed:
                    missing.append(f"{row_label}.threshold_seed.{field}")

        thresholds = row.get("thresholds")
        if not isinstance(thresholds, dict):
            for field in PAYLOAD_THRESHOLD_FIELDS:
                missing.append(f"{row_label}.thresholds.{field}")
        else:
            for field in PAYLOAD_THRESHOLD_FIELDS:
                if thresholds.get(field) is None:
                    missing.append(f"{row_label}.thresholds.{field}")

    return tuple(missing)


def audit_baseline(path: pathlib.Path) -> BaselineReport:
    data = json.loads(path.read_text(encoding="utf-8"))
    rows = data.get("rows", [])
    threshold_surface = data.get("threshold_surface")
    missing_fields = tuple(
        field
        for field in ("measured_p50", "measured_p90", "measured_p95", "measured_max")
        if any(field not in row for row in rows)
    )
    missing_payload_fields = (
        missing_payload_contract_fields(rows)
        if requires_renderer_payload_contract(threshold_surface)
        else ()
    )
    return BaselineReport(
        path=path,
        rows=len(rows),
        repeat=data.get("repeat"),
        threshold_surface=threshold_surface,
        legacy=classify_legacy(path),
        missing_fields=missing_fields,
        missing_payload_fields=missing_payload_fields,
    )


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Audit checked-in perf baselines for contract coverage."
    )
    parser.add_argument(
        "--matrix",
        type=pathlib.Path,
        help="Contract matrix markdown to scan for baseline paths.",
    )
    parser.add_argument(
        "--strict",
        action="store_true",
        help=(
            "Fail when a non-legacy baseline is missing measured_p50/p90/p95/max, "
            "threshold_surface, or renderer payload fields required by its threshold surface."
        ),
    )
    parser.add_argument("paths", nargs="*", type=pathlib.Path, help="Baseline JSON files.")
    args = parser.parse_args()

    if args.paths:
        paths = list(args.paths)
    elif args.matrix:
        paths = discover_from_matrix(args.matrix)
    else:
        paths = sorted(pathlib.Path("docs/workstreams/perf-baselines").glob("*.json"))

    exit_code = 0
    for path in paths:
        if not path.exists():
            print(f"missing\t{path}")
            exit_code = 1
            continue

        report = audit_baseline(path)
        surface = report.threshold_surface or "None"
        missing = ",".join(report.missing_fields) if report.missing_fields else "-"
        payload_missing = (
            ",".join(report.missing_payload_fields) if report.missing_payload_fields else "-"
        )
        status = "legacy" if report.legacy else "contract"
        print(
            f"{status}\t{path}\trows={report.rows}\trepeat={report.repeat}\t"
            f"surface={surface}\tmissing={missing}\tpayload_missing={payload_missing}"
        )
        if args.strict and not report.legacy:
            if (
                report.threshold_surface is None
                or report.missing_fields
                or report.missing_payload_fields
            ):
                exit_code = 1

    return exit_code


if __name__ == "__main__":
    raise SystemExit(main())
