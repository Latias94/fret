#!/usr/bin/env python3
"""Generate deterministic shadcn parity discovery reports from mapping fixtures."""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


SUPPORTED_SCHEMA_VERSION = 1
SUPPORTED_CHECK_KINDS = {
    "existing_gate",
    "fret_layout_sidecar",
    "upstream_dom_snapshot",
    "live_measurement_required",
    "expected_mismatch",
    "blocked",
}
SUPPORTED_OBSERVED = {"pass", "fail", "missing", "unknown"}
SUPPORTED_CONFIDENCE = {"high", "medium", "low"}
OWNER_ORDER = [
    "component_recipe",
    "gallery_composition",
    "mechanism_core",
    "diagnostics_surface",
    "upstream_reference",
    "unknown",
]
SUPPORTED_OWNER_KINDS = set(OWNER_ORDER)
LAYER_ORDER = [
    "runner",
    "mechanism",
    "policy",
    "recipe",
    "app_demo",
    "upstream",
    "unknown",
]
SUPPORTED_LAYER_KINDS = set(LAYER_ORDER)
SUPPORTED_PROMOTION_TARGETS = {
    "diag_script",
    "component_fixture",
    "mechanism_harness",
    "none",
}
SUPPORTED_PREDICATE_KINDS = {
    "bounds_metric",
    "bounds_metric_delta",
    "root_metric",
}
SUPPORTED_EVIDENCE_SOURCES = {
    "auto",
    "layout_sidecar",
    "bundle_schema2_semantics",
}
SUPPORTED_METRICS = {
    "x",
    "y",
    "width",
    "height",
    "left",
    "top",
    "right",
    "bottom",
    "center_x",
    "center_y",
}
SUPPORTED_COMPARISONS = {
    "between",
    "gte",
    "lte",
    "eq",
}
STATUS_ORDER = ["pass_known", "needs_live_measurement", "mismatch", "blocked"]
TRIAGE_LEVEL_ORDER = ["critical", "high", "medium", "low", "none"]
TEST_ID_LABEL_RE = re.compile(r"\[test_id=([^\]]+)\]")
PX_VALUE_RE = re.compile(r"^-?\d+(?:\.\d+)?px$")
OWNER_BY_PROMOTION_TARGET = {
    "diag_script": "diagnostics_surface",
    "component_fixture": "gallery_composition",
    "mechanism_harness": "mechanism_core",
    "none": "unknown",
}
LAYER_BY_OWNER = {
    "component_recipe": "recipe",
    "gallery_composition": "app_demo",
    "mechanism_core": "mechanism",
    "diagnostics_surface": "runner",
    "upstream_reference": "upstream",
    "unknown": "unknown",
}
STATUS_TRIAGE_SCORE = {
    "mismatch": 60,
    "blocked": 40,
    "needs_live_measurement": 20,
    "pass_known": 0,
}
LAYER_TRIAGE_SCORE = {
    "mechanism": 20,
    "runner": 18,
    "policy": 15,
    "recipe": 10,
    "app_demo": 6,
    "upstream": 4,
    "unknown": 5,
}
PROMOTION_TRIAGE_SCORE = {
    "mechanism_harness": 12,
    "diag_script": 10,
    "component_fixture": 6,
    "none": 0,
}
AXIS_TRIAGE_SCORE = {
    "semantics": 12,
    "interaction": 12,
    "layout": 8,
    "chrome": 5,
    "teaching": 2,
}
CONFIDENCE_TRIAGE_SCORE = {
    "high": 8,
    "medium": 4,
    "low": 0,
}


class FixtureError(ValueError):
    """Raised when a mapping fixture does not match the prototype schema."""


@dataclass(frozen=True)
class Bounds:
    x: float
    y: float
    width: float
    height: float

    @classmethod
    def from_rect(cls, rect: dict[str, Any]) -> "Bounds":
        try:
            return cls(
                x=float(rect["x"]),
                y=float(rect["y"]),
                width=float(rect["w"]),
                height=float(rect["h"]),
            )
        except (KeyError, TypeError, ValueError) as exc:
            raise FixtureError(f"invalid sidecar rect: {rect!r}") from exc

    def metric(self, name: str) -> float:
        if name in {"x", "left"}:
            return self.x
        if name in {"y", "top"}:
            return self.y
        if name == "width":
            return self.width
        if name == "height":
            return self.height
        if name == "right":
            return self.x + self.width
        if name == "bottom":
            return self.y + self.height
        if name == "center_x":
            return self.x + self.width * 0.5
        if name == "center_y":
            return self.y + self.height * 0.5
        raise FixtureError(f"unsupported metric {name!r}")

    def to_json(self) -> dict[str, float]:
        return {
            "x": round(self.x, 3),
            "y": round(self.y, 3),
            "w": round(self.width, 3),
            "h": round(self.height, 3),
        }


@dataclass(frozen=True)
class LayoutNode:
    test_id: str
    bounds: Bounds
    raw_bounds: Bounds
    scale_factor: float
    coordinate_units: str
    node: str
    sidecar_path: str
    root_index: int
    kind: str | None
    label: str | None = None
    semantics: dict[str, Any] | None = None
    source: str = "layout_sidecar"


@dataclass(frozen=True)
class LayoutRoot:
    bounds: Bounds
    raw_bounds: Bounds
    scale_factor: float
    coordinate_units: str
    sidecar_path: str
    source: str = "layout_sidecar"


@dataclass(frozen=True)
class DomNode:
    target_id: str
    bounds: Bounds
    device_pixel_ratio: float
    snapshot_path: str
    snapshot_name: str
    theme: str
    mode: str
    variant: str
    context_id: str | None
    viewport: dict[str, Any]
    path: str
    tag: str
    attrs: dict[str, Any]
    class_name: str | None
    text: str | None
    computed_style: dict[str, Any]
    child_count: int


@dataclass
class LayoutEvidence:
    nodes_by_test_id: dict[str, list[LayoutNode]]
    bundle_nodes_by_test_id: dict[str, list[LayoutNode]]
    sidecar_paths: list[str]
    bundle_paths: list[str]
    roots: list[LayoutRoot]
    text_paint_facts_by_node: dict[str, list[dict[str, Any]]]
    text_paint_associated_facts_by_node: dict[str, list[dict[str, Any]]]
    text_label_facts_by_node: dict[str, list[dict[str, Any]]]
    text_paint_bundle_entry_count: int
    text_paint_fact_row_count: int
    text_paint_association_row_count: int
    text_paint_unassociated_row_count: int
    text_label_fact_row_count: int

    @classmethod
    def empty(cls) -> "LayoutEvidence":
        return cls(
            nodes_by_test_id={},
            bundle_nodes_by_test_id={},
            sidecar_paths=[],
            bundle_paths=[],
            roots=[],
            text_paint_facts_by_node={},
            text_paint_associated_facts_by_node={},
            text_label_facts_by_node={},
            text_paint_bundle_entry_count=0,
            text_paint_fact_row_count=0,
            text_paint_association_row_count=0,
            text_paint_unassociated_row_count=0,
            text_label_fact_row_count=0,
        )

    def find(
        self, test_id: str, evidence_source: str | None = None
    ) -> LayoutNode | None:
        if evidence_source == "layout_sidecar":
            nodes = self.nodes_by_test_id.get(test_id)
            return nodes[0] if nodes else None
        if evidence_source == "bundle_schema2_semantics":
            nodes = self.bundle_nodes_by_test_id.get(test_id)
            return nodes[0] if nodes else None
        nodes = self.nodes_by_test_id.get(test_id)
        if not nodes:
            nodes = self.bundle_nodes_by_test_id.get(test_id)
            if not nodes:
                return None
        return nodes[0]

    def duplicate_count(self, test_id: str, evidence_source: str | None = None) -> int:
        if evidence_source == "layout_sidecar":
            return len(self.nodes_by_test_id.get(test_id, []))
        if evidence_source == "bundle_schema2_semantics":
            return len(self.bundle_nodes_by_test_id.get(test_id, []))
        nodes = self.nodes_by_test_id.get(test_id)
        if nodes:
            return len(nodes)
        return len(self.bundle_nodes_by_test_id.get(test_id, []))

    def find_root(self, evidence_source: str | None = None) -> LayoutRoot | None:
        if not self.roots:
            return None
        if evidence_source is not None and evidence_source != "auto":
            for root in self.roots:
                if root.source == evidence_source:
                    return root
            return None
        for root in self.roots:
            if root.source == "layout_sidecar":
                return root
        return self.roots[0]

    def test_id_count(self) -> int:
        return len(set(self.nodes_by_test_id) | set(self.bundle_nodes_by_test_id))


@dataclass
class DomEvidence:
    nodes_by_target_id: dict[str, DomNode]
    nodes_by_snapshot_path: dict[tuple[str, str, str, str, str], DomNode]
    snapshot_paths: list[str]
    contexts: list[dict[str, Any]]

    @classmethod
    def empty(cls) -> "DomEvidence":
        return cls(
            nodes_by_target_id={},
            nodes_by_snapshot_path={},
            snapshot_paths=[],
            contexts=[],
        )

    def find(self, target_id: str) -> DomNode | None:
        return self.nodes_by_target_id.get(target_id)

    def find_path(
        self,
        snapshot: str,
        theme: str,
        path: str,
        mode: str = "",
        variant: str = "",
    ) -> DomNode | None:
        return self.nodes_by_snapshot_path.get((snapshot, theme, mode, variant, path))


