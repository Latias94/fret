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
TEST_ID_LABEL_RE = re.compile(r"\[test_id=([^\]]+)\]")
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


@dataclass(frozen=True)
class LayoutRoot:
    bounds: Bounds
    raw_bounds: Bounds
    scale_factor: float
    coordinate_units: str
    sidecar_path: str


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
    viewport: dict[str, Any]
    path: str
    tag: str
    attrs: dict[str, Any]
    class_name: str | None


@dataclass
class LayoutEvidence:
    nodes_by_test_id: dict[str, list[LayoutNode]]
    sidecar_paths: list[str]
    roots: list[LayoutRoot]

    @classmethod
    def empty(cls) -> "LayoutEvidence":
        return cls(nodes_by_test_id={}, sidecar_paths=[], roots=[])

    def find(self, test_id: str) -> LayoutNode | None:
        nodes = self.nodes_by_test_id.get(test_id)
        if not nodes:
            return None
        return nodes[0]

    def duplicate_count(self, test_id: str) -> int:
        return len(self.nodes_by_test_id.get(test_id, []))

    def find_root(self) -> LayoutRoot | None:
        if not self.roots:
            return None
        return self.roots[0]


@dataclass
class DomEvidence:
    nodes_by_target_id: dict[str, DomNode]
    snapshot_paths: list[str]
    contexts: list[dict[str, Any]]

    @classmethod
    def empty(cls) -> "DomEvidence":
        return cls(nodes_by_target_id={}, snapshot_paths=[], contexts=[])

    def find(self, target_id: str) -> DomNode | None:
        return self.nodes_by_target_id.get(target_id)


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


def _validate_upstream_contexts(mapping: dict[str, Any]) -> None:
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
    _validate_upstream_contexts(mapping)

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
    debug = node.get("debug") if isinstance(node.get("debug"), dict) else {}
    result: list[str] = []
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


def load_layout_evidence(paths: list[Path]) -> LayoutEvidence:
    evidence = LayoutEvidence.empty()
    for path in sorted(paths):
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
                    )
                )

    for nodes in evidence.nodes_by_test_id.values():
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
        "viewport": viewport,
        "device_pixel_ratio": float(theme_data.get("devicePixelRatio") or 1.0),
        "snapshot_path": snapshot_path,
    }


