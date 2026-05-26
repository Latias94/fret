#!/usr/bin/env python3
"""Build a shadcn component harness coverage matrix from existing parity evidence."""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from dataclasses import dataclass
from datetime import date
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[2]

COMPONENT_ALIASES = {
    "calendar-hijri": "calendar",
    "data-table/datagrid": "data-table",
}

STATE_DEPTH_AXES = {
    "disabled": {
        "label": "DIS",
        "keywords": [
            "disabled",
            "aria-disabled",
            "focusable-disabled",
        ],
    },
    "hover": {
        "label": "HOV",
        "keywords": [
            "hover",
            "hovered",
        ],
    },
    "focus_visible": {
        "label": "FOCUS-VIS",
        "keywords": [
            "focus-visible",
            "focus visible",
            "focus ring",
        ],
    },
    "pressed": {
        "label": "PRESS",
        "keywords": [
            "pressed",
        ],
    },
    "drag": {
        "label": "DRAG",
        "keywords": [
            "drag",
            "dragged",
            "splitter",
            "resize",
            "resizing",
        ],
    },
    "open": {
        "label": "OPEN",
        "keywords": [
            ".open",
            "_open",
            "open-",
            "open state",
            "open mode",
            "open snapshot",
            "open story",
            "open surface",
            "open-change",
            "opened",
            "expanded state",
            "expanded=true",
        ],
    },
    "keyboard": {
        "label": "KEY",
        "keywords": [
            "keyboard",
            "enter",
            "escape",
            "arrow",
            "home",
        ],
    },
    "mobile": {
        "label": "MOB",
        "keywords": [
            "mobile",
            "vp375",
            "375x",
            "responsive",
            "constrained viewport",
        ],
    },
    "rtl": {
        "label": "RTL",
        "keywords": [
            "rtl",
            "right-to-left",
        ],
    },
    "text_metrics": {
        "label": "TEXT-MET",
        "keywords": [
            "text metrics",
            "style-aware text",
            "text measurement",
            "text-paint",
            "text paint",
        ],
    },
    "paint_token": {
        "label": "PAINT",
        "keywords": [
            "paint",
            "token",
            "chrome",
            "foreground",
            "background",
            "border",
            "radius",
            "contrast",
            "color",
        ],
    },
}