def _require_object(value: Any, path: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise FixtureError(f"{path} must be an object")
    return value


def _require_list(value: Any, path: str) -> list[Any]:
    if not isinstance(value, list):
        raise FixtureError(f"{path} must be a list")
    return value


def _require_str(value: Any, path: str) -> str:
    if not isinstance(value, str) or not value:
        raise FixtureError(f"{path} must be a non-empty string")
    return value


def _require_str_list(value: Any, path: str) -> list[str]:
    items = _require_list(value, path)
    result: list[str] = []
    for index, item in enumerate(items):
        result.append(_require_str(item, f"{path}[{index}]"))
    return result


def _require_unique_ids(items: list[dict[str, Any]], path: str) -> None:
    seen: set[str] = set()
    for index, item in enumerate(items):
        item_id = _require_str(item.get("id"), f"{path}[{index}].id")
        if item_id in seen:
            raise FixtureError(f"{path} contains duplicate id {item_id!r}")
        seen.add(item_id)


def _validate_predicates(check: dict[str, Any], check_path: str) -> None:
    raw_predicates = check.get("predicates")
    if raw_predicates is None:
        return
    predicates = [
        _require_object(item, f"{check_path}.predicates[{index}]")
        for index, item in enumerate(_require_list(raw_predicates, f"{check_path}.predicates"))
    ]
    for predicate_index, predicate in enumerate(predicates):
        predicate_path = f"{check_path}.predicates[{predicate_index}]"
        kind = _require_str(predicate.get("kind"), f"{predicate_path}.kind")
        if kind not in SUPPORTED_PREDICATE_KINDS:
            raise FixtureError(f"{predicate_path}.kind has unsupported value {kind!r}")
        metric = _require_str(predicate.get("metric"), f"{predicate_path}.metric")
        if metric not in SUPPORTED_METRICS:
            raise FixtureError(f"{predicate_path}.metric has unsupported value {metric!r}")
        comparison = _require_str(
            predicate.get("comparison"), f"{predicate_path}.comparison"
        )
        if comparison not in SUPPORTED_COMPARISONS:
            raise FixtureError(
                f"{predicate_path}.comparison has unsupported value {comparison!r}"
            )
        evidence_source = predicate.get("evidence_source")
        if evidence_source is not None:
            evidence_source = _require_str(
                evidence_source, f"{predicate_path}.evidence_source"
            )
            if evidence_source not in SUPPORTED_EVIDENCE_SOURCES:
                raise FixtureError(
                    f"{predicate_path}.evidence_source has unsupported value {evidence_source!r}"
                )
        _require_float(predicate.get("eps_px", 0.0), f"{predicate_path}.eps_px")
        if kind == "bounds_metric":
            _require_str(predicate.get("target"), f"{predicate_path}.target")
        elif kind == "bounds_metric_delta":
            _require_str(predicate.get("a"), f"{predicate_path}.a")
            _require_str(predicate.get("b"), f"{predicate_path}.b")
        if comparison == "between":
            _require_float(predicate.get("min_px"), f"{predicate_path}.min_px")
            _require_float(predicate.get("max_px"), f"{predicate_path}.max_px")
        else:
            _require_float(predicate.get("value_px"), f"{predicate_path}.value_px")


def _validate_upstream_predicates(check: dict[str, Any], check_path: str) -> None:
    raw_predicates = check.get("upstream_predicates")
    if raw_predicates is None:
        return
    shadow_check = dict(check)
    shadow_check["predicates"] = raw_predicates
    _validate_predicates(shadow_check, check_path)


def _validate_upstream_dom_target_refs(
    value: Any, target_ids: set[str], path: str
) -> list[str]:
    if value is None:
        return []
    refs = _require_str_list(value, path)
    for ref_id in refs:
        if ref_id not in target_ids:
            raise FixtureError(f"{path} references unknown upstream DOM target id {ref_id!r}")
    return refs


def _require_float(value: Any, path: str) -> float:
    if not isinstance(value, int | float):
        raise FixtureError(f"{path} must be a number")
    return float(value)


def _resolve_owner(check: dict[str, Any]) -> str:
    owner = check.get("owner")
    if owner is not None:
        owner = _require_str(owner, "$.parts[].checks[].owner")
        if owner not in SUPPORTED_OWNER_KINDS:
            raise FixtureError(f"$.parts[].checks[].owner has unsupported value {owner!r}")
        return owner

    target = check["promotion"]["target"]
    return OWNER_BY_PROMOTION_TARGET.get(target, "unknown")


def _resolve_layer(check: dict[str, Any], owner: str) -> str:
    layer = check.get("layer")
    if layer is not None:
        layer = _require_str(layer, "$.parts[].checks[].layer")
        if layer not in SUPPORTED_LAYER_KINDS:
            raise FixtureError(f"$.parts[].checks[].layer has unsupported value {layer!r}")
        return layer
    return LAYER_BY_OWNER.get(owner, "unknown")


def _validate_upstream_contexts(mapping: dict[str, Any]) -> list[dict[str, Any]]:
    raw_contexts = mapping.get("upstream_contexts", [])
    contexts = [
        _require_object(item, f"$.upstream_contexts[{index}]")
        for index, item in enumerate(
            _require_list(raw_contexts, "$.upstream_contexts")
        )
    ]
    _require_unique_ids(contexts, "$.upstream_contexts")
    for index, context in enumerate(contexts):
        context_path = f"$.upstream_contexts[{index}]"
        _require_str(context.get("snapshot"), f"{context_path}.snapshot")
        _require_str(context.get("theme"), f"{context_path}.theme")
        mode = context.get("mode")
        if mode is not None:
            _require_str(mode, f"{context_path}.mode")
        variant = context.get("variant")
        if variant is not None:
            _require_str(variant, f"{context_path}.variant")
        viewport = _require_object(context.get("viewport"), f"{context_path}.viewport")
        _require_float(viewport.get("width_px"), f"{context_path}.viewport.width_px")
        _require_float(viewport.get("height_px"), f"{context_path}.viewport.height_px")
        device_pixel_ratio = context.get("device_pixel_ratio")
        if device_pixel_ratio is not None:
            _require_float(device_pixel_ratio, f"{context_path}.device_pixel_ratio")
    return contexts


def _context_matches_snapshot(
    context: dict[str, Any],
    snapshot_name: str,
    theme: str,
    snapshot_mode: str,
    snapshot_variant: str,
    theme_data: dict[str, Any],
) -> bool:
    if context.get("snapshot") != snapshot_name:
        return False
    if context.get("theme") != theme:
        return False
    if (context.get("mode") or "") != snapshot_mode:
        return False
    if (context.get("variant") or "") != snapshot_variant:
        return False

    raw_viewport = theme_data.get("viewport")
    if not isinstance(raw_viewport, dict):
        return False
    width = raw_viewport.get("w")
    height = raw_viewport.get("h")
    if not isinstance(width, int | float) or not isinstance(height, int | float):
        return False

    viewport = context.get("viewport")
    if not isinstance(viewport, dict):
        return False
    if round(float(width), 3) != round(float(viewport.get("width_px")), 3):
        return False
    if round(float(height), 3) != round(float(viewport.get("height_px")), 3):
        return False

    context_dpr = context.get("device_pixel_ratio")
    if context_dpr is not None:
        theme_dpr = float(theme_data.get("devicePixelRatio") or 1.0)
        if round(theme_dpr, 3) != round(float(context_dpr), 3):
            return False

    return True


def load_mapping(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise FixtureError(f"{path}: invalid JSON: {exc}") from exc

    mapping = _require_object(data, "$")
    if mapping.get("schema_version") != SUPPORTED_SCHEMA_VERSION:
        raise FixtureError(
            f"$.schema_version must be {SUPPORTED_SCHEMA_VERSION}, got {mapping.get('schema_version')!r}"
        )

    _require_str(mapping.get("component"), "$.component")
    _require_str(mapping.get("style"), "$.style")
    upstream_contexts = _validate_upstream_contexts(mapping)
    context_ids = {context["id"] for context in upstream_contexts}

    report = _require_object(mapping.get("report"), "$.report")
    if report.get("schema_version") != SUPPORTED_SCHEMA_VERSION:
        raise FixtureError(
            f"$.report.schema_version must be {SUPPORTED_SCHEMA_VERSION}, got {report.get('schema_version')!r}"
        )
    _require_str(report.get("id"), "$.report.id")
    _require_str(report.get("generated_date"), "$.report.generated_date")
    _require_str_list(report.get("limitations"), "$.report.limitations")

    source_refs = _require_object(mapping.get("source_refs"), "$.source_refs")
    for bucket in ["upstream", "fret"]:
        raw_refs = _require_list(source_refs.get(bucket), f"$.source_refs.{bucket}")
        refs = [
            _require_object(item, f"$.source_refs.{bucket}[{index}]")
            for index, item in enumerate(raw_refs)
        ]
        _require_unique_ids(refs, f"$.source_refs.{bucket}")
        for index, ref in enumerate(refs):
            _require_str(ref.get("path"), f"$.source_refs.{bucket}[{index}].path")

    raw_dom_targets = mapping.get("upstream_dom_targets", [])
    dom_targets = [
        _require_object(item, f"$.upstream_dom_targets[{index}]")
        for index, item in enumerate(
            _require_list(raw_dom_targets, "$.upstream_dom_targets")
        )
    ]
    _require_unique_ids(dom_targets, "$.upstream_dom_targets")
    dom_target_ids = {target["id"] for target in dom_targets}
    for index, target in enumerate(dom_targets):
        target_path = f"$.upstream_dom_targets[{index}]"
        _require_str(target.get("snapshot"), f"{target_path}.snapshot")
        _require_str(target.get("theme"), f"{target_path}.theme")
        _require_str(target.get("path"), f"{target_path}.path")
        mode = target.get("mode")
        if mode is not None:
            _require_str(mode, f"{target_path}.mode")
        variant = target.get("variant")
        if variant is not None:
            _require_str(variant, f"{target_path}.variant")
        context_id = target.get("context_id")
        if context_id is not None:
            context_id = _require_str(context_id, f"{target_path}.context_id")
            if context_id not in context_ids:
                raise FixtureError(
                    f"{target_path}.context_id references unknown upstream context id {context_id!r}"
                )
        source_ref_id = target.get("source_ref_id")
        if source_ref_id is not None:
            _require_str(source_ref_id, f"{target_path}.source_ref_id")

    parts = [
        _require_object(item, f"$.parts[{index}]")
        for index, item in enumerate(_require_list(mapping.get("parts"), "$.parts"))
    ]
    _require_unique_ids(parts, "$.parts")

    for part_index, part in enumerate(parts):
        part_path = f"$.parts[{part_index}]"
        _require_str(part.get("label"), f"{part_path}.label")
        _require_str(part.get("axis"), f"{part_path}.axis")
        upstream = _require_object(part.get("upstream"), f"{part_path}.upstream")
        fret = _require_object(part.get("fret"), f"{part_path}.fret")
        _require_str_list(
            upstream.get("source_ref_ids"), f"{part_path}.upstream.source_ref_ids"
        )
        _require_str_list(upstream.get("facts"), f"{part_path}.upstream.facts")
        _validate_upstream_dom_target_refs(
            upstream.get("dom_target_ids"),
            dom_target_ids,
            f"{part_path}.upstream.dom_target_ids",
        )
        _require_str_list(
            fret.get("source_ref_ids"), f"{part_path}.fret.source_ref_ids"
        )
        _require_str_list(fret.get("test_ids"), f"{part_path}.fret.test_ids")
        _require_str_list(fret.get("facts"), f"{part_path}.fret.facts")

        checks = [
            _require_object(item, f"{part_path}.checks[{index}]")
            for index, item in enumerate(
                _require_list(part.get("checks"), f"{part_path}.checks")
            )
        ]
        _require_unique_ids(checks, f"{part_path}.checks")
        for check_index, check in enumerate(checks):
            check_path = f"{part_path}.checks[{check_index}]"
            kind = _require_str(check.get("kind"), f"{check_path}.kind")
            if kind not in SUPPORTED_CHECK_KINDS:
                raise FixtureError(f"{check_path}.kind has unsupported value {kind!r}")
            _require_str(check.get("expected"), f"{check_path}.expected")
            observed = _require_str(check.get("observed"), f"{check_path}.observed")
            if observed not in SUPPORTED_OBSERVED:
                raise FixtureError(f"{check_path}.observed has unsupported value {observed!r}")
            confidence = _require_str(check.get("confidence"), f"{check_path}.confidence")
            if confidence not in SUPPORTED_CONFIDENCE:
                raise FixtureError(f"{check_path}.confidence has unsupported value {confidence!r}")
            _require_str_list(check.get("evidence_refs"), f"{check_path}.evidence_refs")
            promotion = _require_object(check.get("promotion"), f"{check_path}.promotion")
            target = _require_str(promotion.get("target"), f"{check_path}.promotion.target")
            if target not in SUPPORTED_PROMOTION_TARGETS:
                raise FixtureError(f"{check_path}.promotion.target has unsupported value {target!r}")
            _require_str(promotion.get("reason"), f"{check_path}.promotion.reason")
            if "owner" in check:
                owner = _require_str(check.get("owner"), f"{check_path}.owner")
                if owner not in SUPPORTED_OWNER_KINDS:
                    raise FixtureError(f"{check_path}.owner has unsupported value {owner!r}")
            if "layer" in check:
                layer = _require_str(check.get("layer"), f"{check_path}.layer")
                if layer not in SUPPORTED_LAYER_KINDS:
                    raise FixtureError(f"{check_path}.layer has unsupported value {layer!r}")
            _validate_predicates(check, check_path)
            _validate_upstream_predicates(check, check_path)

    return mapping


def _test_ids_for_node(node: dict[str, Any]) -> list[str]:
    result: list[str] = []
    raw_test_id = node.get("test_id")
    if isinstance(raw_test_id, str) and raw_test_id:
        result.append(raw_test_id)
    debug = node.get("debug") if isinstance(node.get("debug"), dict) else {}
    raw_test_id = debug.get("test_id")
    if isinstance(raw_test_id, str) and raw_test_id:
        result.append(raw_test_id)
    decoration = debug.get("semantics_decoration")
    if isinstance(decoration, dict):
        decoration_test_id = decoration.get("test_id")
        if isinstance(decoration_test_id, str) and decoration_test_id:
            result.append(decoration_test_id)
    label = node.get("label")
    if isinstance(label, str):
        result.extend(TEST_ID_LABEL_RE.findall(label))
    return list(dict.fromkeys(result))


def _sidecar_nodes(data: dict[str, Any]) -> list[tuple[int, dict[str, Any]]]:
    taffy = data.get("taffy")
    if not isinstance(taffy, dict):
        raise FixtureError("layout sidecar is missing $.taffy")

    nodes: list[tuple[int, dict[str, Any]]] = []
    for node in taffy.get("nodes", []):
        if isinstance(node, dict):
            nodes.append((0, node))
    roots = taffy.get("roots", [])
    if isinstance(roots, list):
        for root_index, root in enumerate(roots):
            if not isinstance(root, dict):
                continue
            dump = root.get("dump")
            if not isinstance(dump, dict):
                continue
            for node in dump.get("nodes", []):
                if isinstance(node, dict):
                    nodes.append((root_index, node))
    return nodes


def _bundle_semantics_entries(data: dict[str, Any]) -> list[tuple[int, dict[str, Any]]]:
    tables = data.get("tables")
    if not isinstance(tables, dict):
        raise FixtureError("bundle schema2 is missing $.tables")
    semantics_table = tables.get("semantics")
    if not isinstance(semantics_table, dict):
        raise FixtureError("bundle schema2 is missing $.tables.semantics")
    entries = semantics_table.get("entries", [])
    if not isinstance(entries, list):
        raise FixtureError("$.tables.semantics.entries must be a list")

    results: list[tuple[int, dict[str, Any]]] = []
    for entry_index, entry in enumerate(entries):
        if not isinstance(entry, dict):
            continue
        semantics = entry.get("semantics")
        if not isinstance(semantics, dict):
            continue
        nodes = semantics.get("nodes", [])
        if not isinstance(nodes, list):
            continue
        for node in nodes:
            if isinstance(node, dict):
                results.append((entry_index, node))
    return results


def _non_empty_dict(value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        return {}
    return {k: v for k, v in value.items() if v not in (None, False, [], {})}


def _semantics_fact_from_bundle_node(node: dict[str, Any]) -> dict[str, Any]:
    flags = _non_empty_dict(node.get("flags"))
    actions = _non_empty_dict(node.get("actions"))
    relations: dict[str, Any] = {}
    for key in ("labelled_by", "described_by", "controls"):
        value = node.get(key)
        if isinstance(value, list) and value:
            relations[key] = value

    fact: dict[str, Any] = {
        "role": node.get("role"),
        "label": node.get("label"),
        "value": node.get("value"),
        "flags": flags,
        "actions": actions,
        "relations": relations,
        "active_descendant": node.get("active_descendant"),
        "text_selection": node.get("text_selection"),
        "text_composition": node.get("text_composition"),
        "pos_in_set": node.get("pos_in_set"),
        "set_size": node.get("set_size"),
        "level": node.get("level"),
        "scroll": _non_empty_dict(node.get("scroll")),
        "source": "bundle_schema2_semantics",
    }
    return {k: v for k, v in fact.items() if v not in (None, [], {})}


def _bundle_text_paint_entries(data: dict[str, Any]) -> list[dict[str, Any]]:
    tables = data.get("tables")
    if not isinstance(tables, dict):
        return []
    text_paint_table = tables.get("text_paint")
    if not isinstance(text_paint_table, dict):
        return []
    entries = text_paint_table.get("entries", [])
    if not isinstance(entries, list):
        raise FixtureError("$.tables.text_paint.entries must be a list")
    return [entry for entry in entries if isinstance(entry, dict)]


def _bundle_text_paint_fact_rows(
    data: dict[str, Any], bundle_path: str
) -> dict[str, list[dict[str, Any]]]:
    rows_by_node: dict[str, list[dict[str, Any]]] = {}
    for entry in _bundle_text_paint_entries(data):
        base = {
            "window": entry.get("window"),
            "frame_id": entry.get("frame_id"),
            "window_snapshot_seq": entry.get("window_snapshot_seq"),
            "bundle_schema2_path": bundle_path,
            "source": "bundle_schema2_text_paint",
        }
        for key in (
            "widget_measure_hotspots",
            "paint_widget_hotspots",
            "paint_text_prepare_hotspots",
        ):
            items = entry.get(key)
            if not isinstance(items, list):
                continue
            for item in items:
                if not isinstance(item, dict):
                    continue
                node_id = item.get("node")
                if not isinstance(node_id, int | str):
                    continue
                fact = dict(base)
                fact["kind"] = key
                fact.update(item)
                rows_by_node.setdefault(str(node_id), []).append(fact)
        text_input = entry.get("text_input")
        if isinstance(text_input, dict):
            fact = dict(base)
            fact["kind"] = "text_input"
            fact.update(text_input)
            rows_by_node.setdefault("window_text_input", []).append(fact)
        render_text = entry.get("render_text")
        if isinstance(render_text, dict):
            fact = dict(base)
            fact["kind"] = "render_text"
            fact.update(render_text)
            rows_by_node.setdefault("window_render_text", []).append(fact)
    return rows_by_node


def _merge_text_paint_facts(
    target: dict[str, list[dict[str, Any]]], rows: dict[str, list[dict[str, Any]]]
) -> None:
    for node_id, facts in rows.items():
        target.setdefault(node_id, []).extend(facts)


def _bundle_semantics_nodes_by_id(data: dict[str, Any]) -> dict[int, dict[str, Any]]:
    nodes_by_id: dict[int, dict[str, Any]] = {}
    try:
        entries = _bundle_semantics_entries(data)
    except FixtureError:
        return nodes_by_id
    for _, node in entries:
        node_id = node.get("id")
        if isinstance(node_id, int):
            nodes_by_id[node_id] = node
    return nodes_by_id


def _semantics_ancestor_ids(
    node_id: int, nodes_by_id: dict[int, dict[str, Any]]
) -> list[int]:
    ancestors: list[int] = []
    seen = {node_id}
    current = nodes_by_id.get(node_id, {}).get("parent")
    while isinstance(current, int) and current not in seen:
        ancestors.append(current)
        seen.add(current)
        current = nodes_by_id.get(current, {}).get("parent")
    return ancestors


def _preview_label(value: Any) -> str | None:
    if not isinstance(value, str) or not value:
        return None
    return value[:120]


def _associate_text_paint_rows_to_semantics_ancestors(
    data: dict[str, Any],
    rows_by_node: dict[str, list[dict[str, Any]]],
) -> tuple[dict[str, list[dict[str, Any]]], int, int]:
    nodes_by_id = _bundle_semantics_nodes_by_id(data)
    if not nodes_by_id:
        row_count = sum(len(rows) for rows in rows_by_node.values())
        return {}, 0, row_count

    associated: dict[str, list[dict[str, Any]]] = {}
    matched_source_nodes: set[str] = set()
    total_source_nodes = set(rows_by_node)
    for source_node_id, rows in rows_by_node.items():
        try:
            numeric_source_id = int(source_node_id)
        except ValueError:
            continue
        source_node = nodes_by_id.get(numeric_source_id)
        if source_node is None:
            continue
        source_role = source_node.get("role")
        source_label = _preview_label(source_node.get("label"))
        for ancestor_id in _semantics_ancestor_ids(numeric_source_id, nodes_by_id):
            ancestor_node = nodes_by_id.get(ancestor_id)
            if ancestor_node is None:
                continue
            ancestor_test_ids = _test_ids_for_node(ancestor_node)
            if not ancestor_test_ids:
                continue
            matched_source_nodes.add(source_node_id)
            for row in rows:
                associated_row = dict(row)
                associated_row["association_kind"] = "semantics_descendant"
                associated_row["associated_source_role"] = source_role
                if source_label is not None:
                    associated_row["associated_source_label_preview"] = source_label
                associated_row["associated_ancestor_test_ids"] = ancestor_test_ids
                associated_row["associated_ancestor_role"] = ancestor_node.get("role")
                ancestor_label = _preview_label(ancestor_node.get("label"))
                if ancestor_label is not None:
                    associated_row["associated_ancestor_label_preview"] = ancestor_label
                associated.setdefault(str(ancestor_id), []).append(associated_row)

    associated_count = sum(len(rows) for rows in associated.values())
    unassociated_count = sum(
        len(rows)
        for node_id, rows in rows_by_node.items()
        if node_id in total_source_nodes and node_id not in matched_source_nodes
    )
    return associated, associated_count, unassociated_count


def _bundle_text_label_rows_by_test_id_ancestor(
    data: dict[str, Any], bundle_path: str
) -> dict[str, list[dict[str, Any]]]:
    nodes_by_id = _bundle_semantics_nodes_by_id(data)
    label_rows: dict[str, list[dict[str, Any]]] = {}
    for node_id, node in nodes_by_id.items():
        if node.get("role") != "text":
            continue
        label = _preview_label(node.get("label"))
        if label is None:
            continue
        rect = node.get("bounds")
        for ancestor_id in _semantics_ancestor_ids(node_id, nodes_by_id):
            ancestor_node = nodes_by_id.get(ancestor_id)
            if ancestor_node is None:
                continue
            ancestor_test_ids = _test_ids_for_node(ancestor_node)
            if not ancestor_test_ids:
                continue
            row: dict[str, Any] = {
                "source": "bundle_schema2_semantics_text_descendant",
                "node": str(node_id),
                "role": "text",
                "label_preview": label,
                "ancestor_test_ids": ancestor_test_ids,
                "ancestor_role": ancestor_node.get("role"),
                "bundle_schema2_path": bundle_path,
            }
            ancestor_label = _preview_label(ancestor_node.get("label"))
            if ancestor_label is not None:
                row["ancestor_label_preview"] = ancestor_label
            if isinstance(rect, dict):
                row["bounds"] = Bounds.from_rect(rect).to_json()
            label_rows.setdefault(str(ancestor_id), []).append(row)
    return label_rows


def _bundle_semantics_roots(
    data: dict[str, Any],
    bundle_path: str,
    scale_factor: float,
    coordinate_units: str,
) -> list[LayoutRoot]:
    tables = data.get("tables")
    if not isinstance(tables, dict):
        return []
    semantics_table = tables.get("semantics")
    if not isinstance(semantics_table, dict):
        return []
    entries = semantics_table.get("entries", [])
    if not isinstance(entries, list):
        return []

    roots: list[LayoutRoot] = []
    for entry in entries:
        if not isinstance(entry, dict):
            continue
        semantics = entry.get("semantics")
        if not isinstance(semantics, dict):
            continue
        nodes = semantics.get("nodes", [])
        if not isinstance(nodes, list):
            continue
        nodes_by_id: dict[int, dict[str, Any]] = {}
        for node in nodes:
            if not isinstance(node, dict):
                continue
            node_id = node.get("id")
            if isinstance(node_id, int):
                nodes_by_id[node_id] = node
        raw_roots = semantics.get("roots", [])
        if not isinstance(raw_roots, list):
            continue
        for root in raw_roots:
            if not isinstance(root, dict):
                continue
            root_id = root.get("root")
            if not isinstance(root_id, int):
                continue
            node = nodes_by_id.get(root_id)
            if node is None:
                continue
            rect = node.get("bounds")
            if not isinstance(rect, dict):
                continue
            roots.append(
                LayoutRoot(
                    bounds=Bounds.from_rect(rect),
                    raw_bounds=Bounds.from_rect(rect),
                    scale_factor=scale_factor,
                    coordinate_units=coordinate_units,
                    sidecar_path=bundle_path,
                    source="bundle_schema2_semantics",
                )
            )
    return roots


def load_layout_evidence(
    sidecar_paths: list[Path],
    bundle_paths: list[Path] | None = None,
) -> LayoutEvidence:
    evidence = LayoutEvidence.empty()
    for path in sorted(sidecar_paths):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            raise FixtureError(f"{path}: invalid layout sidecar JSON: {exc}") from exc
        meta = data.get("meta", {}) if isinstance(data.get("meta"), dict) else {}
        scale_factor = float(meta.get("scale_factor") or 1.0)
        coordinate_units = str(meta.get("coordinate_units") or "logical_px")
        sidecar_path = str(path).replace("\\", "/")
        evidence.sidecar_paths.append(sidecar_path)
        root_bounds = meta.get("root_bounds")
        if isinstance(root_bounds, dict):
            evidence.roots.append(
                LayoutRoot(
                    bounds=Bounds.from_rect(root_bounds),
                    raw_bounds=Bounds.from_rect(root_bounds),
                    scale_factor=scale_factor,
                    coordinate_units=coordinate_units,
                    sidecar_path=sidecar_path,
                    source="layout_sidecar",
                )
            )
        for root_index, node in _sidecar_nodes(data):
            rect = node.get("abs_rect")
            if not isinstance(rect, dict):
                continue
            debug = node.get("debug") if isinstance(node.get("debug"), dict) else {}
            for test_id in _test_ids_for_node(node):
                evidence.nodes_by_test_id.setdefault(test_id, []).append(
                    LayoutNode(
                        test_id=test_id,
                        bounds=Bounds.from_rect(rect),
                        raw_bounds=Bounds.from_rect(rect),
                        scale_factor=scale_factor,
                        coordinate_units=coordinate_units,
                        node=str(node.get("node", "")),
                        sidecar_path=sidecar_path,
                        root_index=root_index,
                        kind=debug.get("instance_kind")
                        if isinstance(debug.get("instance_kind"), str)
                        else None,
                        label=node.get("label") if isinstance(node.get("label"), str) else None,
                        source="layout_sidecar",
                    )
                )

    for path in sorted(bundle_paths or []):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            raise FixtureError(f"{path}: invalid bundle schema2 JSON: {exc}") from exc
        if data.get("schema_version") != 2:
            raise FixtureError(
                f"{path}: bundle schema2 must declare schema_version 2, got {data.get('schema_version')!r}"
            )
        bundle_path = str(path).replace("\\", "/")
        evidence.bundle_paths.append(bundle_path)
        env = data.get("env") if isinstance(data.get("env"), dict) else {}
        scale_factors_seen = env.get("scale_factors_seen") if isinstance(env, dict) else []
        scale_factor = 1.0
        if isinstance(scale_factors_seen, list) and scale_factors_seen:
            first_scale = scale_factors_seen[0]
            if isinstance(first_scale, int | float):
                scale_factor = float(first_scale)
        coordinate_units = "logical_px"
        evidence.roots.extend(
            _bundle_semantics_roots(data, bundle_path, scale_factor, coordinate_units)
        )
        text_paint_rows = _bundle_text_paint_fact_rows(data, bundle_path)
        evidence.text_paint_bundle_entry_count += len(_bundle_text_paint_entries(data))
        evidence.text_paint_fact_row_count += sum(
            len(rows) for rows in text_paint_rows.values()
        )
        _merge_text_paint_facts(
            evidence.text_paint_facts_by_node,
            text_paint_rows,
        )
        associated_rows, associated_count, unassociated_count = (
            _associate_text_paint_rows_to_semantics_ancestors(data, text_paint_rows)
        )
        _merge_text_paint_facts(
            evidence.text_paint_associated_facts_by_node,
            associated_rows,
        )
        evidence.text_paint_association_row_count += associated_count
        evidence.text_paint_unassociated_row_count += unassociated_count
        text_label_rows = _bundle_text_label_rows_by_test_id_ancestor(data, bundle_path)
        _merge_text_paint_facts(
            evidence.text_label_facts_by_node,
            text_label_rows,
        )
        evidence.text_label_fact_row_count += sum(
            len(rows) for rows in text_label_rows.values()
        )
        for root_index, node in _bundle_semantics_entries(data):
            rect = node.get("bounds")
            if not isinstance(rect, dict):
                continue
            for test_id in _test_ids_for_node(node):
                evidence.bundle_nodes_by_test_id.setdefault(test_id, []).append(
                    LayoutNode(
                        test_id=test_id,
                        bounds=Bounds.from_rect(rect),
                        raw_bounds=Bounds.from_rect(rect),
                        scale_factor=scale_factor,
                        coordinate_units=coordinate_units,
                        node=str(node.get("id", "")),
                        sidecar_path=bundle_path,
                        root_index=root_index,
                        kind=str(node.get("role"))
                        if isinstance(node.get("role"), str)
                        else None,
                        label=node.get("label") if isinstance(node.get("label"), str) else None,
                        semantics=_semantics_fact_from_bundle_node(node),
                        source="bundle_schema2_semantics",
                    )
                )

    for nodes in evidence.nodes_by_test_id.values():
        nodes.sort(key=lambda n: (n.sidecar_path, n.root_index, n.node))
    for nodes in evidence.bundle_nodes_by_test_id.values():
        nodes.sort(key=lambda n: (n.sidecar_path, n.root_index, n.node))
    return evidence


def _walk_dom_nodes(node: dict[str, Any]) -> list[dict[str, Any]]:
    nodes = [node]
    for child in node.get("children", []):
        if isinstance(child, dict):
            nodes.extend(_walk_dom_nodes(child))
    return nodes


def _snapshot_dom_nodes(theme_data: dict[str, Any]) -> dict[str, dict[str, Any]]:
    nodes: dict[str, dict[str, Any]] = {}
    root = theme_data.get("root")
    if isinstance(root, dict):
        for node in _walk_dom_nodes(root):
            path = node.get("path")
            if isinstance(path, str):
                nodes.setdefault(path, node)
    for bucket in ["portals", "portalWrappers"]:
        raw_items = theme_data.get(bucket)
        if not isinstance(raw_items, list):
            continue
        for item in raw_items:
            if not isinstance(item, dict):
                continue
            for node in _walk_dom_nodes(item):
                path = node.get("path")
                if isinstance(path, str):
                    nodes.setdefault(path, node)
    return nodes


def _dom_context_from_snapshot(
    snapshot_path: str,
    snapshot_name: str,
    theme: str,
    snapshot_mode: str,
    snapshot_variant: str,
    context_id: str | None,
    theme_data: dict[str, Any],
) -> dict[str, Any]:
    raw_viewport = theme_data.get("viewport")
    viewport: dict[str, Any] = {}
    if isinstance(raw_viewport, dict):
        width = raw_viewport.get("w")
        height = raw_viewport.get("h")
        if isinstance(width, int | float):
            viewport["width_px"] = width
        if isinstance(height, int | float):
            viewport["height_px"] = height

    return {
        "snapshot": snapshot_name,
        "theme": theme,
        "mode": snapshot_mode,
        "variant": snapshot_variant,
        **({"context_id": context_id} if context_id is not None else {}),
        "viewport": viewport,
        "device_pixel_ratio": float(theme_data.get("devicePixelRatio") or 1.0),
        "snapshot_path": snapshot_path,
    }


def _dom_node_from_snapshot_node(
    node: dict[str, Any],
    target_id: str,
    snapshot_path: str,
    snapshot_name: str,
    theme: str,
    snapshot_mode: str,
    snapshot_variant: str,
    context_id: str | None,
    device_pixel_ratio: float,
    viewport: dict[str, Any],
    path: str,
) -> DomNode | None:
    rect = node.get("rect")
    if not isinstance(rect, dict):
        return None
    attrs = node.get("attrs") if isinstance(node.get("attrs"), dict) else {}
    class_name = node.get("className")
    text = node.get("text")
    computed_style = (
        node.get("computedStyle") if isinstance(node.get("computedStyle"), dict) else {}
    )
    children = node.get("children") if isinstance(node.get("children"), list) else []
    return DomNode(
        target_id=target_id,
        bounds=Bounds.from_rect(rect),
        device_pixel_ratio=device_pixel_ratio,
        snapshot_path=snapshot_path,
        snapshot_name=snapshot_name,
        theme=theme,
        mode=snapshot_mode,
        variant=snapshot_variant,
        context_id=context_id,
        viewport=viewport,
        path=path,
        tag=str(node.get("tag", "")),
        attrs=attrs,
        class_name=class_name if isinstance(class_name, str) else None,
        text=text if isinstance(text, str) else None,
        computed_style=computed_style,
        child_count=len(children),
    )


def load_dom_evidence(
    paths: list[Path],
    targets: list[dict[str, Any]],
    contexts: list[dict[str, Any]],
) -> DomEvidence:
    evidence = DomEvidence.empty()
    targets_by_snapshot: dict[tuple[str, str, str, str], list[dict[str, Any]]] = {}
    for target in targets:
        targets_by_snapshot.setdefault(
            (
                target["snapshot"],
                target["theme"],
                target.get("mode") or "",
                target.get("variant") or "",
            ),
            [],
        ).append(target)

    for path in sorted(paths):
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError as exc:
            raise FixtureError(f"{path}: invalid upstream DOM snapshot JSON: {exc}") from exc
        snapshot_name = _require_str(data.get("name"), f"{path}.name")
        snapshot_mode = data.get("mode")
        if snapshot_mode is not None:
            snapshot_mode = _require_str(snapshot_mode, f"{path}.mode")
        else:
            snapshot_mode = ""
        snapshot_variant = data.get("variant")
        if snapshot_variant is not None:
            snapshot_variant = _require_str(snapshot_variant, f"{path}.variant")
        else:
            snapshot_variant = ""
        themes = _require_object(data.get("themes"), f"{path}.themes")
        snapshot_path = str(path).replace("\\", "/")
        evidence.snapshot_paths.append(snapshot_path)
        for theme, raw_theme_data in themes.items():
            if not isinstance(raw_theme_data, dict):
                continue
            matched_contexts = [
                context
                for context in contexts
                if _context_matches_snapshot(
                    context,
                    snapshot_name,
                    theme,
                    snapshot_mode,
                    snapshot_variant,
                    raw_theme_data,
                )
            ]
            wanted_targets = targets_by_snapshot.get(
                (snapshot_name, theme, snapshot_mode, snapshot_variant), []
            )
            if not wanted_targets:
                continue
            matched_context_ids = {context["id"] for context in matched_contexts}
            if len(matched_contexts) > 1:
                ambiguous_targets = [
                    target
                    for target in wanted_targets
                    if target.get("context_id") is None
                ]
                if ambiguous_targets:
                    raise FixtureError(
                        f"{path}: ambiguous upstream DOM match for {snapshot_name!r} "
                        f"theme={theme!r} mode={snapshot_mode!r} variant={snapshot_variant!r}; "
                        "add context_id to each upstream_dom_target for this snapshot family"
                    )
            matched_targets = []
            for target in wanted_targets:
                target_context_id = target.get("context_id")
                if target_context_id is not None and target_context_id not in matched_context_ids:
                    raise FixtureError(
                        f"{path}: upstream_dom_target {target['id']!r} references "
                        f"context_id {target_context_id!r} but no matching upstream_context was found"
                    )
                matched_targets.append(target)
            if not matched_targets:
                continue
            if matched_contexts:
                for context in matched_contexts:
                    evidence.contexts.append(
                        _dom_context_from_snapshot(
                            snapshot_path,
                            snapshot_name,
                            theme,
                            snapshot_mode,
                            snapshot_variant,
                            context["id"],
                            raw_theme_data,
                        )
                    )
            else:
                evidence.contexts.append(
                    _dom_context_from_snapshot(
                        snapshot_path,
                        snapshot_name,
                        theme,
                        snapshot_mode,
                        snapshot_variant,
                        None,
                        raw_theme_data,
                    )
                )
            nodes_by_path = _snapshot_dom_nodes(raw_theme_data)
            device_pixel_ratio = float(raw_theme_data.get("devicePixelRatio") or 1.0)
            viewport = (
                raw_theme_data.get("viewport")
                if isinstance(raw_theme_data.get("viewport"), dict)
                else {}
            )
            for node_path, node in nodes_by_path.items():
                dom_node = _dom_node_from_snapshot_node(
                    node,
                    f"{snapshot_name}:{theme}:{snapshot_mode}:{snapshot_variant}:{node_path}",
                    snapshot_path,
                    snapshot_name,
                    theme,
                    snapshot_mode,
                    snapshot_variant,
                    None,
                    device_pixel_ratio,
                    viewport,
                    node_path,
                )
                if dom_node is not None:
                    evidence.nodes_by_snapshot_path[
                        (snapshot_name, theme, snapshot_mode, snapshot_variant, node_path)
                    ] = dom_node
            for target in matched_targets:
                node = nodes_by_path.get(target["path"])
                if node is None:
                    continue
                dom_node = _dom_node_from_snapshot_node(
                    node,
                    target["id"],
                    snapshot_path,
                    snapshot_name,
                    theme,
                    snapshot_mode,
                    snapshot_variant,
                    target.get("context_id"),
                    device_pixel_ratio,
                    viewport,
                    target["path"],
                )
                if dom_node is None:
                    continue
                evidence.nodes_by_target_id[target["id"]] = dom_node
    return evidence


def _comparison_passes(
    observed: float, comparison: str, predicate: dict[str, Any]
) -> tuple[bool, str]:
    eps = float(predicate.get("eps_px", 0.0))
    if comparison == "between":
        min_px = float(predicate["min_px"])
        max_px = float(predicate["max_px"])
        return min_px - eps <= observed <= max_px + eps, f"{min_px:g}..{max_px:g}"
    value_px = float(predicate["value_px"])
    if comparison == "gte":
        return observed + eps >= value_px, f">= {value_px:g}"
    if comparison == "lte":
        return observed - eps <= value_px, f"<= {value_px:g}"
    if comparison == "eq":
        return abs(observed - value_px) <= eps, f"== {value_px:g}"
    raise FixtureError(f"unsupported comparison {comparison!r}")


def evaluate_predicate(
    predicate: dict[str, Any], evidence: LayoutEvidence
) -> dict[str, Any]:
    kind = predicate["kind"]
    metric = predicate["metric"]
    comparison = predicate["comparison"]
    evidence_source = predicate.get("evidence_source")
    if evidence_source == "auto":
        evidence_source = None
    if kind == "root_metric":
        root = evidence.find_root(evidence_source)
        if root is None:
            return {
                "kind": kind,
                "status": "missing",
                "metric": metric,
                "reason": "missing_root_bounds",
                **(
                    {"requested_evidence_source": evidence_source}
                    if evidence_source is not None
                    else {}
                ),
            }
        observed = root.bounds.metric(metric)
        passed, expected = _comparison_passes(observed, comparison, predicate)
        return {
            "kind": kind,
            "status": "pass" if passed else "fail",
            "target": "layout_root",
            "metric": metric,
            "comparison": comparison,
            "expected": expected,
            "eps_px": float(predicate.get("eps_px", 0.0)),
            "observed_px": round(observed, 3),
            "bounds": root.bounds.to_json(),
            "raw_bounds": root.raw_bounds.to_json(),
            "scale_factor": root.scale_factor,
            "coordinate_units": root.coordinate_units,
            "evidence_source": root.source,
            **(
                {"requested_evidence_source": evidence_source}
                if evidence_source is not None
                else {}
            ),
            **(
                {"sidecar_path": root.sidecar_path}
                if root.source == "layout_sidecar"
                else {"bundle_schema2_path": root.sidecar_path}
            ),
            "duplicate_count": len(evidence.roots),
        }
    if kind == "bounds_metric":
        target = predicate["target"]
        node = evidence.find(target, evidence_source)
        if node is None:
            return {
                "kind": kind,
                "status": "missing",
                "target": target,
                "metric": metric,
                "reason": "missing_test_id",
                **(
                    {"requested_evidence_source": evidence_source}
                    if evidence_source is not None
                    else {}
                ),
            }
        observed = node.bounds.metric(metric)
        passed, expected = _comparison_passes(observed, comparison, predicate)
        return {
            "kind": kind,
            "status": "pass" if passed else "fail",
            "target": target,
            "metric": metric,
            "comparison": comparison,
            "expected": expected,
            "eps_px": float(predicate.get("eps_px", 0.0)),
            "observed_px": round(observed, 3),
            "bounds": node.bounds.to_json(),
            "raw_bounds": node.raw_bounds.to_json(),
            "scale_factor": node.scale_factor,
            "coordinate_units": node.coordinate_units,
            "node": node.node,
            "kind_hint": node.kind,
            "evidence_source": node.source,
            **(
                {"requested_evidence_source": evidence_source}
                if evidence_source is not None
                else {}
            ),
            **(
                {"sidecar_path": node.sidecar_path}
                if node.source == "layout_sidecar"
                else {"bundle_schema2_path": node.sidecar_path}
            ),
            "duplicate_count": evidence.duplicate_count(target, evidence_source),
        }
    if kind == "bounds_metric_delta":
        a_id = predicate["a"]
        b_id = predicate["b"]
        a_node = evidence.find(a_id, evidence_source)
        b_node = evidence.find(b_id, evidence_source)
        if a_node is None or b_node is None:
            return {
                "kind": kind,
                "status": "missing",
                "a": a_id,
                "b": b_id,
                "metric": metric,
                "reason": "missing_test_id",
                **(
                    {"requested_evidence_source": evidence_source}
                    if evidence_source is not None
                    else {}
                ),
            }
        observed = a_node.bounds.metric(metric) - b_node.bounds.metric(metric)
        passed, expected = _comparison_passes(observed, comparison, predicate)
        return {
            "kind": kind,
            "status": "pass" if passed else "fail",
            "a": a_id,
            "b": b_id,
            "metric": metric,
            "comparison": comparison,
            "expected": expected,
            "eps_px": float(predicate.get("eps_px", 0.0)),
            "observed_px": round(observed, 3),
            "a_bounds": a_node.bounds.to_json(),
            "b_bounds": b_node.bounds.to_json(),
            "a_raw_bounds": a_node.raw_bounds.to_json(),
            "b_raw_bounds": b_node.raw_bounds.to_json(),
            "scale_factors": sorted({a_node.scale_factor, b_node.scale_factor}),
            "coordinate_units": sorted({a_node.coordinate_units, b_node.coordinate_units}),
            "a_node": a_node.node,
            "b_node": b_node.node,
            "evidence_sources": sorted({a_node.source, b_node.source}),
            **(
                {"requested_evidence_source": evidence_source}
                if evidence_source is not None
                else {}
            ),
            "evidence_paths": sorted({a_node.sidecar_path, b_node.sidecar_path}),
            **(
                {"sidecar_path": a_node.sidecar_path}
                if a_node.source == "layout_sidecar" and b_node.source == "layout_sidecar"
                else {}
            ),
            "a_duplicate_count": evidence.duplicate_count(a_id, evidence_source),
            "b_duplicate_count": evidence.duplicate_count(b_id, evidence_source),
        }
    raise FixtureError(f"unsupported predicate kind {kind!r}")


def evaluate_dom_predicate(
    predicate: dict[str, Any], evidence: DomEvidence
) -> dict[str, Any]:
    kind = predicate["kind"]
    metric = predicate["metric"]
    comparison = predicate["comparison"]
    if kind == "bounds_metric":
        target = predicate["target"]
        node = evidence.find(target)
        if node is None:
            return {
                "kind": kind,
                "status": "missing",
                "target": target,
                "metric": metric,
                "reason": "missing_upstream_dom_target",
            }
        observed = node.bounds.metric(metric)
        passed, expected = _comparison_passes(observed, comparison, predicate)
        return {
            "kind": kind,
            "status": "pass" if passed else "fail",
            "target": target,
            "metric": metric,
            "comparison": comparison,
            "expected": expected,
            "eps_px": float(predicate.get("eps_px", 0.0)),
            "observed_px": round(observed, 3),
            "bounds": node.bounds.to_json(),
            "device_pixel_ratio": node.device_pixel_ratio,
            "snapshot_path": node.snapshot_path,
            "snapshot_name": node.snapshot_name,
            "theme": node.theme,
            "mode": node.mode,
            "variant": node.variant,
            "context_id": node.context_id,
            "viewport": node.viewport,
            "path": node.path,
            "tag": node.tag,
            "attrs": node.attrs,
            "class_name": node.class_name,
        }
    if kind == "bounds_metric_delta":
        a_id = predicate["a"]
        b_id = predicate["b"]
        a_node = evidence.find(a_id)
        b_node = evidence.find(b_id)
        if a_node is None or b_node is None:
            return {
                "kind": kind,
                "status": "missing",
                "a": a_id,
                "b": b_id,
                "metric": metric,
                "reason": "missing_upstream_dom_target",
            }
        observed = a_node.bounds.metric(metric) - b_node.bounds.metric(metric)
        passed, expected = _comparison_passes(observed, comparison, predicate)
        return {
            "kind": kind,
            "status": "pass" if passed else "fail",
            "a": a_id,
            "b": b_id,
            "metric": metric,
            "comparison": comparison,
            "expected": expected,
            "eps_px": float(predicate.get("eps_px", 0.0)),
            "observed_px": round(observed, 3),
            "a_bounds": a_node.bounds.to_json(),
            "b_bounds": b_node.bounds.to_json(),
            "device_pixel_ratios": sorted(
                {a_node.device_pixel_ratio, b_node.device_pixel_ratio}
            ),
            "snapshot_path": a_node.snapshot_path,
            "snapshot_name": a_node.snapshot_name,
            "theme": a_node.theme,
            "mode": a_node.mode,
            "variant": a_node.variant,
            "context_id": a_node.context_id,
            "a_snapshot_path": a_node.snapshot_path,
            "b_snapshot_path": b_node.snapshot_path,
            "a_path": a_node.path,
            "b_path": b_node.path,
        }
    raise FixtureError(f"unsupported predicate kind {kind!r}")


def _measurement_status(predicate_results: list[dict[str, Any]]) -> str:
    if any(result["status"] == "missing" for result in predicate_results):
        return "missing"
    if any(result["status"] == "fail" for result in predicate_results):
        return "fail"
    return "pass"


def evaluate_fret_measurement(
    check: dict[str, Any], evidence: LayoutEvidence
) -> dict[str, Any] | None:
    predicates = check.get("predicates")
    if not predicates:
        return None
    if not evidence.sidecar_paths and not evidence.bundle_paths:
        return {
            "source": "fret_layout_sidecar",
            "status": "missing",
            "reason": "no_layout_evidence_provided",
            "predicate_count": len(predicates),
            "predicates": [],
        }

    predicate_results = [
        evaluate_predicate(predicate, evidence) for predicate in predicates
    ]
    return {
        "source": (
            "fret_layout_sidecar"
            if not evidence.bundle_paths
            else "fret_layout_sidecar+bundle_schema2_semantics"
        ),
        "status": _measurement_status(predicate_results),
        "sidecar_paths": evidence.sidecar_paths,
        **({"bundle_schema2_paths": evidence.bundle_paths} if evidence.bundle_paths else {}),
        "predicate_count": len(predicate_results),
        "predicates": predicate_results,
    }


def evaluate_upstream_dom_measurement(
    check: dict[str, Any], evidence: DomEvidence
) -> dict[str, Any] | None:
    predicates = check.get("upstream_predicates")
    if not predicates:
        return None
    if not evidence.snapshot_paths:
        return {
            "source": "upstream_dom_snapshot",
            "status": "missing",
            "reason": "no_upstream_dom_snapshots_provided",
            "predicate_count": len(predicates),
            "predicates": [],
        }

    predicate_results = [
        evaluate_dom_predicate(predicate, evidence) for predicate in predicates
    ]
    return {
        "source": "upstream_dom_snapshot",
        "status": _measurement_status(predicate_results),
        "snapshot_paths": evidence.snapshot_paths,
        "predicate_count": len(predicate_results),
        "predicates": predicate_results,
    }


def _comparison_hint(
    fret_predicate: dict[str, Any], upstream_predicate: dict[str, Any]
) -> dict[str, Any] | None:
    if fret_predicate.get("status") == "missing" or upstream_predicate.get("status") == "missing":
        return None
    if "observed_px" not in fret_predicate or "observed_px" not in upstream_predicate:
        return None

    upstream_px = float(upstream_predicate["observed_px"])
    fret_logical_px = float(fret_predicate["observed_px"])
    result: dict[str, Any] = {
        "metric": fret_predicate.get("metric"),
        "upstream_observed_px": round(upstream_px, 3),
        "fret_logical_observed_px": round(fret_logical_px, 3),
        "logical_delta_px": round(fret_logical_px - upstream_px, 3),
    }
    raw_bounds = fret_predicate.get("raw_bounds")
    metric = fret_predicate.get("metric")
    raw_px = None
    if isinstance(raw_bounds, dict) and isinstance(metric, str):
        raw_key = "w" if metric == "width" else "h" if metric == "height" else None
        if raw_key and raw_key in raw_bounds:
            raw_px = float(raw_bounds[raw_key])
    if raw_px is not None:
        raw_delta = raw_px - upstream_px
        result["fret_raw_observed_px"] = round(raw_px, 3)
        result["raw_delta_px"] = round(raw_delta, 3)
        eps = max(
            float(fret_predicate.get("eps_px", 0.0)),
            float(upstream_predicate.get("eps_px", 0.0)),
            1.0,
        )
        if abs(raw_delta) <= eps and abs(fret_logical_px - upstream_px) > eps:
            result["classification_hint"] = "diagnostics_unit_contract"
            result["reason"] = (
                "Fret raw sidecar bounds match upstream DOM CSS px while predicate "
                "bounds diverge; check whether a stale reader transformed sidecar "
                "coordinates."
            )
    return result


def combine_measurements(
    fret: dict[str, Any] | None, upstream: dict[str, Any] | None
) -> dict[str, Any] | None:
    if fret is None and upstream is None:
        return None
    if fret is None:
        return upstream
    if upstream is None:
        return fret

    comparisons: list[dict[str, Any]] = []
    used_upstream: set[int] = set()
    for fret_predicate in fret.get("predicates", []):
        upstream_predicate = None
        for index, candidate in enumerate(upstream.get("predicates", [])):
            if index in used_upstream:
                continue
            if (
                candidate.get("kind") == fret_predicate.get("kind")
                and candidate.get("metric") == fret_predicate.get("metric")
            ):
                upstream_predicate = candidate
                used_upstream.add(index)
                break
        if upstream_predicate is None:
            continue
        hint = _comparison_hint(fret_predicate, upstream_predicate)
        if hint is not None:
            comparisons.append(hint)

    if upstream["status"] in {"missing", "fail"}:
        status = upstream["status"]
    else:
        status = fret["status"]
    return {
        "source": f"{fret['source']}+{upstream['source']}",
        "status": status,
        "fret": fret,
        "upstream_dom": upstream,
        "comparisons": comparisons,
    }


def evaluate_live_fact_measurement(
    check: dict[str, Any], live_facts: dict[str, Any]
) -> dict[str, Any] | None:
    requirements = check.get("live_fact_requirements")
    if not isinstance(requirements, dict):
        return None

    results = []
    for field, minimum in sorted(requirements.items()):
        if not isinstance(field, str) or not isinstance(minimum, int | float):
            continue
        observed = live_facts.get(field, 0)
        if not isinstance(observed, int | float):
            observed = 0
        results.append(
            {
                "field": field,
                "observed": observed,
                "minimum": minimum,
                "status": "pass" if observed >= minimum else "missing",
            }
        )

    if not results:
        return None
    return {
        "source": "live_fact_requirements",
        "status": _measurement_status(results),
        "part_id": live_facts["part_id"],
        "requirements": results,
    }


def _px_value(value: Any) -> float | None:
    if not isinstance(value, str) or PX_VALUE_RE.match(value) is None:
        return None
    return round(float(value[:-2]), 3)


def _style_px_map(style: dict[str, Any], names: list[str]) -> dict[str, float]:
    result: dict[str, float] = {}
    for name in names:
        value = _px_value(style.get(name))
        if value is not None:
            result[name] = value
    return result


def _style_string_map(style: dict[str, Any], names: list[str]) -> dict[str, str]:
    result: dict[str, str] = {}
    for name in names:
        value = style.get(name)
        if isinstance(value, str):
            result[name] = value
    return result


def _class_tokens(class_name: str | None) -> list[str]:
    if not class_name:
        return []
    return [token for token in class_name.split() if token]


def _attr_string(attrs: dict[str, Any], key: str) -> str | None:
    value = attrs.get(key)
    if isinstance(value, str) and value:
        return value
    if isinstance(value, bool):
        return str(value).lower()
    return None


def _dom_implicit_role(node: DomNode) -> str | None:
    tag = node.tag.lower()
    if tag == "button":
        return "button"
    if tag == "input":
        input_type = str(node.attrs.get("type") or "text").lower()
        if input_type in {"button", "submit", "reset"}:
            return "button"
        if input_type == "checkbox":
            return "checkbox"
        if input_type == "radio":
            return "radio"
        return "textbox"
    if tag == "select":
        return "combobox"
    if tag == "textarea":
        return "textbox"
    if tag == "a" and node.attrs.get("href"):
        return "link"
    return None


def _dom_focusable(node: DomNode) -> bool:
    tabindex = _attr_string(node.attrs, "tabindex")
    if tabindex is not None and tabindex != "-1":
        return True
    tag = node.tag.lower()
    if tag in {"button", "input", "select", "textarea"}:
        return not _dom_disabled(node)
    if tag == "a":
        return bool(node.attrs.get("href"))
    role = _attr_string(node.attrs, "role")
    return role in {"button", "checkbox", "combobox", "menuitem", "option", "radio", "switch", "tab"}


def _dom_disabled(node: DomNode) -> bool:
    disabled_attr = node.attrs.get("disabled")
    aria_disabled = _attr_string(node.attrs, "aria-disabled")
    return disabled_attr is True or aria_disabled == "true"


def _dom_semantics_fact(node: DomNode) -> dict[str, Any]:
    attrs = node.attrs
    states = {
        key: _attr_string(attrs, key)
        for key in (
            "aria-expanded",
            "aria-selected",
            "aria-checked",
            "aria-pressed",
            "aria-disabled",
            "aria-invalid",
            "aria-required",
            "aria-current",
        )
        if _attr_string(attrs, key) is not None
    }
    relations = {
        key: _attr_string(attrs, key)
        for key in ("aria-controls", "aria-describedby", "aria-labelledby", "aria-owns")
        if _attr_string(attrs, key) is not None
    }
    accessible_name = (
        _attr_string(attrs, "aria-label")
        or _attr_string(attrs, "placeholder")
        or _attr_string(attrs, "value")
        or node.text
    )
    return {
        "role": _attr_string(attrs, "role") or _dom_implicit_role(node),
        "implicit_role": _dom_implicit_role(node),
        "accessible_name": accessible_name,
        "states": states,
        "relations": relations,
        "tab_index": _attr_string(attrs, "tabindex"),
        "source": "upstream_dom_accessibility_attrs",
    }


def _dom_interaction_fact(node: DomNode) -> dict[str, Any]:
    attrs = node.attrs
    return {
        "focusable": _dom_focusable(node),
        "disabled": _dom_disabled(node),
        "has_popup": _attr_string(attrs, "aria-haspopup"),
        "cursor": node.computed_style.get("cursor"),
        "pointer_events": node.computed_style.get("pointerEvents"),
        "keyboard_target": _dom_focusable(node),
        "source": "upstream_dom_accessibility_attrs",
    }


def _dom_descendants(evidence: DomEvidence, node: DomNode) -> list[DomNode]:
    prefix = f"{node.path}."
    descendants = [
        candidate
        for key, candidate in evidence.nodes_by_snapshot_path.items()
        if key[0] == node.snapshot_name
        and key[1] == node.theme
        and key[2] == node.mode
        and key[3] == node.variant
        and key[4].startswith(prefix)
    ]
    descendants.sort(key=lambda item: item.path)
    return descendants


def _dom_target_facts(node: DomNode, evidence: DomEvidence) -> dict[str, Any]:
    style = node.computed_style
    descendants = _dom_descendants(evidence, node)
    svg_descendants = [item for item in descendants if item.tag.lower() == "svg"]
    text_descendants = [
        item for item in descendants if isinstance(item.text, str) and item.text
    ]
    facts: dict[str, Any] = {
        "target_id": node.target_id,
        "snapshot": node.snapshot_name,
        "theme": node.theme,
        "mode": node.mode,
        "variant": node.variant,
        "context_id": node.context_id,
        "path": node.path,
        "tag": node.tag,
        "attrs": node.attrs,
        "class_tokens": _class_tokens(node.class_name),
        "bounds": node.bounds.to_json(),
        "device_pixel_ratio": node.device_pixel_ratio,
        "child_count": node.child_count,
        "layout": {
            **_style_string_map(
                style,
                [
                    "display",
                    "position",
                    "boxSizing",
                    "alignItems",
                    "justifyContent",
                    "flexDirection",
                    "flexWrap",
                    "flex",
                    "flexGrow",
                    "flexShrink",
                    "gap",
                    "rowGap",
                    "columnGap",
                    "overflow",
                    "whiteSpace",
                ],
            ),
            **_style_px_map(
                style,
                [
                    "width",
                    "height",
                    "minWidth",
                    "minHeight",
                    "maxWidth",
                    "maxHeight",
                    "paddingTop",
                    "paddingRight",
                    "paddingBottom",
                    "paddingLeft",
                    "marginTop",
                    "marginRight",
                    "marginBottom",
                    "marginLeft",
                    "gap",
                    "rowGap",
                    "columnGap",
                ],
            ),
        },
        "text": {
            **({"text": node.text} if node.text else {}),
            **_style_string_map(
                style,
                ["fontFamily", "fontWeight", "textAlign", "textTransform", "whiteSpace"],
            ),
            **_style_px_map(style, ["fontSize", "lineHeight", "letterSpacing"]),
        },
        "paint": {
            **_style_string_map(
                style,
                [
                    "color",
                    "backgroundColor",
                    "borderTopColor",
                    "borderRightColor",
                    "borderBottomColor",
                    "borderLeftColor",
                    "boxShadow",
                    "opacity",
                ],
            ),
            "border_widths_px": _style_px_map(
                style,
                [
                    "borderTopWidth",
                    "borderRightWidth",
                    "borderBottomWidth",
                    "borderLeftWidth",
                ],
            ),
            "corner_radii_px": _style_px_map(
                style,
                [
                    "borderTopLeftRadius",
                    "borderTopRightRadius",
                    "borderBottomRightRadius",
                    "borderBottomLeftRadius",
                ],
            ),
        },
        "semantics": _dom_semantics_fact(node),
        "interaction": _dom_interaction_fact(node),
        "descendant_summary": {
            "count": len(descendants),
            "svg_count": len(svg_descendants),
            "text_count": len(text_descendants),
        },
        "source": "upstream_dom_computed_style",
    }
    if svg_descendants:
        facts["icon"] = {
            "first_svg_bounds": svg_descendants[0].bounds.to_json(),
            "first_svg_class_tokens": _class_tokens(svg_descendants[0].class_name),
            "svg_count": len(svg_descendants),
        }
    if text_descendants:
        facts["text"]["descendant_text"] = [item.text for item in text_descendants]
    return facts


def _fret_node_fact(node: LayoutNode) -> dict[str, Any]:
    result: dict[str, Any] = {
        "test_id": node.test_id,
        "bounds": node.bounds.to_json(),
        "raw_bounds": node.raw_bounds.to_json(),
        "scale_factor": node.scale_factor,
        "coordinate_units": node.coordinate_units,
        "node": node.node,
        "kind_hint": node.kind,
        "source": node.source,
    }
    if node.source == "layout_sidecar":
        result["sidecar_path"] = node.sidecar_path
    else:
        result["bundle_schema2_path"] = node.sidecar_path
    if node.label:
        label = node.label
        result["label_preview"] = label[:500]
        if "padding:" in label:
            result["has_chrome_padding_hint"] = True
        if "border:" in label or "border_color:" in label:
            result["has_chrome_border_hint"] = True
        if "corner_radii:" in label:
            result["has_corner_radii_hint"] = True
        if "SvgIconProps" in label:
            result["has_icon_hint"] = True
    if node.semantics:
        result["semantics"] = node.semantics
        result["interaction"] = {
            "focusable": bool(node.semantics.get("actions", {}).get("focus")),
            "invokable": bool(node.semantics.get("actions", {}).get("invoke")),
            "set_value": bool(node.semantics.get("actions", {}).get("set_value")),
            "disabled": bool(node.semantics.get("flags", {}).get("disabled")),
            "focused": bool(node.semantics.get("flags", {}).get("focused")),
            "source": "bundle_schema2_semantics_actions",
        }
    return result


COMPACT_EVIDENCE_PATH_LIMIT = 3
COMPACT_NODE_SAMPLE_LIMIT = 3
COMPACT_NODE_FACT_LIMIT = 6
COMPACT_TEXT_PAINT_FACT_LIMIT = 8


def _stable_json_key(value: Any) -> str:
    return json.dumps(
        value,
        sort_keys=True,
        ensure_ascii=False,
        separators=(",", ":"),
    )


def _append_unique_sample(items: list[Any], value: Any, limit: int) -> None:
    if value is None or value in items or len(items) >= limit:
        return
    items.append(value)


TEXT_PAINT_VOLATILE_KEYS = {
    "window",
    "frame_id",
    "window_snapshot_seq",
    "bundle_schema2_path",
    "node",
    "element",
    "measure_time_us",
    "paint_time_us",
    "prepare_time_us",
    "inclusive_time_us",
}


def _compact_text_paint_fact(row: dict[str, Any]) -> dict[str, Any]:
    fact = {
        key: value
        for key, value in row.items()
        if key not in TEXT_PAINT_VOLATILE_KEYS and value not in (None, [], {})
    }
    fact["source"] = "bundle_schema2_text_paint"
    return fact


def _compact_text_paint_facts(
    rows: list[dict[str, Any]], limit: int = COMPACT_TEXT_PAINT_FACT_LIMIT
) -> list[dict[str, Any]]:
    compact_by_key: dict[str, dict[str, Any]] = {}
    ordered_keys: list[str] = []
    for row in rows:
        fact = _compact_text_paint_fact(row)
        key = _stable_json_key(fact)
        existing = compact_by_key.get(key)
        if existing is None:
            existing = dict(fact)
            existing["observed_count"] = 0
            compact_by_key[key] = existing
            ordered_keys.append(key)
        existing["observed_count"] += 1
        _append_unique_sample(
            existing.setdefault("evidence_paths", []),
            row.get("bundle_schema2_path"),
            COMPACT_EVIDENCE_PATH_LIMIT,
        )
        _append_unique_sample(
            existing.setdefault("node_samples", []),
            str(row.get("node")) if row.get("node") is not None else None,
            COMPACT_NODE_SAMPLE_LIMIT,
        )

    compact = [compact_by_key[key] for key in ordered_keys[:limit]]
    omitted = max(0, len(ordered_keys) - limit)
    if omitted and compact:
        compact[-1]["omitted_compact_fact_count"] = omitted
    return compact


def _merge_compact_text_paint_facts(
    existing_rows: list[dict[str, Any]], incoming_rows: list[dict[str, Any]]
) -> list[dict[str, Any]]:
    merged_by_key: dict[str, dict[str, Any]] = {}
    ordered_keys: list[str] = []
    for row in [*existing_rows, *incoming_rows]:
        stable = {
            key: value
            for key, value in row.items()
            if key
            not in {
                "observed_count",
                "evidence_paths",
                "node_samples",
                "omitted_compact_fact_count",
            }
        }
        key = _stable_json_key(stable)
        current = merged_by_key.get(key)
        if current is None:
            current = dict(stable)
            current["observed_count"] = 0
            merged_by_key[key] = current
            ordered_keys.append(key)
        current["observed_count"] += int(row.get("observed_count", 1))
        for path in row.get("evidence_paths", []):
            _append_unique_sample(
                current.setdefault("evidence_paths", []),
                path,
                COMPACT_EVIDENCE_PATH_LIMIT,
            )
        for sample in row.get("node_samples", []):
            _append_unique_sample(
                current.setdefault("node_samples", []),
                sample,
                COMPACT_NODE_SAMPLE_LIMIT,
            )

    return [merged_by_key[key] for key in ordered_keys[:COMPACT_TEXT_PAINT_FACT_LIMIT]]


SEMANTIC_TEXT_VOLATILE_KEYS = {
    "bundle_schema2_path",
    "node",
}


def _compact_semantic_text_fact(row: dict[str, Any]) -> dict[str, Any]:
    return {
        key: value
        for key, value in row.items()
        if key not in SEMANTIC_TEXT_VOLATILE_KEYS and value not in (None, [], {})
    }


def _compact_semantic_text_facts(
    rows: list[dict[str, Any]], limit: int = COMPACT_TEXT_PAINT_FACT_LIMIT
) -> list[dict[str, Any]]:
    compact_by_key: dict[str, dict[str, Any]] = {}
    ordered_keys: list[str] = []
    for row in rows:
        fact = _compact_semantic_text_fact(row)
        key = _stable_json_key(fact)
        existing = compact_by_key.get(key)
        if existing is None:
            existing = dict(fact)
            existing["observed_count"] = 0
            compact_by_key[key] = existing
            ordered_keys.append(key)
        existing["observed_count"] += 1
        _append_unique_sample(
            existing.setdefault("evidence_paths", []),
            row.get("bundle_schema2_path"),
            COMPACT_EVIDENCE_PATH_LIMIT,
        )
        _append_unique_sample(
            existing.setdefault("node_samples", []),
            row.get("node"),
            COMPACT_NODE_SAMPLE_LIMIT,
        )
        _append_unique_sample(
            existing.setdefault("bounds_samples", []),
            row.get("bounds"),
            COMPACT_NODE_SAMPLE_LIMIT,
        )

    compact = [compact_by_key[key] for key in ordered_keys[:limit]]
    omitted = max(0, len(ordered_keys) - limit)
    if omitted and compact:
        compact[-1]["omitted_compact_fact_count"] = omitted
    return compact


def _merge_compact_semantic_text_facts(
    existing_rows: list[dict[str, Any]], incoming_rows: list[dict[str, Any]]
) -> list[dict[str, Any]]:
    merged_by_key: dict[str, dict[str, Any]] = {}
    ordered_keys: list[str] = []
    for row in [*existing_rows, *incoming_rows]:
        stable = {
            key: value
            for key, value in row.items()
            if key
            not in {
                "observed_count",
                "evidence_paths",
                "node_samples",
                "bounds_samples",
                "omitted_compact_fact_count",
            }
        }
        key = _stable_json_key(stable)
        current = merged_by_key.get(key)
        if current is None:
            current = dict(stable)
            current["observed_count"] = 0
            merged_by_key[key] = current
            ordered_keys.append(key)
        current["observed_count"] += int(row.get("observed_count", 1))
        for path in row.get("evidence_paths", []):
            _append_unique_sample(
                current.setdefault("evidence_paths", []),
                path,
                COMPACT_EVIDENCE_PATH_LIMIT,
            )
        for sample in row.get("node_samples", []):
            _append_unique_sample(
                current.setdefault("node_samples", []),
                sample,
                COMPACT_NODE_SAMPLE_LIMIT,
            )
        for sample in row.get("bounds_samples", []):
            _append_unique_sample(
                current.setdefault("bounds_samples", []),
                sample,
                COMPACT_NODE_SAMPLE_LIMIT,
            )

    return [merged_by_key[key] for key in ordered_keys[:COMPACT_TEXT_PAINT_FACT_LIMIT]]


def _rows_for_layout_node(
    rows_by_node: dict[str, list[dict[str, Any]]], node: LayoutNode
) -> list[dict[str, Any]]:
    rows = rows_by_node.get(node.node, [])
    if node.source != "bundle_schema2_semantics":
        return rows
    return [
        row
        for row in rows
        if row.get("bundle_schema2_path") == node.sidecar_path
    ]


def _fret_node_fact_with_text_paint(
    node: LayoutNode, evidence: LayoutEvidence
) -> dict[str, Any]:
    fact = _fret_node_fact(node)
    text_paint_facts = _compact_text_paint_facts(
        _rows_for_layout_node(evidence.text_paint_facts_by_node, node)
    )
    if text_paint_facts:
        fact["text_paint"] = text_paint_facts
    associated_text_paint_facts = _compact_text_paint_facts(
        _rows_for_layout_node(evidence.text_paint_associated_facts_by_node, node)
    )
    if associated_text_paint_facts:
        fact["associated_text_paint"] = associated_text_paint_facts
    text_label_facts = _compact_semantic_text_facts(
        _rows_for_layout_node(evidence.text_label_facts_by_node, node)
    )
    if text_label_facts:
        fact["semantic_text_descendants"] = text_label_facts
    if text_label_facts and not text_paint_facts and not associated_text_paint_facts:
        fact["text_paint_coverage"] = {
            "status": "semantic_text_without_text_paint_hotspot",
            "reason": "bundle schema2 text_paint is hotspot-sparse for this node subtree",
        }
    return fact


def _compact_fret_node_facts(
    facts: list[dict[str, Any]], limit: int = COMPACT_NODE_FACT_LIMIT
) -> list[dict[str, Any]]:
    compact_by_key: dict[str, dict[str, Any]] = {}
    ordered_keys: list[str] = []
    for fact in facts:
        signature = {
            "test_id": fact.get("test_id"),
            "kind_hint": fact.get("kind_hint"),
            "coordinate_units": fact.get("coordinate_units"),
            "scale_factor": fact.get("scale_factor"),
            "source": fact.get("source"),
            "semantics": fact.get("semantics"),
            "interaction": fact.get("interaction"),
        }
        key = _stable_json_key(signature)
        existing = compact_by_key.get(key)
        source_path = fact.get("sidecar_path") or fact.get("bundle_schema2_path")
        if existing is None:
            existing = dict(fact)
            existing.pop("sidecar_path", None)
            existing.pop("bundle_schema2_path", None)
            existing.pop("text_paint", None)
            existing.pop("associated_text_paint", None)
            existing.pop("semantic_text_descendants", None)
            existing.pop("text_paint_coverage", None)
            existing["observed_count"] = 0
            compact_by_key[key] = existing
            ordered_keys.append(key)

        existing["observed_count"] += 1
        _append_unique_sample(
            existing.setdefault("evidence_paths", []),
            source_path,
            COMPACT_EVIDENCE_PATH_LIMIT,
        )
        _append_unique_sample(
            existing.setdefault("node_samples", []),
            fact.get("node"),
            COMPACT_NODE_SAMPLE_LIMIT,
        )
        _append_unique_sample(
            existing.setdefault("bounds_samples", []),
            fact.get("bounds"),
            COMPACT_NODE_SAMPLE_LIMIT,
        )
        if fact.get("text_paint"):
            existing["text_paint"] = _merge_compact_text_paint_facts(
                existing.get("text_paint", []), fact["text_paint"]
            )
        if fact.get("associated_text_paint"):
            existing["associated_text_paint"] = _merge_compact_text_paint_facts(
                existing.get("associated_text_paint", []),
                fact["associated_text_paint"],
            )
        if fact.get("semantic_text_descendants"):
            existing["semantic_text_descendants"] = (
                _merge_compact_semantic_text_facts(
                    existing.get("semantic_text_descendants", []),
                    fact["semantic_text_descendants"],
                )
            )
        if (
            fact.get("text_paint_coverage")
            and not existing.get("text_paint")
            and not existing.get("associated_text_paint")
        ):
            existing["text_paint_coverage"] = fact["text_paint_coverage"]

    compact = [compact_by_key[key] for key in ordered_keys[:limit]]
    omitted = max(0, len(ordered_keys) - limit)
    if omitted and compact:
        compact[-1]["omitted_compact_fact_count"] = omitted
    return compact


def _fret_semantics_facts_for_test_id(
    test_id: str, evidence: LayoutEvidence
) -> list[dict[str, Any]]:
    return _compact_fret_node_facts(
        [
            _fret_node_fact_with_text_paint(node, evidence)
            for node in evidence.bundle_nodes_by_test_id.get(test_id, [])
            if node.semantics
        ]
    )


def _semantics_summary(
    test_id: str, evidence: LayoutEvidence, compact_facts: list[dict[str, Any]]
) -> dict[str, Any]:
    observed_count = sum(
        1 for node in evidence.bundle_nodes_by_test_id.get(test_id, []) if node.semantics
    )
    return {
        "observed_node_count": observed_count,
        "compact_node_count": len(compact_facts),
        "omitted_node_count": max(
            0, observed_count - sum(fact.get("observed_count", 1) for fact in compact_facts)
        ),
    }


def _fact_has_semantics(fact: dict[str, Any]) -> bool:
    if fact.get("semantics"):
        return True
    primary = fact.get("primary")
    if isinstance(primary, dict) and _fact_has_semantics(primary):
        return True
    return any(
        isinstance(item, dict) and _fact_has_semantics(item)
        for key in ("semantics_nodes", "related")
        for item in fact.get(key, [])
    )


def _fact_has_interaction(fact: dict[str, Any]) -> bool:
    if fact.get("interaction"):
        return True
    primary = fact.get("primary")
    if isinstance(primary, dict) and _fact_has_interaction(primary):
        return True
    return any(
        isinstance(item, dict) and _fact_has_interaction(item)
        for key in ("semantics_nodes", "related")
        for item in fact.get(key, [])
    )


def _fact_text_paint_count(fact: dict[str, Any]) -> int:
    count = len(fact.get("text_paint", []))
    count += len(fact.get("associated_text_paint", []))
    primary = fact.get("primary")
    if isinstance(primary, dict):
        count += _fact_text_paint_count(primary)
    for key in ("semantics_nodes", "related"):
        count += sum(
            _fact_text_paint_count(item)
            for item in fact.get(key, [])
            if isinstance(item, dict)
        )
    return count


def _fact_direct_text_paint_count(fact: dict[str, Any]) -> int:
    count = len(fact.get("text_paint", []))
    primary = fact.get("primary")
    if isinstance(primary, dict):
        count += _fact_direct_text_paint_count(primary)
    for key in ("semantics_nodes", "related"):
        count += sum(
            _fact_direct_text_paint_count(item)
            for item in fact.get(key, [])
            if isinstance(item, dict)
        )
    return count


def _fact_associated_text_paint_count(fact: dict[str, Any]) -> int:
    count = len(fact.get("associated_text_paint", []))
    primary = fact.get("primary")
    if isinstance(primary, dict):
        count += _fact_associated_text_paint_count(primary)
    for key in ("semantics_nodes", "related"):
        count += sum(
            _fact_associated_text_paint_count(item)
            for item in fact.get(key, [])
            if isinstance(item, dict)
        )
    return count


def _fact_semantic_text_count(fact: dict[str, Any]) -> int:
    count = len(fact.get("semantic_text_descendants", []))
    primary = fact.get("primary")
    if isinstance(primary, dict):
        count += _fact_semantic_text_count(primary)
    for key in ("semantics_nodes", "related"):
        count += sum(
            _fact_semantic_text_count(item)
            for item in fact.get(key, [])
            if isinstance(item, dict)
        )
    return count


def _related_test_ids(test_id: str, evidence: LayoutEvidence) -> list[str]:
    candidates = [
        f"{test_id}.chrome",
        f"{test_id}-icon",
        f"{test_id}-content",
        f"{test_id}-label",
    ]
    return [
        candidate
        for candidate in candidates
        if candidate in evidence.nodes_by_test_id
        or candidate in evidence.bundle_nodes_by_test_id
    ]


def _fret_test_id_facts(test_id: str, evidence: LayoutEvidence) -> dict[str, Any]:
    primary = evidence.find(test_id)
    facts: dict[str, Any] = {"test_id": test_id}
    if primary is not None:
        facts["primary"] = _fret_node_fact_with_text_paint(primary, evidence)
    semantics_nodes = _fret_semantics_facts_for_test_id(test_id, evidence)
    if semantics_nodes:
        facts["semantics_nodes"] = semantics_nodes
        facts["semantics_summary"] = _semantics_summary(
            test_id, evidence, semantics_nodes
        )
    related = []
    for related_test_id in _related_test_ids(test_id, evidence):
        related_node = evidence.find(related_test_id)
        if related_node is not None:
            related.append(_fret_node_fact_with_text_paint(related_node, evidence))
        related_semantics_nodes = _fret_semantics_facts_for_test_id(
            related_test_id, evidence
        )
        if related_semantics_nodes:
            related.append(
                {
                    "test_id": related_test_id,
                    "semantics_nodes": related_semantics_nodes,
                    "semantics_summary": _semantics_summary(
                        related_test_id, evidence, related_semantics_nodes
                    ),
                    "source": "bundle_schema2_semantics",
                }
            )
    if related:
        facts["related"] = related
    if primary is None and not related and not semantics_nodes:
        facts["status"] = "missing"
    return facts


def _part_live_facts(
    source_part: dict[str, Any],
    report_part: dict[str, Any],
    layout_evidence: LayoutEvidence,
    dom_evidence: DomEvidence,
) -> dict[str, Any]:
    upstream = source_part.get("upstream", {}) if isinstance(source_part, dict) else {}
    dom_target_ids = (
        upstream.get("dom_target_ids") if isinstance(upstream.get("dom_target_ids"), list) else []
    )
    upstream_facts = []
    for target_id in dom_target_ids:
        if not isinstance(target_id, str):
            continue
        node = dom_evidence.find(target_id)
        if node is not None:
            upstream_facts.append(_dom_target_facts(node, dom_evidence))

    fret_facts = [
        _fret_test_id_facts(test_id, layout_evidence)
        for test_id in report_part.get("test_ids", [])
        if isinstance(test_id, str)
    ]
    return {
        "part_id": report_part["id"],
        "upstream_dom_target_count": len(upstream_facts),
        "fret_test_id_count": len(fret_facts),
        "upstream_semantics_fact_count": sum(
            1 for fact in upstream_facts if fact.get("semantics")
        ),
        "upstream_interaction_fact_count": sum(
            1 for fact in upstream_facts if fact.get("interaction")
        ),
        "fret_semantics_fact_count": sum(
            1 for fact in fret_facts if _fact_has_semantics(fact)
        ),
        "fret_interaction_fact_count": sum(
            1 for fact in fret_facts if _fact_has_interaction(fact)
        ),
        "fret_text_paint_fact_count": sum(
            _fact_text_paint_count(fact) for fact in fret_facts
        ),
        "fret_text_paint_direct_fact_count": sum(
            _fact_direct_text_paint_count(fact) for fact in fret_facts
        ),
        "fret_text_paint_associated_fact_count": sum(
            _fact_associated_text_paint_count(fact) for fact in fret_facts
        ),
        "fret_text_label_fact_count": sum(
            _fact_semantic_text_count(fact) for fact in fret_facts
        ),
        "fret_text_paint_bundle_entry_count": layout_evidence.text_paint_bundle_entry_count,
        "fret_text_paint_row_count": layout_evidence.text_paint_fact_row_count,
        "fret_text_paint_association_row_count": (
            layout_evidence.text_paint_association_row_count
        ),
        "fret_text_paint_unassociated_row_count": (
            layout_evidence.text_paint_unassociated_row_count
        ),
        "fret_text_label_row_count": layout_evidence.text_label_fact_row_count,
        "upstream": upstream_facts,
        "fret": fret_facts,
    }


def _generate_live_facts(
    mapping: dict[str, Any],
    report_parts: list[dict[str, Any]],
    layout_evidence: LayoutEvidence,
    dom_evidence: DomEvidence,
) -> dict[str, Any]:
    source_parts_by_id = {
        part["id"]: part
        for part in mapping.get("parts", [])
        if isinstance(part, dict) and isinstance(part.get("id"), str)
    }
    parts = [
        _part_live_facts(
            source_parts_by_id.get(report_part["id"], {}),
            report_part,
            layout_evidence,
            dom_evidence,
        )
        for report_part in report_parts
    ]
    return {
        "schema_version": 1,
        "upstream_source": "upstream_dom_computed_style",
        "fret_source": "layout_sidecar+bundle_schema2_semantics",
        "part_count": len(parts),
        "upstream_dom_target_count": sum(
            part["upstream_dom_target_count"] for part in parts
        ),
        "fret_test_id_count": sum(part["fret_test_id_count"] for part in parts),
        "upstream_semantics_fact_count": sum(
            part["upstream_semantics_fact_count"] for part in parts
        ),
        "upstream_interaction_fact_count": sum(
            part["upstream_interaction_fact_count"] for part in parts
        ),
        "fret_semantics_fact_count": sum(
            part["fret_semantics_fact_count"] for part in parts
        ),
        "fret_interaction_fact_count": sum(
            part["fret_interaction_fact_count"] for part in parts
        ),
        "fret_text_paint_fact_count": sum(
            part["fret_text_paint_fact_count"] for part in parts
        ),
        "fret_text_paint_direct_fact_count": sum(
            part["fret_text_paint_direct_fact_count"] for part in parts
        ),
        "fret_text_paint_associated_fact_count": sum(
            part["fret_text_paint_associated_fact_count"] for part in parts
        ),
        "fret_text_label_fact_count": sum(
            part["fret_text_label_fact_count"] for part in parts
        ),
        "fret_text_paint_bundle_entry_count": layout_evidence.text_paint_bundle_entry_count,
        "fret_text_paint_row_count": layout_evidence.text_paint_fact_row_count,
        "fret_text_paint_association_row_count": (
            layout_evidence.text_paint_association_row_count
        ),
        "fret_text_paint_unassociated_row_count": (
            layout_evidence.text_paint_unassociated_row_count
        ),
        "fret_text_label_row_count": layout_evidence.text_label_fact_row_count,
        "parts": parts,
    }


def check_status(check: dict[str, Any], measurement: dict[str, Any] | None) -> str:
    if measurement is not None:
        observed = measurement["status"]
        if observed == "missing":
            return "blocked"
        if observed == "fail":
            return "mismatch"
        if observed == "pass":
            return "pass_known"

    kind = check["kind"]
    observed = check["observed"]

    if kind == "blocked" or observed == "missing":
        return "blocked"
    if kind == "expected_mismatch" or observed == "fail":
        return "mismatch"
    if kind == "live_measurement_required":
        return "needs_live_measurement"
    if kind == "existing_gate" and observed == "pass":
        return "pass_known"
    return "needs_live_measurement"


def merge_part_status(check_statuses: list[str]) -> str:
    if "blocked" in check_statuses:
        return "blocked"
    if "mismatch" in check_statuses:
        return "mismatch"
    if "needs_live_measurement" in check_statuses:
        return "needs_live_measurement"
    return "pass_known"


def merge_confidence(checks: list[dict[str, Any]]) -> str:
    values = [check["confidence"] for check in checks]
    if "low" in values:
        return "low"
    if "medium" in values:
        return "medium"
    return "high"


def _triage_level(score: int) -> str:
    if score >= 90:
        return "critical"
    if score >= 70:
        return "high"
    if score >= 40:
        return "medium"
    if score > 0:
        return "low"
    return "none"


def _expected_gap_px(result: dict[str, Any]) -> float:
    observed = result.get("observed_px")
    expected = result.get("expected")
    comparison = result.get("comparison")
    if not isinstance(observed, int | float) or not isinstance(expected, str):
        return 0.0
    numbers = [float(item) for item in re.findall(r"-?\d+(?:\.\d+)?", expected)]
    if not numbers:
        return 0.0
    observed = float(observed)
    if comparison == "eq":
        return abs(observed - numbers[0])
    if comparison == "gte":
        return max(0.0, numbers[0] - observed)
    if comparison == "lte":
        return max(0.0, observed - numbers[0])
    if comparison == "between" and len(numbers) >= 2:
        return max(numbers[0] - observed, observed - numbers[1], 0.0)
    return 0.0


def _measurement_gap_px(value: Any) -> float:
    if isinstance(value, dict):
        gaps: list[float] = []
        logical_delta = value.get("logical_delta_px")
        if isinstance(logical_delta, int | float):
            gaps.append(abs(float(logical_delta)))
        gaps.append(_expected_gap_px(value))
        for child in value.values():
            gaps.append(_measurement_gap_px(child))
        return max(gaps, default=0.0)
    if isinstance(value, list):
        return max((_measurement_gap_px(item) for item in value), default=0.0)
    return 0.0


def _gap_score(gap_px: float) -> int:
    if gap_px >= 64.0:
        return 10
    if gap_px >= 16.0:
        return 6
    if gap_px >= 4.0:
        return 3
    return 0


def triage_check(
    check: dict[str, Any],
    part: dict[str, Any],
    status: str,
    owner: str,
    layer: str,
    measurement: dict[str, Any] | None,
) -> dict[str, Any]:
    if status == "pass_known":
        return {
            "score": 0,
            "level": "none",
            "reasons": ["passing evidence"],
        }

    axis = part["axis"]
    promotion_target = check["promotion"]["target"]
    score = STATUS_TRIAGE_SCORE[status]
    reasons = [f"status:{status}"]

    layer_score = LAYER_TRIAGE_SCORE.get(layer, 0)
    if layer_score:
        score += layer_score
        reasons.append(f"layer:{layer}")

    promotion_score = PROMOTION_TRIAGE_SCORE.get(promotion_target, 0)
    if promotion_score:
        score += promotion_score
        reasons.append(f"promotion:{promotion_target}")

    axis_score = AXIS_TRIAGE_SCORE.get(axis, 0)
    if axis_score:
        score += axis_score
        reasons.append(f"axis:{axis}")

    confidence_score = CONFIDENCE_TRIAGE_SCORE[check["confidence"]]
    if confidence_score:
        score += confidence_score
        reasons.append(f"confidence:{check['confidence']}")

    gap_px = _measurement_gap_px(measurement) if measurement is not None else 0.0
    gap_score = _gap_score(gap_px)
    if gap_score:
        score += gap_score
        reasons.append(f"measurement_gap_px:{gap_px:.3g}")

    if status == "blocked":
        reasons.append("required evidence is missing")
    elif status == "needs_live_measurement":
        reasons.append("source fact is not yet live-measured")

    score = min(score, 100)
    return {
        "score": score,
        "level": _triage_level(score),
        "reasons": reasons,
    }


def merge_triage(checks: list[dict[str, Any]]) -> dict[str, Any]:
    if not checks:
        return {"score": 0, "level": "none", "reasons": ["no checks"]}
    highest = max(checks, key=lambda check: check["triage"]["score"])
    return {
        "score": highest["triage"]["score"],
        "level": highest["triage"]["level"],
        "highest_check_id": highest["id"],
        "reasons": highest["triage"]["reasons"],
    }


def _top_findings(report_parts: list[dict[str, Any]]) -> list[dict[str, Any]]:
    findings: list[dict[str, Any]] = []
    for part in report_parts:
        for check in part["checks"]:
            if check["status"] == "pass_known":
                continue
            findings.append(
                {
                    "part_id": part["id"],
                    "check_id": check["id"],
                    "status": check["status"],
                    "axis": part["axis"],
                    "owner": check["owner"],
                    "layer": check["layer"],
                    "promotion_target": check["promotion"]["target"],
                    "triage": check["triage"],
                }
            )
    findings.sort(
        key=lambda item: (
            -int(item["triage"]["score"]),
            item["part_id"],
            item["check_id"],
        )
    )
    return findings[:10]


def _source_ref_index(mapping: dict[str, Any]) -> dict[str, dict[str, str]]:
    refs: dict[str, dict[str, str]] = {}
    source_refs = mapping.get("source_refs", {})
    if not isinstance(source_refs, dict):
        return refs
    for bucket in ("upstream", "fret"):
        raw_refs = source_refs.get(bucket, [])
        if not isinstance(raw_refs, list):
            continue
        for ref in raw_refs:
            if not isinstance(ref, dict):
                continue
            ref_id = ref.get("id")
            path = ref.get("path")
            if not isinstance(ref_id, str) or not isinstance(path, str):
                continue
            refs[ref_id] = {
                "id": ref_id,
                "bucket": bucket,
                "path": path.replace("\\", "/"),
            }
    return refs


def _resolve_source_refs(
    source_ref_index: dict[str, dict[str, str]], ids: list[str]
) -> list[dict[str, str]]:
    resolved = []
    for ref_id in ids:
        ref = source_ref_index.get(ref_id)
        if ref is None:
            resolved.append({"id": ref_id, "bucket": "unknown", "path": ""})
        else:
            resolved.append(ref)
    return resolved


def _part_source_ref_ids(part: dict[str, Any], bucket: str) -> list[str]:
    source = part.get(bucket, {})
    if not isinstance(source, dict):
        return []
    source_ref_ids = source.get("source_ref_ids", [])
    if not isinstance(source_ref_ids, list):
        return []
    return [item for item in source_ref_ids if isinstance(item, str)]


def _agent_next_step(check: dict[str, Any]) -> str:
    status = check["status"]
    target = check["promotion"]["target"]
    owner = check["owner"]
    layer = check["layer"]
    if status == "mismatch":
        return (
            f"Repair the {layer}/{owner} owner until the measured predicate passes, "
            f"then promote or refresh the {target} gate."
        )
    if status == "blocked":
        return (
            "Capture or wire the missing source/Fret evidence first; do not edit recipes "
            "until the missing fact is measurable."
        )
    if status == "needs_live_measurement":
        return (
            "Replace the curated or fixture-only fact with live upstream/Fret measurement, "
            f"then decide whether it belongs in a {target} gate."
        )
    return (
        f"Keep this as a regression lock for the {layer}/{owner} owner; harden only if "
        "confidence or coverage gaps remain."
    )


def _agent_queue_item(
    source_part: dict[str, Any],
    report_part: dict[str, Any],
    check: dict[str, Any],
    source_ref_index: dict[str, dict[str, str]],
) -> dict[str, Any]:
    upstream_ref_ids = _part_source_ref_ids(source_part, "upstream")
    fret_ref_ids = _part_source_ref_ids(source_part, "fret")
    return {
        "part_id": report_part["id"],
        "part_label": report_part["label"],
        "check_id": check["id"],
        "status": check["status"],
        "axis": report_part["axis"],
        "owner": check["owner"],
        "layer": check["layer"],
        "promotion_target": check["promotion"]["target"],
        "confidence": check["confidence"],
        "triage": check["triage"],
        "expected": check["expected"],
        "observed": check["observed"],
        "observed_source": check["observed_source"],
        "test_ids": report_part["test_ids"],
        "source_refs": {
            "upstream": _resolve_source_refs(source_ref_index, upstream_ref_ids),
            "fret": _resolve_source_refs(source_ref_index, fret_ref_ids),
        },
        "evidence_refs": check["evidence_refs"],
        "next_step": _agent_next_step(check),
    }


def _generate_agent_packet(
    mapping: dict[str, Any],
    mapping_path: Path,
    report: dict[str, Any],
) -> dict[str, Any]:
    source_ref_index = _source_ref_index(mapping)
    source_parts_by_id = {
        part["id"]: part
        for part in mapping.get("parts", [])
        if isinstance(part, dict) and isinstance(part.get("id"), str)
    }
    repair_queue: list[dict[str, Any]] = []
    hardening_queue: list[dict[str, Any]] = []
    gate_queue: list[dict[str, Any]] = []

    for report_part in report["parts"]:
        source_part = source_parts_by_id.get(report_part["id"], {})
        for check in report_part["checks"]:
            item = _agent_queue_item(
                source_part, report_part, check, source_ref_index
            )
            if check["status"] != "pass_known":
                repair_queue.append(item)
            elif check["confidence"] != "high" or report_part["confidence"] != "high":
                hardening_queue.append(item)

            target = check["promotion"]["target"]
            if target != "none":
                gate_queue.append(
                    {
                        "part_id": report_part["id"],
                        "check_id": check["id"],
                        "target": target,
                        "reason": check["promotion"]["reason"],
                        "status": check["status"],
                        "owner": check["owner"],
                        "layer": check["layer"],
                    }
                )

    if repair_queue:
        status = "needs_repair"
    elif hardening_queue:
        status = "needs_hardening"
    else:
        status = "regression_locked"

    upstream_refs = [
        ref for ref in source_ref_index.values() if ref["bucket"] == "upstream"
    ]
    fret_refs = [ref for ref in source_ref_index.values() if ref["bucket"] == "fret"]

    return {
        "schema_version": 1,
        "status": status,
        "component": report["component"],
        "style": report["style"],
        "source_mapping": str(mapping_path).replace("\\", "/"),
        "goal": (
            "Give a repair agent the minimum source truth, Fret wiring, evidence, "
            "and promotion queues needed to turn parity findings into gates."
        ),
        "truth": {
            "upstream_refs": upstream_refs,
            "upstream_contexts": report["upstream_contexts"],
            "live_facts": report.get("live_facts", {}),
            "limitations": report["limitations"],
        },
        "fret_wiring": {
            "refs": fret_refs,
            "test_ids": sorted(
                {
                    test_id
                    for part in report["parts"]
                    for test_id in part.get("test_ids", [])
                }
            ),
        },
        "evidence": report["evidence_contexts"],
        "summary": {
            "status_counts": report["summary"]["status_counts"],
            "owner_status_counts": report["summary"]["owner_status_counts"],
            "layer_status_counts": report["summary"]["layer_status_counts"],
            "top_findings": report["summary"]["top_findings"],
            "upstream_live_fact_count": report["summary"].get(
                "upstream_live_fact_count", 0
            ),
            "fret_live_fact_count": report["summary"].get("fret_live_fact_count", 0),
            "upstream_semantics_fact_count": report["summary"].get(
                "upstream_semantics_fact_count", 0
            ),
            "upstream_interaction_fact_count": report["summary"].get(
                "upstream_interaction_fact_count", 0
            ),
            "fret_semantics_fact_count": report["summary"].get(
                "fret_semantics_fact_count", 0
            ),
            "fret_interaction_fact_count": report["summary"].get(
                "fret_interaction_fact_count", 0
            ),
            "fret_text_paint_fact_count": report["summary"].get(
                "fret_text_paint_fact_count", 0
            ),
            "fret_text_paint_direct_fact_count": report["summary"].get(
                "fret_text_paint_direct_fact_count", 0
            ),
            "fret_text_paint_associated_fact_count": report["summary"].get(
                "fret_text_paint_associated_fact_count", 0
            ),
            "fret_text_label_fact_count": report["summary"].get(
                "fret_text_label_fact_count", 0
            ),
            "fret_text_paint_bundle_entry_count": report["summary"].get(
                "fret_text_paint_bundle_entry_count", 0
            ),
            "fret_text_paint_row_count": report["summary"].get(
                "fret_text_paint_row_count", 0
            ),
            "fret_text_paint_association_row_count": report["summary"].get(
                "fret_text_paint_association_row_count", 0
            ),
            "fret_text_paint_unassociated_row_count": report["summary"].get(
                "fret_text_paint_unassociated_row_count", 0
            ),
            "fret_text_label_row_count": report["summary"].get(
                "fret_text_label_row_count", 0
            ),
            "repair_queue_count": len(repair_queue),
            "hardening_queue_count": len(hardening_queue),
            "gate_queue_count": len(gate_queue),
        },
        "repair_queue": repair_queue,
        "hardening_queue": hardening_queue,
        "gate_queue": gate_queue,
    }


def _triage_level_counts(report_parts: list[dict[str, Any]]) -> dict[str, int]:
    counts = {level: 0 for level in TRIAGE_LEVEL_ORDER}
    for part in report_parts:
        level = part["triage"]["level"]
        counts[level] = counts.get(level, 0) + 1
    return counts


def generate_report(
    mapping: dict[str, Any],
    mapping_path: Path,
    layout_evidence: LayoutEvidence,
    dom_evidence: DomEvidence,
) -> dict[str, Any]:
    status_counts = {status: 0 for status in STATUS_ORDER}
    owner_counts: dict[str, int] = {owner: 0 for owner in OWNER_ORDER}
    owner_status_counts: dict[str, dict[str, int]] = {
        owner: {status: 0 for status in STATUS_ORDER} for owner in OWNER_ORDER
    }
    layer_counts: dict[str, int] = {layer: 0 for layer in LAYER_ORDER}
    layer_status_counts: dict[str, dict[str, int]] = {
        layer: {status: 0 for status in STATUS_ORDER} for layer in LAYER_ORDER
    }
    promotion_counts: dict[str, int] = {
        "diag_script": 0,
        "component_fixture": 0,
        "mechanism_harness": 0,
        "none": 0,
    }
    report_parts: list[dict[str, Any]] = []

    for part in mapping["parts"]:
        checks = []
        check_statuses = []
        part_live_facts = _part_live_facts(
            part,
            {"id": part["id"], "test_ids": part["fret"]["test_ids"]},
            layout_evidence,
            dom_evidence,
        )
        for check in part["checks"]:
            measurement = combine_measurements(
                evaluate_fret_measurement(check, layout_evidence),
                evaluate_upstream_dom_measurement(check, dom_evidence),
            )
            live_measurement = evaluate_live_fact_measurement(check, part_live_facts)
            if live_measurement is not None:
                measurement = live_measurement if measurement is None else {
                    "source": f"{measurement['source']}+{live_measurement['source']}",
                    "status": "pass"
                    if measurement["status"] == "pass"
                    and live_measurement["status"] == "pass"
                    else live_measurement["status"],
                    "predicate_measurement": measurement,
                    "live_fact_measurement": live_measurement,
                }
            status = check_status(check, measurement)
            owner = _resolve_owner(check)
            layer = _resolve_layer(check, owner)
            triage = triage_check(check, part, status, owner, layer, measurement)
            check_statuses.append(status)
            target = check["promotion"]["target"]
            promotion_counts[target] += 1
            owner_counts[owner] = owner_counts.get(owner, 0) + 1
            owner_status_counts.setdefault(
                owner, {status_name: 0 for status_name in STATUS_ORDER}
            )[status] += 1
            layer_counts[layer] = layer_counts.get(layer, 0) + 1
            layer_status_counts.setdefault(
                layer, {status_name: 0 for status_name in STATUS_ORDER}
            )[status] += 1
            report_check = {
                "id": check["id"],
                "kind": check["kind"],
                "status": status,
                "expected": check["expected"],
                "observed": measurement["status"] if measurement else check["observed"],
                "observed_source": measurement["source"] if measurement else "fixture",
                "confidence": check["confidence"],
                "owner": owner,
                "layer": layer,
                "triage": triage,
                "evidence_refs": check["evidence_refs"],
                "promotion": check["promotion"],
            }
            if measurement is not None:
                report_check["measurement"] = measurement
            checks.append(report_check)

        part_status = merge_part_status(check_statuses)
        status_counts[part_status] += 1
        report_part = {
            "id": part["id"],
            "label": part["label"],
            "axis": part["axis"],
            "status": part_status,
            "confidence": merge_confidence(part["checks"]),
            "triage": merge_triage(checks),
            "test_ids": part["fret"]["test_ids"],
            "upstream_facts": part["upstream"]["facts"],
            "fret_facts": part["fret"]["facts"],
            "checks": checks,
        }
        report_parts.append(report_part)

    top_findings = _top_findings(report_parts)
    live_facts = _generate_live_facts(
        mapping, report_parts, layout_evidence, dom_evidence
    )

    report = {
        "schema_version": SUPPORTED_SCHEMA_VERSION,
        "component": mapping["component"],
        "style": mapping["style"],
        "generated_date": mapping["report"]["generated_date"],
        "generated_by": "tools/parity-discovery/shadcn_parity_discovery.py",
        "source_mapping": str(mapping_path).replace("\\", "/"),
        "upstream_contexts": mapping.get("upstream_contexts", []),
        "evidence_contexts": {
            "fret_layout_sidecar": layout_evidence.sidecar_paths,
            "fret_bundle_schema2": layout_evidence.bundle_paths,
            "upstream_dom": dom_evidence.contexts,
        },
        "summary": {
            "part_count": len(report_parts),
            "status_counts": status_counts,
            "owner_counts": owner_counts,
            "owner_status_counts": owner_status_counts,
            "layer_counts": layer_counts,
            "layer_status_counts": layer_status_counts,
            "triage_level_counts": _triage_level_counts(report_parts),
            "top_findings": top_findings,
            "promotion_target_counts": promotion_counts,
            "layout_sidecar_count": len(layout_evidence.sidecar_paths),
            "bundle_schema2_count": len(layout_evidence.bundle_paths),
            "layout_test_id_count": len(layout_evidence.nodes_by_test_id),
            "bundle_semantics_test_id_count": len(layout_evidence.bundle_nodes_by_test_id),
            "measured_test_id_count": layout_evidence.test_id_count(),
            "upstream_dom_snapshot_count": len(dom_evidence.snapshot_paths),
            "upstream_dom_target_count": len(dom_evidence.nodes_by_target_id),
            "upstream_context_count": len(mapping.get("upstream_contexts", [])),
            "upstream_dom_context_count": len(dom_evidence.contexts),
            "upstream_live_fact_count": live_facts["upstream_dom_target_count"],
            "fret_live_fact_count": live_facts["fret_test_id_count"],
            "upstream_semantics_fact_count": live_facts[
                "upstream_semantics_fact_count"
            ],
            "upstream_interaction_fact_count": live_facts[
                "upstream_interaction_fact_count"
            ],
            "fret_semantics_fact_count": live_facts["fret_semantics_fact_count"],
            "fret_interaction_fact_count": live_facts[
                "fret_interaction_fact_count"
            ],
            "fret_text_paint_fact_count": live_facts["fret_text_paint_fact_count"],
            "fret_text_paint_direct_fact_count": live_facts[
                "fret_text_paint_direct_fact_count"
            ],
            "fret_text_paint_associated_fact_count": live_facts[
                "fret_text_paint_associated_fact_count"
            ],
            "fret_text_label_fact_count": live_facts["fret_text_label_fact_count"],
            "fret_text_paint_bundle_entry_count": live_facts[
                "fret_text_paint_bundle_entry_count"
            ],
            "fret_text_paint_row_count": live_facts["fret_text_paint_row_count"],
            "fret_text_paint_association_row_count": live_facts[
                "fret_text_paint_association_row_count"
            ],
            "fret_text_paint_unassociated_row_count": live_facts[
                "fret_text_paint_unassociated_row_count"
            ],
            "fret_text_label_row_count": live_facts["fret_text_label_row_count"],
        },
        "live_facts": live_facts,
        "parts": report_parts,
        "limitations": mapping["report"]["limitations"],
    }
    report["agent_packet"] = _generate_agent_packet(mapping, mapping_path, report)
    return report


def _require_path_list(value: Any, path: str) -> list[Path]:
    if value is None:
        return []
    return [Path(item) for item in _require_str_list(value, path)]


def load_suite(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise FixtureError(f"{path}: invalid suite JSON: {exc}") from exc

    suite = _require_object(data, "$")
    if suite.get("schema_version") != SUPPORTED_SCHEMA_VERSION:
        raise FixtureError(
            f"$.schema_version must be {SUPPORTED_SCHEMA_VERSION}, got {suite.get('schema_version')!r}"
        )
    _require_str(suite.get("id"), "$.id")
    reports = [
        _require_object(item, f"$.reports[{index}]")
        for index, item in enumerate(_require_list(suite.get("reports"), "$.reports"))
    ]
    _require_unique_ids(reports, "$.reports")
    for index, report in enumerate(reports):
        report_path = f"$.reports[{index}]"
        _require_str(report.get("mapping"), f"{report_path}.mapping")
        _require_str(report.get("output"), f"{report_path}.output")
        _require_path_list(
            report.get("fret_layout_sidecars"), f"{report_path}.fret_layout_sidecars"
        )
        _require_path_list(
            report.get("fret_layout_sidecar_dirs"),
            f"{report_path}.fret_layout_sidecar_dirs",
        )
        _require_path_list(
            report.get("fret_bundle_schema2"), f"{report_path}.fret_bundle_schema2"
        )
        _require_path_list(
            report.get("fret_bundle_schema2_dirs"),
            f"{report_path}.fret_bundle_schema2_dirs",
        )
        _require_path_list(
            report.get("upstream_dom_snapshots"),
            f"{report_path}.upstream_dom_snapshots",
        )
        _require_path_list(
            report.get("upstream_dom_snapshot_dirs"),
            f"{report_path}.upstream_dom_snapshot_dirs",
        )
    return suite


def _resolve_existing_paths(paths: list[Path], kind: str) -> list[Path]:
    unique: dict[str, Path] = {}
    for path in paths:
        if not path.exists():
            raise FixtureError(f"{kind} does not exist: {path}")
        unique[str(path.resolve())] = path
    return [unique[key] for key in sorted(unique)]


def collect_sidecar_paths(paths: list[Path], dirs: list[Path]) -> list[Path]:
    all_paths = list(paths)
    for directory in dirs:
        if not directory.exists():
            raise FixtureError(f"layout sidecar directory does not exist: {directory}")
        all_paths.extend(directory.rglob("layout.taffy.v1.json"))
    return _resolve_existing_paths(all_paths, "layout sidecar")


def collect_bundle_schema2_paths(
    paths: list[Path], dirs: list[Path], sidecar_paths: list[Path]
) -> list[Path]:
    all_paths = list(paths)
    for directory in dirs:
        if not directory.exists():
            raise FixtureError(f"bundle schema2 directory does not exist: {directory}")
        all_paths.extend(directory.rglob("bundle.schema2.json"))
    for sidecar_path in sidecar_paths:
        candidate = sidecar_path.parent / "bundle.schema2.json"
        if candidate.exists():
            all_paths.append(candidate)
    return _resolve_existing_paths(all_paths, "bundle schema2")


def collect_dom_snapshot_paths(paths: list[Path], dirs: list[Path]) -> list[Path]:
    all_paths = list(paths)
    for directory in dirs:
        if not directory.exists():
            raise FixtureError(f"upstream DOM snapshot directory does not exist: {directory}")
        all_paths.extend(directory.rglob("*.json"))
    return _resolve_existing_paths(all_paths, "upstream DOM snapshot")


def generate_report_from_spec(report_spec: dict[str, Any]) -> dict[str, Any]:
    mapping_path = Path(report_spec["mapping"])
    output_path = Path(report_spec["output"])
    mapping = load_mapping(mapping_path)
    layout_sidecar_paths = collect_sidecar_paths(
        _require_path_list(
            report_spec.get("fret_layout_sidecars"),
            "$.reports[].fret_layout_sidecars",
        ),
        _require_path_list(
            report_spec.get("fret_layout_sidecar_dirs"),
            "$.reports[].fret_layout_sidecar_dirs",
        ),
    )
    bundle_schema2_paths = collect_bundle_schema2_paths(
        _require_path_list(
            report_spec.get("fret_bundle_schema2"),
            "$.reports[].fret_bundle_schema2",
        ),
        _require_path_list(
            report_spec.get("fret_bundle_schema2_dirs"),
            "$.reports[].fret_bundle_schema2_dirs",
        ),
        layout_sidecar_paths,
    )
    layout_evidence = load_layout_evidence(
        layout_sidecar_paths,
        bundle_schema2_paths,
    )
    dom_evidence = load_dom_evidence(
        collect_dom_snapshot_paths(
            _require_path_list(
                report_spec.get("upstream_dom_snapshots"),
                "$.reports[].upstream_dom_snapshots",
            ),
            _require_path_list(
                report_spec.get("upstream_dom_snapshot_dirs"),
                "$.reports[].upstream_dom_snapshot_dirs",
            ),
        ),
        mapping.get("upstream_dom_targets", []),
        mapping.get("upstream_contexts", []),
    )
    report = generate_report(mapping, mapping_path, layout_evidence, dom_evidence)
    write_report(report, output_path)
    return report


def load_generated_report(path: Path) -> dict[str, Any]:
    try:
        report = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise FixtureError(f"{path}: invalid generated report JSON: {exc}") from exc
    report = _require_object(report, str(path))
    if report.get("schema_version") != SUPPORTED_SCHEMA_VERSION:
        raise FixtureError(
            f"{path}: report schema_version must be {SUPPORTED_SCHEMA_VERSION}, "
            f"got {report.get('schema_version')!r}"
        )
    _require_str(report.get("component"), f"{path}.component")
    _require_object(report.get("summary"), f"{path}.summary")
    _require_list(report.get("parts"), f"{path}.parts")
    if "agent_packet" not in report:
        report["agent_packet"] = _legacy_agent_packet(report, path)
    return report


def _legacy_agent_packet(report: dict[str, Any], path: Path) -> dict[str, Any]:
    top_findings = report["summary"].get("top_findings", [])
    status_counts = report["summary"].get("status_counts", {})
    non_passing = sum(
        int(status_counts.get(status, 0))
        for status in ("needs_live_measurement", "mismatch", "blocked")
    )
    return {
        "schema_version": 1,
        "status": "needs_repair" if non_passing else "regression_locked",
        "component": report["component"],
        "style": report.get("style"),
        "source_mapping": report.get("source_mapping"),
        "goal": "Legacy packet synthesized from a generated report without embedded agent_packet.",
        "truth": {
            "upstream_refs": [],
            "upstream_contexts": report.get("upstream_contexts", []),
            "live_facts": report.get("live_facts", {}),
            "limitations": report.get("limitations", []),
        },
        "fret_wiring": {
            "refs": [],
            "test_ids": sorted(
                {
                    test_id
                    for part in report.get("parts", [])
                    for test_id in part.get("test_ids", [])
                    if isinstance(test_id, str)
                }
            ),
        },
        "evidence": report.get("evidence_contexts", {}),
        "summary": {
            "status_counts": status_counts,
            "owner_status_counts": report["summary"].get("owner_status_counts", {}),
            "layer_status_counts": report["summary"].get("layer_status_counts", {}),
            "top_findings": top_findings,
            "upstream_live_fact_count": report["summary"].get(
                "upstream_live_fact_count", 0
            ),
            "fret_live_fact_count": report["summary"].get("fret_live_fact_count", 0),
            "upstream_semantics_fact_count": report["summary"].get(
                "upstream_semantics_fact_count", 0
            ),
            "upstream_interaction_fact_count": report["summary"].get(
                "upstream_interaction_fact_count", 0
            ),
            "fret_semantics_fact_count": report["summary"].get(
                "fret_semantics_fact_count", 0
            ),
            "fret_interaction_fact_count": report["summary"].get(
                "fret_interaction_fact_count", 0
            ),
            "fret_text_paint_fact_count": report["summary"].get(
                "fret_text_paint_fact_count", 0
            ),
            "fret_text_paint_direct_fact_count": report["summary"].get(
                "fret_text_paint_direct_fact_count", 0
            ),
            "fret_text_paint_associated_fact_count": report["summary"].get(
                "fret_text_paint_associated_fact_count", 0
            ),
            "fret_text_label_fact_count": report["summary"].get(
                "fret_text_label_fact_count", 0
            ),
            "fret_text_paint_bundle_entry_count": report["summary"].get(
                "fret_text_paint_bundle_entry_count", 0
            ),
            "fret_text_paint_row_count": report["summary"].get(
                "fret_text_paint_row_count", 0
            ),
            "fret_text_paint_association_row_count": report["summary"].get(
                "fret_text_paint_association_row_count", 0
            ),
            "fret_text_paint_unassociated_row_count": report["summary"].get(
                "fret_text_paint_unassociated_row_count", 0
            ),
            "fret_text_label_row_count": report["summary"].get(
                "fret_text_label_row_count", 0
            ),
            "repair_queue_count": non_passing,
            "hardening_queue_count": 0,
            "gate_queue_count": 0,
        },
        "repair_queue": top_findings if non_passing else [],
        "hardening_queue": [],
        "gate_queue": [],
        "legacy_source_report": str(path).replace("\\", "/"),
    }


def _merge_counts(
    reports: list[dict[str, Any]], summary_key: str, ordered_keys: list[str]
) -> dict[str, int]:
    counts = {key: 0 for key in ordered_keys}
    for report in reports:
        for key, value in report["summary"][summary_key].items():
            counts[key] = counts.get(key, 0) + int(value)
    return counts


def generate_suite_report(
    suite: dict[str, Any],
    suite_path: Path,
    reports: list[dict[str, Any]],
) -> dict[str, Any]:
    report_rows = []
    top_findings = []
    agent_rows = []
    for report_spec, report in zip(suite["reports"], reports, strict=True):
        report_id = report_spec["id"]
        output = str(Path(report_spec["output"])).replace("\\", "/")
        report_rows.append(
            {
                "id": report_id,
                "component": report["component"],
                "style": report["style"],
                "output": output,
                "status_counts": report["summary"]["status_counts"],
                "layer_status_counts": report["summary"]["layer_status_counts"],
                "triage_level_counts": report["summary"]["triage_level_counts"],
                "top_findings": report["summary"]["top_findings"],
            }
        )
        agent_packet = report["agent_packet"]
        agent_rows.append(
            {
                "id": report_id,
                "component": report["component"],
                "output": output,
                "status": agent_packet["status"],
                "repair_queue_count": agent_packet["summary"]["repair_queue_count"],
                "hardening_queue_count": agent_packet["summary"][
                    "hardening_queue_count"
                ],
                "gate_queue_count": agent_packet["summary"]["gate_queue_count"],
            }
        )
        for finding in report["summary"]["top_findings"]:
            enriched = dict(finding)
            enriched["report_id"] = report_id
            enriched["component"] = report["component"]
            enriched["report_output"] = output
            top_findings.append(enriched)

    top_findings.sort(
        key=lambda item: (
            -int(item["triage"]["score"]),
            item["report_id"],
            item["part_id"],
            item["check_id"],
        )
    )

    repair_count = sum(item["repair_queue_count"] for item in agent_rows)
    hardening_count = sum(item["hardening_queue_count"] for item in agent_rows)

    return {
        "schema_version": SUPPORTED_SCHEMA_VERSION,
        "suite_id": suite["id"],
        "generated_date": suite.get("generated_date"),
        "generated_by": "tools/parity-discovery/shadcn_parity_discovery.py",
        "source_suite": str(suite_path).replace("\\", "/"),
        "summary": {
            "report_count": len(reports),
            "part_count": sum(report["summary"]["part_count"] for report in reports),
            "status_counts": _merge_counts(reports, "status_counts", STATUS_ORDER),
            "layer_counts": _merge_counts(reports, "layer_counts", LAYER_ORDER),
            "triage_level_counts": _merge_counts(
                reports, "triage_level_counts", TRIAGE_LEVEL_ORDER
            ),
            "top_findings": top_findings[:20],
        },
        "agent_packet": {
            "schema_version": 1,
            "status": (
                "needs_repair"
                if repair_count
                else "needs_hardening"
                if hardening_count
                else "regression_locked"
            ),
            "suite_id": suite["id"],
            "goal": (
                "Summarize per-component repair readiness without duplicating each "
                "component report's full agent packet."
            ),
            "repair_queue_count": repair_count,
            "hardening_queue_count": hardening_count,
            "reports": agent_rows,
            "top_findings": top_findings[:20],
        },
        "reports": report_rows,
    }


def write_report(report: dict[str, Any], output_path: Path) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(report, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
        newline="\n",
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate a deterministic shadcn parity discovery report."
    )
    parser.add_argument(
        "--mapping", type=Path, help="Path to a mapping fixture JSON file."
    )
    parser.add_argument(
        "--output", type=Path, help="Path to write the report JSON artifact."
    )
    parser.add_argument(
        "--suite",
        type=Path,
        help="Path to a suite manifest that generates multiple report artifacts.",
    )
    parser.add_argument(
        "--suite-output",
        type=Path,
        help="Path to write the generated suite summary JSON artifact.",
    )
    parser.add_argument(
        "--suite-from-existing-reports",
        action="store_true",
        help=(
            "Build only the suite summary from each report output path instead of "
            "regenerating component reports. Useful when archived sidecars are not "
            "available in the current worktree."
        ),
    )
    parser.add_argument(
        "--fret-layout-sidecar",
        action="append",
        type=Path,
        default=[],
        help="Path to a Fret layout.taffy.v1.json sidecar. May be repeated.",
    )
    parser.add_argument(
        "--fret-layout-sidecar-dir",
        action="append",
        type=Path,
        default=[],
        help="Directory to search recursively for layout.taffy.v1.json sidecars.",
    )
    parser.add_argument(
        "--fret-bundle-schema2",
        action="append",
        type=Path,
        default=[],
        help="Path to a bundle.schema2.json file. May be repeated.",
    )
    parser.add_argument(
        "--fret-bundle-schema2-dir",
        action="append",
        type=Path,
        default=[],
        help="Directory to search recursively for bundle.schema2.json files.",
    )
    parser.add_argument(
        "--upstream-dom-snapshot",
        action="append",
        type=Path,
        default=[],
        help="Path to an upstream shadcn DOM snapshot JSON. May be repeated.",
    )
    parser.add_argument(
        "--upstream-dom-snapshot-dir",
        action="append",
        type=Path,
        default=[],
        help="Directory to search recursively for upstream DOM snapshot JSON files.",
    )
    return parser.parse_args()


def resolve_sidecar_paths(args: argparse.Namespace) -> list[Path]:
    return collect_sidecar_paths(
        list(args.fret_layout_sidecar),
        list(args.fret_layout_sidecar_dir),
    )


def resolve_bundle_schema2_paths(
    args: argparse.Namespace, sidecar_paths: list[Path]
) -> list[Path]:
    return collect_bundle_schema2_paths(
        list(args.fret_bundle_schema2),
        list(args.fret_bundle_schema2_dir),
        sidecar_paths,
    )


def resolve_dom_snapshot_paths(args: argparse.Namespace) -> list[Path]:
    return collect_dom_snapshot_paths(
        list(args.upstream_dom_snapshot),
        list(args.upstream_dom_snapshot_dir),
    )


def main() -> int:
    args = parse_args()
    try:
        if args.suite is not None:
            if args.mapping is not None or args.output is not None:
                raise FixtureError("--suite cannot be combined with --mapping or --output")
            if args.suite_output is None:
                raise FixtureError("--suite requires --suite-output")
            suite = load_suite(args.suite)
            if args.suite_from_existing_reports:
                reports = [
                    load_generated_report(Path(item["output"]))
                    for item in suite["reports"]
                ]
            else:
                reports = [generate_report_from_spec(item) for item in suite["reports"]]
            suite_report = generate_suite_report(suite, args.suite, reports)
            write_report(suite_report, args.suite_output)
            print(
                "generated "
                f"{args.suite_output} "
                f"({suite_report['summary']['report_count']} reports, "
                f"{suite_report['summary']['part_count']} parts, "
                f"{len(suite_report['summary']['top_findings'])} top findings)"
            )
            return 0

        if args.mapping is None or args.output is None:
            raise FixtureError("--mapping and --output are required unless --suite is used")

        mapping = load_mapping(args.mapping)
        layout_sidecar_paths = resolve_sidecar_paths(args)
        layout_evidence = load_layout_evidence(
            layout_sidecar_paths,
            resolve_bundle_schema2_paths(args, layout_sidecar_paths),
        )
        dom_targets = mapping.get("upstream_dom_targets", [])
        dom_evidence = load_dom_evidence(
            resolve_dom_snapshot_paths(args),
            dom_targets,
            mapping.get("upstream_contexts", []),
        )
        report = generate_report(
            mapping, args.mapping, layout_evidence, dom_evidence
        )
        write_report(report, args.output)
    except FixtureError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2
    except OSError as exc:
        print(f"error: {exc}", file=sys.stderr)
        return 2

    print(
        "generated "
        f"{args.output} "
        f"({report['summary']['part_count']} parts, "
        f"{report['summary']['layout_sidecar_count']} layout sidecars, "
        f"{report['summary']['bundle_schema2_count']} bundle schema2 files, "
        f"{report['summary']['upstream_dom_snapshot_count']} upstream DOM snapshots)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