def load_dom_evidence(
    paths: list[Path], targets: list[dict[str, Any]]
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
            wanted_targets = targets_by_snapshot.get(
                (snapshot_name, theme, snapshot_mode, snapshot_variant), []
            )
            if not wanted_targets:
                continue
            evidence.contexts.append(
                _dom_context_from_snapshot(
                    snapshot_path,
                    snapshot_name,
                    theme,
                    snapshot_mode,
                    snapshot_variant,
                    raw_theme_data,
                )
            )
            nodes_by_path = _snapshot_dom_nodes(raw_theme_data)
            device_pixel_ratio = float(raw_theme_data.get("devicePixelRatio") or 1.0)
            for target in wanted_targets:
                node = nodes_by_path.get(target["path"])
                if node is None:
                    continue
                rect = node.get("rect")
                if not isinstance(rect, dict):
                    continue
                attrs = node.get("attrs") if isinstance(node.get("attrs"), dict) else {}
                class_name = node.get("className")
                evidence.nodes_by_target_id[target["id"]] = DomNode(
                    target_id=target["id"],
                    bounds=Bounds.from_rect(rect),
                    device_pixel_ratio=device_pixel_ratio,
                    snapshot_path=snapshot_path,
                    snapshot_name=snapshot_name,
                    theme=theme,
                    mode=snapshot_mode,
                    variant=snapshot_variant,
                    viewport=raw_theme_data.get("viewport")
                    if isinstance(raw_theme_data.get("viewport"), dict)
                    else {},
                    path=target["path"],
                    tag=str(node.get("tag", "")),
                    attrs=attrs,
                    class_name=class_name if isinstance(class_name, str) else None,
                )
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
    if kind == "root_metric":
        root = evidence.find_root()
        if root is None:
            return {
                "kind": kind,
                "status": "missing",
                "metric": metric,
                "reason": "missing_root_bounds",
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
            "sidecar_path": root.sidecar_path,
            "duplicate_count": len(evidence.roots),
        }
    if kind == "bounds_metric":
        target = predicate["target"]
        node = evidence.find(target)
        if node is None:
            return {
                "kind": kind,
                "status": "missing",
                "target": target,
                "metric": metric,
                "reason": "missing_test_id",
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
            "sidecar_path": node.sidecar_path,
            "duplicate_count": evidence.duplicate_count(target),
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
                "reason": "missing_test_id",
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
            "sidecar_path": a_node.sidecar_path,
            "a_duplicate_count": evidence.duplicate_count(a_id),
            "b_duplicate_count": evidence.duplicate_count(b_id),
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
    if not evidence.sidecar_paths:
        return {
            "source": "fret_layout_sidecar",
            "status": "missing",
            "reason": "no_layout_sidecars_provided",
            "predicate_count": len(predicates),
            "predicates": [],
        }

    predicate_results = [
        evaluate_predicate(predicate, evidence) for predicate in predicates
    ]
    return {
        "source": "fret_layout_sidecar",
        "status": _measurement_status(predicate_results),
        "sidecar_paths": evidence.sidecar_paths,
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
        "source": "fret_layout_sidecar+upstream_dom_snapshot",
        "status": status,
        "fret": fret,
        "upstream_dom": upstream,
        "comparisons": comparisons,
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
        for check in part["checks"]:
            measurement = combine_measurements(
                evaluate_fret_measurement(check, layout_evidence),
                evaluate_upstream_dom_measurement(check, dom_evidence),
            )
            status = check_status(check, measurement)
            owner = _resolve_owner(check)
            layer = _resolve_layer(check, owner)
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
                "evidence_refs": check["evidence_refs"],
                "promotion": check["promotion"],
            }
            if measurement is not None:
                report_check["measurement"] = measurement
            checks.append(report_check)

        part_status = merge_part_status(check_statuses)
        status_counts[part_status] += 1
        report_parts.append(
            {
                "id": part["id"],
                "label": part["label"],
                "axis": part["axis"],
                "status": part_status,
                "confidence": merge_confidence(part["checks"]),
                "test_ids": part["fret"]["test_ids"],
                "upstream_facts": part["upstream"]["facts"],
                "fret_facts": part["fret"]["facts"],
                "checks": checks,
            }
        )

    return {
        "schema_version": SUPPORTED_SCHEMA_VERSION,
        "component": mapping["component"],
        "style": mapping["style"],
        "generated_date": mapping["report"]["generated_date"],
        "generated_by": "tools/parity-discovery/shadcn_parity_discovery.py",
        "source_mapping": str(mapping_path).replace("\\", "/"),
        "upstream_contexts": mapping.get("upstream_contexts", []),
        "evidence_contexts": {
            "upstream_dom": dom_evidence.contexts,
        },
        "summary": {
            "part_count": len(report_parts),
            "status_counts": status_counts,
            "owner_counts": owner_counts,
            "owner_status_counts": owner_status_counts,
            "layer_counts": layer_counts,
            "layer_status_counts": layer_status_counts,
            "promotion_target_counts": promotion_counts,
            "layout_sidecar_count": len(layout_evidence.sidecar_paths),
            "measured_test_id_count": len(layout_evidence.nodes_by_test_id),
            "upstream_dom_snapshot_count": len(dom_evidence.snapshot_paths),
            "upstream_dom_target_count": len(dom_evidence.nodes_by_target_id),
            "upstream_context_count": len(mapping.get("upstream_contexts", [])),
            "upstream_dom_context_count": len(dom_evidence.contexts),
        },
        "parts": report_parts,
        "limitations": mapping["report"]["limitations"],
    }


def write_report(report: dict[str, Any], output_path: Path) -> None:
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(
        json.dumps(report, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate a deterministic shadcn parity discovery report."
    )
    parser.add_argument(
        "--mapping", required=True, type=Path, help="Path to a mapping fixture JSON file."
    )
    parser.add_argument(
        "--output", required=True, type=Path, help="Path to write the report JSON artifact."
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
    paths = list(args.fret_layout_sidecar)
    for directory in args.fret_layout_sidecar_dir:
        if not directory.exists():
            raise FixtureError(f"layout sidecar directory does not exist: {directory}")
        paths.extend(directory.rglob("layout.taffy.v1.json"))
    unique: dict[str, Path] = {}
    for path in paths:
        if not path.exists():
            raise FixtureError(f"layout sidecar does not exist: {path}")
        unique[str(path.resolve())] = path
    return [unique[key] for key in sorted(unique)]


def resolve_dom_snapshot_paths(args: argparse.Namespace) -> list[Path]:
    paths = list(args.upstream_dom_snapshot)
    for directory in args.upstream_dom_snapshot_dir:
        if not directory.exists():
            raise FixtureError(f"upstream DOM snapshot directory does not exist: {directory}")
        paths.extend(directory.rglob("*.json"))
    unique: dict[str, Path] = {}
    for path in paths:
        if not path.exists():
            raise FixtureError(f"upstream DOM snapshot does not exist: {path}")
        unique[str(path.resolve())] = path
    return [unique[key] for key in sorted(unique)]


def main() -> int:
    args = parse_args()
    try:
        mapping = load_mapping(args.mapping)
        layout_evidence = load_layout_evidence(resolve_sidecar_paths(args))
        dom_targets = mapping.get("upstream_dom_targets", [])
        dom_evidence = load_dom_evidence(resolve_dom_snapshot_paths(args), dom_targets)
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
        f"{report['summary']['upstream_dom_snapshot_count']} upstream DOM snapshots)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