COMPONENT_STATE_DEPTH_REQUIREMENTS = {
    "accordion": ["disabled", "open", "keyboard", "rtl", "text_metrics", "paint_token"],
    "alert": ["keyboard", "rtl", "text_metrics", "paint_token"],
    "alert-dialog": ["open", "keyboard", "mobile", "rtl", "text_metrics", "paint_token"],
    "badge": ["hover", "focus_visible", "keyboard", "rtl", "text_metrics", "paint_token"],
    "button": [
        "disabled",
        "hover",
        "focus_visible",
        "pressed",
        "keyboard",
        "text_metrics",
        "paint_token",
    ],
    "button-group": ["disabled", "keyboard", "text_metrics", "paint_token"],
    "calendar": ["hover", "focus_visible", "mobile", "text_metrics", "paint_token"],
    "checkbox": ["disabled", "focus_visible", "keyboard", "rtl", "text_metrics", "paint_token"],
    "collapsible": ["disabled", "open", "keyboard", "rtl", "text_metrics", "paint_token"],
    "command": ["disabled", "open", "keyboard", "rtl", "text_metrics", "paint_token"],
    "empty": ["keyboard", "rtl", "text_metrics", "paint_token"],
    "combobox": ["open", "keyboard", "mobile", "text_metrics", "paint_token"],
    "context-menu": ["open", "keyboard", "paint_token"],
    "data-table": ["keyboard", "rtl", "paint_token"],
    "date-picker": ["open", "keyboard", "mobile", "paint_token"],
    "dialog": ["open", "keyboard", "paint_token"],
    "drawer": ["open", "mobile", "paint_token"],
    "dropdown-menu": ["open", "keyboard", "mobile", "paint_token"],
    "hover-card": ["hover", "open", "keyboard", "paint_token"],
    "input": ["disabled", "focus_visible", "keyboard", "paint_token"],
    "input-otp": ["disabled", "focus_visible", "keyboard", "paint_token"],
    "item": ["hover", "focus_visible", "keyboard", "rtl", "text_metrics", "paint_token"],
    "kbd": ["keyboard", "rtl", "text_metrics", "paint_token"],
    "label": ["disabled", "keyboard", "rtl", "text_metrics", "paint_token"],
    "menubar": ["disabled", "open", "keyboard", "paint_token"],
    "navigation-menu": ["hover", "focus_visible", "open", "keyboard", "paint_token"],
    "popover": ["open", "keyboard", "paint_token"],
    "progress": ["rtl", "paint_token"],
    "radio-group": ["disabled", "focus_visible", "keyboard", "rtl", "text_metrics", "paint_token"],
    "resizable": ["drag", "keyboard", "rtl", "paint_token"],
    "scroll-area": ["hover", "focus_visible", "drag", "keyboard", "rtl", "text_metrics", "paint_token"],
    "select": ["open", "keyboard", "paint_token"],
    "separator": ["rtl", "text_metrics", "paint_token"],
    "sheet": ["open", "keyboard", "mobile", "paint_token"],
    "sidebar": ["hover", "focus_visible", "open", "keyboard", "mobile", "rtl", "paint_token"],
    "skeleton": ["rtl", "text_metrics", "paint_token"],
    "spinner": ["disabled", "rtl", "text_metrics", "paint_token"],
    "table": ["hover", "paint_token"],
    "textarea": ["disabled", "focus_visible", "drag", "keyboard", "rtl", "text_metrics", "paint_token"],
    "toggle": ["disabled", "hover", "focus_visible", "pressed", "keyboard", "rtl", "text_metrics", "paint_token"],
    "toggle-group": [
        "disabled",
        "hover",
        "focus_visible",
        "pressed",
        "keyboard",
        "rtl",
        "text_metrics",
        "paint_token",
    ],
    "tooltip": ["disabled", "hover", "focus_visible", "open", "keyboard", "paint_token"],
}


@dataclass(frozen=True)
class MarkdownTable:
    section: str
    headers: list[str]
    rows: list[dict[str, str]]


def _repo_path(path: str) -> Path:
    candidate = Path(path)
    if candidate.is_absolute():
        return candidate
    return ROOT / candidate


def _read_json(path: Path) -> dict[str, Any]:
    with path.open("r", encoding="utf-8") as handle:
        data = json.load(handle)
    if not isinstance(data, dict):
        raise ValueError(f"{path} must contain a JSON object")
    return data


def _split_md_row(line: str) -> list[str]:
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def _is_separator_row(line: str) -> bool:
    cells = _split_md_row(line)
    return bool(cells) and all(re.fullmatch(r":?-{3,}:?", cell) for cell in cells)


def _strip_inline_md(value: str) -> str:
    value = value.strip()
    if value.startswith("`") and value.endswith("`"):
        value = value[1:-1]
    return value.strip()


def _normalize_component(value: str) -> str:
    value = _strip_inline_md(value).lower()
    value = value.replace("_", "-").replace(" / ", "/")
    value = re.sub(r"\s+", "-", value)
    value = value.strip("-")
    return COMPONENT_ALIASES.get(value, value)


def _extract_audit_path(value: str) -> str | None:
    match = re.search(r"docs/audits/[A-Za-z0-9_.\-/]+\.md", value)
    return match.group(0) if match else None


def parse_markdown_tables(path: Path) -> list[MarkdownTable]:
    lines = path.read_text(encoding="utf-8").splitlines()
    tables: list[MarkdownTable] = []
    section = ""
    index = 0
    while index < len(lines):
        line = lines[index]
        if line.startswith("##"):
            section = line.lstrip("#").strip()
            index += 1
            continue
        if line.startswith("|") and index + 1 < len(lines) and _is_separator_row(lines[index + 1]):
            headers = [_strip_inline_md(header) for header in _split_md_row(line)]
            rows: list[dict[str, str]] = []
            index += 2
            while index < len(lines) and lines[index].startswith("|"):
                cells = _split_md_row(lines[index])
                row = {
                    headers[cell_index]: cells[cell_index] if cell_index < len(cells) else ""
                    for cell_index in range(len(headers))
                }
                rows.append(row)
                index += 1
            tables.append(MarkdownTable(section=section, headers=headers, rows=rows))
            continue
        index += 1
    return tables


def load_component_inventory(progress_doc: Path) -> dict[str, dict[str, Any]]:
    inventory: dict[str, dict[str, Any]] = {}
    for table in parse_markdown_tables(progress_doc):
        headers = set(table.headers)
        if table.section == "shadcn/ui v4 Registry Baseline" and "Registry name" in headers:
            for row in table.rows:
                component = _normalize_component(row.get("Registry name", ""))
                if not component:
                    continue
                notes = row.get("Notes", "")
                inventory[component] = {
                    "component": component,
                    "inventory_kind": "registry",
                    "rust_module": _strip_inline_md(row.get("Rust module", "")),
                    "implementation_status": _strip_inline_md(row.get("Status", "")),
                    "audit_status": _strip_inline_md(row.get("Audit", "")),
                    "audit_path": _extract_audit_path(notes),
                }
        elif table.section == "Non-registry surfaces" and "Surface" in headers:
            for row in table.rows:
                component = _normalize_component(row.get("Surface", ""))
                if not component:
                    continue
                notes = row.get("Notes", "")
                inventory[component] = {
                    "component": component,
                    "inventory_kind": "non_registry",
                    "rust_module": _strip_inline_md(row.get("Rust module", "")),
                    "implementation_status": _strip_inline_md(row.get("Status", "")),
                    "audit_status": _strip_inline_md(row.get("Audit", "")),
                    "audit_path": _extract_audit_path(notes),
                }
    return inventory


def load_alignment_priorities(progress_doc: Path) -> dict[str, dict[str, str]]:
    priorities: dict[str, dict[str, str]] = {}
    for table in parse_markdown_tables(progress_doc):
        if table.section != "Alignment Queue (2026-03)":
            continue
        for row in table.rows:
            component = _normalize_component(row.get("Component", ""))
            if not component:
                continue
            priorities[component] = {
                "priority": _strip_inline_md(row.get("Priority", "")),
                "risk_class": _strip_inline_md(row.get("Risk class", "")),
                "primary_upstream_truth": _strip_inline_md(
                    row.get("Primary upstream truth", "")
                ),
                "likely_owner_layer": _strip_inline_md(row.get("Likely owner layer", "")),
                "recommended_first_gate": _strip_inline_md(
                    row.get("Recommended first gate", "")
                ),
            }
    return priorities


def _script_refs(refs: list[str]) -> list[str]:
    return [ref for ref in refs if ref.startswith("tools/diag-scripts/")]


def _has_snapshot_ref(refs: list[str]) -> bool:
    return any(
        ref.endswith(".json")
        and (
            "goldens/shadcn-web" in ref
            or "/upstream-dom/" in ref.replace("\\", "/")
            or "docs/workstreams/shadcn-parity-discovery" in ref
        )
        for ref in refs
    )


def _contains_any(text: str, keywords: list[str]) -> bool:
    lowered = text.lower()
    normalized = f" {lowered.replace('_', ' ').replace('-', ' ')} "
    for keyword in keywords:
        keyword = keyword.lower()
        if any(separator in keyword for separator in [".", "_", "-"]):
            if keyword in lowered:
                return True
            continue
        pattern = r"(?<![a-z0-9])" + re.escape(keyword) + r"(?![a-z0-9])"
        if re.search(pattern, normalized):
            return True
    return False


def _collect_text(value: Any) -> list[str]:
    if value is None:
        return []
    if isinstance(value, str):
        return [value]
    if isinstance(value, dict):
        texts: list[str] = []
        for child in value.values():
            texts.extend(_collect_text(child))
        return texts
    if isinstance(value, list):
        texts: list[str] = []
        for child in value:
            texts.extend(_collect_text(child))
        return texts
    return [str(value)]


def _load_report_summary(path: str) -> dict[str, Any]:
    report_path = _repo_path(path)
    if not report_path.exists():
        return {}
    try:
        return _read_json(report_path).get("summary", {})
    except (OSError, ValueError, json.JSONDecodeError):
        return {}


def load_suite_reports(suite_report: Path) -> dict[str, list[dict[str, Any]]]:
    data = _read_json(suite_report)
    agent_reports = {
        report.get("id"): report
        for report in data.get("agent_packet", {}).get("reports", [])
        if isinstance(report, dict)
    }
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for report in data.get("reports", []):
        if not isinstance(report, dict):
            continue
        component = _normalize_component(str(report.get("component", "")))
        if not component:
            continue
        output = str(report.get("output", ""))
        summary = _load_report_summary(output)
        agent = agent_reports.get(report.get("id"), {})
        grouped[component].append(
            {
                "id": report.get("id"),
                "output": output,
                "status_counts": report.get("status_counts", {}),
                "layer_status_counts": report.get("layer_status_counts", {}),
                "repair_queue_count": agent.get("repair_queue_count", 0),
                "hardening_queue_count": agent.get("hardening_queue_count", 0),
                "gate_queue_count": agent.get("gate_queue_count", 0),
                "summary": summary,
            }
        )
    return grouped


def load_extra_reports(paths: list[Path]) -> dict[str, list[dict[str, Any]]]:
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for path in paths:
        if not path.exists():
            continue
        try:
            data = _read_json(path)
        except (OSError, ValueError, json.JSONDecodeError):
            continue
        component = _normalize_component(str(data.get("component", "")))
        if not component:
            continue
        agent_summary = data.get("agent_packet", {}).get("summary", {})
        grouped[component].append(
            {
                "id": path.stem,
                "output": str(path.relative_to(ROOT)),
                "status_counts": data.get("summary", {}).get("status_counts", {}),
                "repair_queue_count": agent_summary.get(
                    "repair_queue_count",
                    data.get("agent_packet", {}).get("repair_queue_count", 0),
                ),
                "hardening_queue_count": agent_summary.get(
                    "hardening_queue_count",
                    data.get("agent_packet", {}).get("hardening_queue_count", 0),
                ),
                "gate_queue_count": agent_summary.get(
                    "gate_queue_count",
                    data.get("agent_packet", {}).get("gate_queue_count", 0),
                ),
                "summary": data.get("summary", {}),
            }
        )
    return grouped


def load_manifest_targets(manifest_path: Path) -> dict[str, list[dict[str, Any]]]:
    data = _read_json(manifest_path)
    grouped: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for target in data.get("coverage_targets", []):
        if not isinstance(target, dict):
            continue
        component = _normalize_component(str(target.get("component", "")))
        if component:
            grouped[component].append(target)
    return grouped


def _axis_coverage(
    targets: list[dict[str, Any]], reports: list[dict[str, Any]]
) -> dict[str, bool]:
    upstream_refs = [
        ref for target in targets for ref in target.get("upstream_refs", []) if isinstance(ref, str)
    ]
    fret_refs = [
        ref for target in targets for ref in target.get("fret_refs", []) if isinstance(ref, str)
    ]
    report_summaries = [report.get("summary", {}) for report in reports]
    return {
        "source_refs": bool(upstream_refs),
        "upstream_dom_snapshot": _has_snapshot_ref(upstream_refs)
        or any(summary.get("upstream_dom_snapshot_count", 0) for summary in report_summaries),
        "fret_layout": bool(fret_refs)
        or any(summary.get("layout_sidecar_count", 0) for summary in report_summaries),
        "fret_bundle_semantics": any(
            summary.get("bundle_schema2_count", 0)
            or summary.get("fret_semantics_fact_count", 0)
            for summary in report_summaries
        ),
        "fret_text_paint": any(
            summary.get("fret_text_paint_row_count", 0)
            or summary.get("fret_text_paint_fact_count", 0)
            for summary in report_summaries
        ),
        "interaction_script": bool(_script_refs(fret_refs)),
        "responsive_viewport": any(
            target.get("viewport_class") != "desktop_1440x900" for target in targets
        )
        or any("responsive" in str(target.get("id", "")) for target in targets),
    }


def _report_depth_texts(report: dict[str, Any]) -> list[str]:
    texts: list[str] = []
    output = str(report.get("output", ""))
    if not output:
        return texts
    report_path = _repo_path(output)
    if not report_path.exists():
        return texts
    try:
        data = _read_json(report_path)
    except (OSError, ValueError, json.JSONDecodeError):
        return texts
    for key in [
        "upstream_contexts",
        "evidence_contexts",
        "validation_gates",
        "live_facts",
        "parts",
        "source_mapping",
    ]:
        texts.extend(_collect_text(data.get(key)))
    return texts


def _state_depth_coverage(
    targets: list[dict[str, Any]], reports: list[dict[str, Any]]
) -> dict[str, bool]:
    target_texts: list[str] = []
    for target in targets:
        target_texts.extend(
            [
                str(target.get("id", "")),
                str(target.get("state", "")),
                str(target.get("trigger", "")),
                str(target.get("viewport_class", "")),
                str(target.get("why", "")),
            ]
        )
        target_texts.extend(
            str(ref)
            for ref in target.get("fret_refs", [])
            if isinstance(ref, str)
        )
        target_texts.extend(
            str(ref)
            for ref in target.get("upstream_refs", [])
            if isinstance(ref, str)
        )

    report_texts: list[str] = []
    summaries = [report.get("summary", {}) for report in reports]
    for report in reports:
        report_texts.extend(
            [
                str(report.get("id", "")),
                str(report.get("output", "")),
            ]
        )
        report_texts.extend(_report_depth_texts(report))

    all_text = "\n".join(target_texts + report_texts)
    depth = {
        axis: _contains_any(all_text, spec["keywords"])
        for axis, spec in STATE_DEPTH_AXES.items()
    }

    if any(target.get("viewport_class") != "desktop_1440x900" for target in targets):
        depth["mobile"] = True
    if any(summary.get("fret_text_paint_row_count", 0) for summary in summaries):
        depth["text_metrics"] = True
        depth["paint_token"] = True
    if any(summary.get("fret_text_paint_fact_count", 0) for summary in summaries):
        depth["text_metrics"] = True
        depth["paint_token"] = True
    if any(summary.get("fret_interaction_fact_count", 0) for summary in summaries):
        for report in reports:
            if _script_refs(_collect_text(report.get("output", ""))):
                depth["keyboard"] = depth["keyboard"] or _contains_any(
                    all_text, STATE_DEPTH_AXES["keyboard"]["keywords"]
                )
                break

    return depth


def _status_for_component(
    inventory_row: dict[str, Any] | None,
    targets: list[dict[str, Any]],
    reports: list[dict[str, Any]],
) -> str:
    repair_count = sum(int(report.get("repair_queue_count", 0)) for report in reports)
    hardening_count = sum(int(report.get("hardening_queue_count", 0)) for report in reports)
    if repair_count:
        return "repair_needed"
    if reports and not repair_count and not hardening_count:
        return "regression_locked"
    if reports:
        return "harness_hardening"
    if targets:
        return "coverage_targeted"
    if inventory_row and inventory_row.get("implementation_status") == "Present":
        return "inventory_only"
    return "not_in_harness"


def _state_depth_missing(component: str, state_depth: dict[str, bool]) -> list[str]:
    required = COMPONENT_STATE_DEPTH_REQUIREMENTS.get(component, [])
    return [axis for axis in required if not state_depth.get(axis)]


def _next_gap(
    status: str,
    axes: dict[str, bool],
    state_depth_missing: list[str],
) -> str:
    if status == "repair_needed":
        return "repair_by_owner_layer"
    if not axes["source_refs"]:
        return "add_upstream_source_refs"
    if not axes["upstream_dom_snapshot"]:
        return "capture_upstream_dom_snapshot"
    if not axes["fret_layout"]:
        return "capture_fret_layout_sidecar"
    if not axes["fret_bundle_semantics"]:
        return "capture_bundle_schema2_semantics"
    if not axes["interaction_script"]:
        return "add_behavior_diag_script"
    if not axes["fret_text_paint"]:
        return "add_text_paint_or_paint_snapshot_gate"
    if status == "coverage_targeted":
        return "promote_target_to_suite_report"
    if state_depth_missing:
        return f"expand_{state_depth_missing[0]}_state_depth"
    return "state_depth_model_satisfied"


def build_matrix(
    progress_doc: Path,
    manifest_path: Path,
    suite_report: Path,
    extra_reports: list[Path],
) -> dict[str, Any]:
    inventory = load_component_inventory(progress_doc)
    priorities = load_alignment_priorities(progress_doc)
    targets_by_component = load_manifest_targets(manifest_path)
    reports_by_component = load_suite_reports(suite_report)
    for component, reports in load_extra_reports(extra_reports).items():
        reports_by_component[component].extend(reports)
    component_names = sorted(
        set(inventory) | set(priorities) | set(targets_by_component) | set(reports_by_component)
    )

    components: list[dict[str, Any]] = []
    status_counts: dict[str, int] = defaultdict(int)
    axis_counts: dict[str, int] = defaultdict(int)
    state_depth_counts: dict[str, int] = defaultdict(int)
    for component in component_names:
        targets = targets_by_component.get(component, [])
        reports = reports_by_component.get(component, [])
        axes = _axis_coverage(targets, reports)
        for axis, covered in axes.items():
            if covered:
                axis_counts[axis] += 1
        state_depth = _state_depth_coverage(targets, reports)
        for axis, covered in state_depth.items():
            if covered:
                state_depth_counts[axis] += 1
        missing_depth = _state_depth_missing(component, state_depth)
        status = _status_for_component(inventory.get(component), targets, reports)
        status_counts[status] += 1
        components.append(
            {
                **(inventory.get(component) or {"component": component}),
                "priority": priorities.get(component, {}).get("priority", ""),
                "risk_class": priorities.get(component, {}).get("risk_class", ""),
                "likely_owner_layer": priorities.get(component, {}).get("likely_owner_layer", ""),
                "harness_status": status,
                "covered_axes": axes,
                "state_depth": state_depth,
                "required_state_depth": COMPONENT_STATE_DEPTH_REQUIREMENTS.get(component, []),
                "missing_state_depth": missing_depth,
                "coverage_target_count": len(targets),
                "harness_report_count": len(reports),
                "repair_queue_count": sum(
                    int(report.get("repair_queue_count", 0)) for report in reports
                ),
                "hardening_queue_count": sum(
                    int(report.get("hardening_queue_count", 0)) for report in reports
                ),
                "gate_queue_count": sum(
                    int(report.get("gate_queue_count", 0)) for report in reports
                ),
                "target_ids": [target.get("id") for target in targets],
                "target_states": sorted(
                    {
                        str(target.get("state", "")).strip()
                        for target in targets
                        if str(target.get("state", "")).strip()
                    }
                ),
                "target_viewport_classes": sorted(
                    {
                        str(target.get("viewport_class", "")).strip()
                        for target in targets
                        if str(target.get("viewport_class", "")).strip()
                    }
                ),
                "report_ids": [report.get("id") for report in reports],
                "next_gap": _next_gap(status, axes, missing_depth),
            }
        )

    return {
        "schema_version": 1,
        "generated_date": date.today().isoformat(),
        "source_docs": {
            "progress_doc": str(progress_doc.relative_to(ROOT)),
            "manifest": str(manifest_path.relative_to(ROOT)),
            "suite_report": str(suite_report.relative_to(ROOT)),
            "extra_reports": [str(path.relative_to(ROOT)) for path in extra_reports],
        },
        "self_rendered_harness_model": {
            "principle": "Do not compare Fret output to HTML tree structure. Compare observable outcomes from upstream DOM/CSS references against Fret layout, semantics, interaction, paint, and text diagnostics.",
            "axes": [
                "source_refs",
                "upstream_dom_snapshot",
                "fret_layout",
                "fret_bundle_semantics",
                "fret_text_paint",
                "interaction_script",
                "responsive_viewport",
            ],
            "state_depth_axes": list(STATE_DEPTH_AXES),
            "component_state_depth_requirements": COMPONENT_STATE_DEPTH_REQUIREMENTS,
        },
        "summary": {
            "component_count": len(components),
            "registry_component_count": sum(
                1 for component in components if component.get("inventory_kind") == "registry"
            ),
            "non_registry_surface_count": sum(
                1 for component in components if component.get("inventory_kind") == "non_registry"
            ),
            "status_counts": dict(sorted(status_counts.items())),
            "axis_component_counts": dict(sorted(axis_counts.items())),
            "state_depth_component_counts": dict(sorted(state_depth_counts.items())),
        },
        "components": components,
    }


AXIS_LABELS = {
    "source_refs": "SRC",
    "upstream_dom_snapshot": "UP-DOM",
    "fret_layout": "LAYOUT",
    "fret_bundle_semantics": "SEM",
    "fret_text_paint": "TEXT",
    "interaction_script": "BEHAV",
    "responsive_viewport": "RESP",
}


def _axis_tokens(axes: dict[str, bool]) -> str:
    tokens = [label for axis, label in AXIS_LABELS.items() if axes.get(axis)]
    return ", ".join(tokens) if tokens else "-"


def _state_depth_tokens(state_depth: dict[str, bool]) -> str:
    tokens = [
        str(spec["label"])
        for axis, spec in STATE_DEPTH_AXES.items()
        if state_depth.get(axis)
    ]
    return ", ".join(tokens) if tokens else "-"


def _missing_state_depth_tokens(component: dict[str, Any]) -> str:
    required = component.get("required_state_depth", [])
    missing = component.get("missing_state_depth", [])
    if not required:
        return "-"
    if not missing:
        return "ok"
    tokens = [str(STATE_DEPTH_AXES[axis]["label"]) for axis in missing]
    return ", ".join(tokens)


def write_markdown(matrix: dict[str, Any], output: Path) -> None:
    lines = [
        "---",
        "title: Shadcn Component Harness Matrix v1",
        "status: active",
        f"date: {matrix['generated_date']}",
        "---",
        "",
        "# Shadcn Component Harness Matrix v1",
        "",
        "This matrix tracks how far automated parity evidence reaches for each shadcn surface in Fret's self-rendered runtime.",
        "",
        "Fret should not compare itself to HTML tree structure. Upstream DOM/CSS snapshots are source references for web-facing shadcn outcomes; Fret proof must come from layout sidecars, bundle schema2 semantics, interaction scripts, text/paint diagnostics, screenshots only when needed, and owner/layer-classified repair queues.",
        "",
        "Axis legend:",
        "",
        "- `SRC`: upstream source refs are attached.",
        "- `UP-DOM`: upstream DOM/CSS snapshot evidence exists.",
        "- `LAYOUT`: Fret layout/geometry evidence exists.",
        "- `SEM`: Fret bundle semantics evidence exists.",
        "- `TEXT`: Fret text/paint evidence exists.",
        "- `BEHAV`: interaction/behavior diag script exists.",
        "- `RESP`: responsive or non-desktop viewport coverage exists.",
        "",
        "State-depth legend:",
        "",
        "- `DIS`: disabled / aria-disabled / focusable-disabled evidence.",
        "- `HOV`: hover evidence.",
        "- `FOCUS-VIS`: focus-visible or focus-ring evidence.",
        "- `PRESS`: pressed-state evidence.",
        "- `DRAG`: splitter drag or resize evidence.",
        "- `OPEN`: open / expanded evidence.",
        "- `KEY`: keyboard path evidence.",
        "- `MOB`: mobile, constrained, or responsive viewport evidence.",
        "- `RTL`: right-to-left evidence.",
        "- `TEXT-MET`: text metrics or style-aware text measurement evidence.",
        "- `PAINT`: paint, token, chrome, color, border, radius, or contrast evidence.",
        "",
        "## Summary",
        "",
        "```json",
        json.dumps(matrix["summary"], indent=2, sort_keys=True),
        "```",
        "",
        "## Component Matrix",
        "",
        "| Component | Kind | Impl | Harness status | Axes | Depth | Missing depth | Targets | Reports | Queues | Next gap |",
        "| --- | --- | --- | --- | --- | --- | --- | ---: | ---: | --- | --- |",
    ]
    for component in matrix["components"]:
        queues = (
            f"repair={component['repair_queue_count']}, "
            f"hardening={component['hardening_queue_count']}, "
            f"gate={component['gate_queue_count']}"
        )
        lines.append(
            "| {component} | {kind} | {impl} | {status} | {axes} | {depth} | {missing_depth} | {targets} | {reports} | {queues} | {gap} |".format(
                component=component["component"],
                kind=component.get("inventory_kind", "-"),
                impl=component.get("implementation_status", "-"),
                status=component["harness_status"],
                axes=_axis_tokens(component["covered_axes"]),
                depth=_state_depth_tokens(component["state_depth"]),
                missing_depth=_missing_state_depth_tokens(component),
                targets=component["coverage_target_count"],
                reports=component["harness_report_count"],
                queues=queues,
                gap=component["next_gap"],
            )
        )
    lines.extend(
        [
            "",
            "## Interpretation",
            "",
            "- `regression_locked` means the current suite report has no repair or hardening queue for that component slice. It does not mean every state, breakpoint, DPI, font metric, and interaction path is covered.",
            "- `Depth` records state signals proven by manifest targets, component packets, validation gates, and Fret diagnostics summaries. `Missing depth` is filtered through component-specific applicability so irrelevant states are not treated as gaps.",
            "- `coverage_targeted` means a priority target exists in the manifest, but it is not yet represented as a current suite report.",
            "- `inventory_only` means the component exists in the shadcn inventory but does not yet have a harness seed.",
            "- The next automation step is to turn high-risk `inventory_only` and `coverage_targeted` rows into fixtures with upstream source refs, Fret `test_id`s, diag scripts, and packet checks.",
            "",
        ]
    )
    output.write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--progress-doc",
        default="docs/shadcn-declarative-progress.md",
        help="Canonical shadcn progress markdown.",
    )
    parser.add_argument(
        "--manifest",
        default="tools/parity-discovery/manifests/shadcn_parity_coverage_v2.json",
        help="Parity coverage manifest JSON.",
    )
    parser.add_argument(
        "--suite-report",
        default="docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/shadcn_parity_suite_report_v2.json",
        help="Current parity suite report JSON.",
    )
    parser.add_argument(
        "--extra-report",
        action="append",
        default=[
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_group_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/drawer_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/calendar_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/select_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/combobox_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/popover_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/dropdown_menu_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/input_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/data_table_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/date_picker_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/resizable_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/sidebar_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/progress_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/badge_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/button_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/accordion_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/alert_dialog_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/aspect_ratio_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/avatar_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/breadcrumb_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/card_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/checkbox_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/collapsible_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/command_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/empty_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/field_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/form_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/input_group_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/item_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/kbd_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/label_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/radio_group_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/scroll_area_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/separator_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/skeleton_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/spinner_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/textarea_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/toggle_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/toggle_group_agent_packet_p0_v1.json",
            "docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/pagination_agent_packet_p0_v1.json",
        ],
        help="Additional component agent packet report to fold into the matrix. May be repeated.",
    )
    parser.add_argument(
        "--output-json",
        default="docs/workstreams/shadcn-component-parity-matrix-v1/artifacts/shadcn_component_harness_matrix_v1.json",
    )
    parser.add_argument(
        "--output-md",
        default="docs/workstreams/shadcn-component-parity-matrix-v1/MATRIX.md",
    )
    args = parser.parse_args()

    matrix = build_matrix(
        _repo_path(args.progress_doc),
        _repo_path(args.manifest),
        _repo_path(args.suite_report),
        [_repo_path(path) for path in args.extra_report],
    )

    output_json = _repo_path(args.output_json)
    output_json.parent.mkdir(parents=True, exist_ok=True)
    output_json.write_text(
        json.dumps(matrix, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )

    output_md = _repo_path(args.output_md)
    output_md.parent.mkdir(parents=True, exist_ok=True)
    write_markdown(matrix, output_md)

    print(
        "generated {json_path} and {md_path} ({count} components)".format(
            json_path=output_json.relative_to(ROOT),
            md_path=output_md.relative_to(ROOT),
            count=matrix["summary"]["component_count"],
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
