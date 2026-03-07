#!/usr/bin/env python3
"""Summarize selected xctrace tables for hello_world_compare investigations."""

from __future__ import annotations

import argparse
import json
import plistlib
import struct
import subprocess
import xml.etree.ElementTree as ET
import zlib
from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

DEFAULT_SCHEMAS = [
    "display-surface-queue",
    "ca-client-present-request",
    "displayed-surfaces-interval",
    "metal-application-encoders-list",
]

GAME_MEMORY_ATTACH_SCHEMAS = [
    "metal-current-allocated-size",
    "metal-io-surface-access",
    "metal-resource-allocations",
    "virtual-memory",
    "metal-residency-set-interval",
    "metal-residency-set-usage-event",
    "metal-residency-set-resource-event",
]

PRESET_SCHEMAS = {
    "default": DEFAULT_SCHEMAS,
    "game-memory-attach": GAME_MEMORY_ATTACH_SCHEMAS,
}

FILTER_SUMMARY_EXCLUDED_KEYS = {
    "schema",
    "present",
    "columns",
    "has_process_column",
    "has_thread_column",
    "note",
}

DIRECT_STORE_FALLBACK_SCHEMAS = {
    "metal-current-allocated-size",
    "metal-io-surface-access",
    "metal-resource-allocations",
    "virtual-memory",
}

DIRECT_STORE_FIELD_SIZES = {
    1: 4,
    2: 8,
    6: 24,
    7: 16,
}


@dataclass
class ExportedTable:
    schema: str
    xml: str
    export_source: str
    export_status: str = "ok"
    cache_path: str | None = None


@dataclass
class ExportFailure:
    schema: str
    export_status: str
    export_source: str
    export_error: str | None = None
    cache_path: str | None = None
    partial_cache_path: str | None = None


class TableAccumulator:
    def __init__(self) -> None:
        self.timestamp_values: list[int] = []
        self.process_counter: Counter[str] = Counter()
        self.thread_counter: Counter[str] = Counter()
        self.surface_counter: Counter[str] = Counter()
        self.surface_extent_counter: Counter[str] = Counter()
        self.pixel_format_counter: Counter[str] = Counter()
        self.cmd_buffer_counter: Counter[str] = Counter()
        self.detachment_counter: Counter[str] = Counter()
        self.direct_to_display_counter: Counter[str] = Counter()
        self.label_counter: Counter[str] = Counter()
        self.event_type_counter: Counter[str] = Counter()
        self.resource_type_counter: Counter[str] = Counter()
        self.allocation_type_counter: Counter[str] = Counter()
        self.operation_counter: Counter[str] = Counter()
        self.track_name_counter: Counter[str] = Counter()
        self.access_type_counter: Counter[str] = Counter()
        self.residency_set_counter: Counter[str] = Counter()
        self.gpu_counter: Counter[str] = Counter()
        self.encoder_duration_total_ns = 0
        self.current_allocated_sizes: list[int] = []
        self.vidmem_values: list[int] = []
        self.sysmem_values: list[int] = []
        self.resource_size_values: list[int] = []
        self.allocation_size_values: list[int] = []
        self.size_values: list[int] = []
        self.row_count = 0

    def observe(
        self,
        row_map: dict[str, ET.Element],
        index: dict[str, ET.Element],
        *,
        process_name: str | None = None,
    ) -> None:
        self.row_count += 1

        ts_elem = first_present(row_map, "timestamp", "start")
        if ts_elem is not None:
            ts_raw = cell_raw_int(ts_elem, index)
            if ts_raw is not None:
                self.timestamp_values.append(ts_raw)

        if process_name is None:
            process_name = extract_fmt(row_map, index, "process")
        if process_name:
            self.process_counter[process_name] += 1

        thread_name = extract_fmt(row_map, index, "thread")
        if thread_name:
            self.thread_counter[thread_name] += 1

        surface_id = extract_fmt(row_map, index, "surface-id", "swap-id")
        if surface_id:
            self.surface_counter[surface_id] += 1

        width = extract_int(row_map, index, "width")
        height = extract_int(row_map, index, "height")
        if width is not None and height is not None:
            pixel_format = extract_fmt(row_map, index, "pixel-format")
            extent = f"{width}x{height}" if not pixel_format else f"{width}x{height} {pixel_format}"
            self.surface_extent_counter[extent] += 1
            if pixel_format:
                self.pixel_format_counter[pixel_format] += 1

        cmd_id = extract_fmt(row_map, index, "cmdbuffer-id", "command-buffer")
        if cmd_id:
            self.cmd_buffer_counter[cmd_id] += 1

        detachment = extract_fmt(row_map, index, "detachment-reason")
        if detachment:
            self.detachment_counter[detachment] += 1

        direct = extract_fmt(row_map, index, "direct-to-display")
        if direct:
            self.direct_to_display_counter[direct] += 1

        label = extract_fmt(
            row_map,
            index,
            "command-buffer-label",
            "cmdbuffer-label",
            "encoder-label",
            "allocation-label",
            "metal-object-label",
            "event-label",
            "label",
        )
        if label:
            self.label_counter[label] += 1

        duration_ns = extract_int(row_map, index, "duration")
        if duration_ns is not None:
            self.encoder_duration_total_ns += duration_ns

        event_type = extract_fmt(row_map, index, "event-type")
        if event_type:
            self.event_type_counter[event_type] += 1

        resource_type = extract_fmt(row_map, index, "resource-type")
        if resource_type:
            self.resource_type_counter[resource_type] += 1

        allocation_type = extract_fmt(row_map, index, "allocation-type")
        if allocation_type:
            self.allocation_type_counter[allocation_type] += 1

        operation = extract_fmt(row_map, index, "operation")
        if operation:
            self.operation_counter[operation] += 1

        track_name = extract_fmt(row_map, index, "track-name")
        if track_name:
            self.track_name_counter[track_name] += 1

        access_type = extract_fmt(row_map, index, "access-type")
        if access_type:
            self.access_type_counter[access_type] += 1

        residency_set = extract_fmt(row_map, index, "residency-set-id")
        if residency_set:
            self.residency_set_counter[residency_set] += 1

        gpu_name = extract_fmt(row_map, index, "gpu")
        if gpu_name:
            self.gpu_counter[gpu_name] += 1

        current_allocated = extract_int(row_map, index, "current-allocated-size")
        if current_allocated is not None:
            self.current_allocated_sizes.append(current_allocated)

        vidmem = extract_int(row_map, index, "vidmem-bytes")
        if vidmem is not None:
            self.vidmem_values.append(vidmem)

        sysmem = extract_int(row_map, index, "sysmem-bytes")
        if sysmem is not None:
            self.sysmem_values.append(sysmem)

        resource_size = extract_int(row_map, index, "resource-size")
        if resource_size is not None:
            self.resource_size_values.append(resource_size)

        allocation_size = extract_int(row_map, index, "allocation-size")
        if allocation_size is not None:
            self.allocation_size_values.append(allocation_size)

        size_value = extract_int(row_map, index, "size")
        if size_value is not None:
            self.size_values.append(size_value)

    def render(self, schema_name: str, columns: list[str]) -> dict[str, Any]:
        first_ts = self.timestamp_values[0] if self.timestamp_values else None
        last_ts = self.timestamp_values[-1] if self.timestamp_values else None
        duration_secs = None
        approx_hz = None
        if (
            first_ts is not None
            and last_ts is not None
            and last_ts > first_ts
            and len(self.timestamp_values) > 1
        ):
            duration_secs = (last_ts - first_ts) / 1_000_000_000.0
            approx_hz = (len(self.timestamp_values) - 1) / duration_secs

        summary: dict[str, Any] = {
            "schema": schema_name,
            "present": True,
            "columns": columns,
            "row_count": self.row_count,
            "has_process_column": "process" in columns,
            "has_thread_column": "thread" in columns,
            "timestamp_first_ns": first_ts,
            "timestamp_last_ns": last_ts,
            "duration_secs": duration_secs,
            "approx_hz": approx_hz,
        }
        if self.process_counter:
            summary["process_counts"] = dict(self.process_counter.most_common(12))
        if self.thread_counter:
            summary["thread_counts_head"] = dict(self.thread_counter.most_common(12))
        if self.surface_counter:
            summary["surface_ids_count"] = len(self.surface_counter)
            summary["surface_ids_head"] = dict(self.surface_counter.most_common(12))
        if self.surface_extent_counter:
            summary["surface_extents_head"] = dict(self.surface_extent_counter.most_common(12))
        if self.pixel_format_counter:
            summary["pixel_formats_head"] = dict(self.pixel_format_counter.most_common(12))
        if self.cmd_buffer_counter:
            summary["command_buffers_count"] = len(self.cmd_buffer_counter)
        if self.detachment_counter:
            summary["detachment_reasons"] = dict(self.detachment_counter.most_common(12))
        if self.direct_to_display_counter:
            summary["direct_to_display"] = dict(self.direct_to_display_counter.most_common())
        if self.label_counter:
            labels_head = dict(self.label_counter.most_common(12))
            summary["labels_head"] = labels_head
            if schema_name == "metal-application-encoders-list":
                summary["encoder_labels"] = labels_head
                summary["encoder_duration_total_ms"] = self.encoder_duration_total_ns / 1_000_000.0
        if self.event_type_counter:
            summary["event_types_head"] = dict(self.event_type_counter.most_common(12))
        if self.resource_type_counter:
            summary["resource_types_head"] = dict(self.resource_type_counter.most_common(12))
        if self.allocation_type_counter:
            summary["allocation_types_head"] = dict(self.allocation_type_counter.most_common(12))
        if self.operation_counter:
            summary["operations_head"] = dict(self.operation_counter.most_common(12))
        if self.track_name_counter:
            summary["track_names_head"] = dict(self.track_name_counter.most_common(12))
        if self.access_type_counter:
            summary["access_types_head"] = dict(self.access_type_counter.most_common(12))
        if self.residency_set_counter:
            summary["residency_set_ids_count"] = len(self.residency_set_counter)
            summary["residency_set_ids_head"] = dict(self.residency_set_counter.most_common(12))
        if self.gpu_counter:
            summary["gpus_head"] = dict(self.gpu_counter.most_common(12))
        if self.current_allocated_sizes:
            summary["current_allocated_size_bytes_first"] = self.current_allocated_sizes[0]
            summary["current_allocated_size_bytes_last"] = self.current_allocated_sizes[-1]
            summary["current_allocated_size_bytes_min"] = min(self.current_allocated_sizes)
            summary["current_allocated_size_bytes_max"] = max(self.current_allocated_sizes)
            summary["current_allocated_size_unique_values_head"] = sorted(set(self.current_allocated_sizes))[:12]
        if self.vidmem_values:
            summary["vidmem_bytes_sum"] = sum(self.vidmem_values)
            summary["vidmem_bytes_max"] = max(self.vidmem_values)
        if self.sysmem_values:
            summary["sysmem_bytes_sum"] = sum(self.sysmem_values)
            summary["sysmem_bytes_max"] = max(self.sysmem_values)
        if self.resource_size_values:
            summary["resource_size_bytes_sum"] = sum(self.resource_size_values)
            summary["resource_size_bytes_max"] = max(self.resource_size_values)
        if self.allocation_size_values:
            summary["allocation_size_bytes_sum"] = sum(self.allocation_size_values)
            summary["allocation_size_bytes_max"] = max(self.allocation_size_values)
        if self.size_values:
            summary["size_bytes_sum"] = sum(self.size_values)
            summary["size_bytes_max"] = max(self.size_values)
        if schema_name == "display-surface-queue" and "process" not in columns:
            summary["note"] = "Global table: no process column; do not treat as app-only frame cadence."
        elif schema_name == "metal-resource-allocations":
            summary["note"] = (
                "Event stream: *_bytes_sum values add allocation event payloads and are not "
                "equivalent to live residency by themselves."
            )
        elif schema_name == "virtual-memory":
            summary["note"] = "Event stream: virtual-memory rows are VM operations, not a direct current-footprint snapshot."
        return summary


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("trace", help="Path to the .trace bundle")
    parser.add_argument(
        "--schema",
        action="append",
        default=[],
        help="Specific schema(s) to export. Overrides --preset when provided.",
    )
    parser.add_argument(
        "--preset",
        choices=sorted(PRESET_SCHEMAS),
        help="Named schema preset to export.",
    )
    parser.add_argument(
        "--process-contains",
        help="Case-insensitive substring used to emit filtered summaries for matching process rows.",
    )
    parser.add_argument(
        "--pid-equals",
        type=int,
        help="Optional numeric pid filter used for exported tables and direct-store fallbacks that expose a `pid` column.",
    )
    parser.add_argument(
        "--export-dir",
        help="Optional directory used to cache exported XML tables and reuse them on later runs.",
    )
    parser.add_argument(
        "--export-timeout-secs",
        type=float,
        help="Optional timeout for each `xctrace export` invocation. When hit, the table is marked as timed out.",
    )
    parser.add_argument(
        "--list-store-schemas",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Inspect the trace bundle directly and list indexed-store schema names/columns without calling xctrace export.",
    )
    parser.add_argument(
        "--store-schema-filter",
        action="append",
        default=[],
        help="Limit `--list-store-schemas` output to schema names containing these substrings. Repeatable.",
    )
    parser.add_argument("--output", help="Optional output JSON path")
    return parser.parse_args()


def dedupe_preserving_order(values: Iterable[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        result.append(value)
    return result


def resolve_schemas(args: argparse.Namespace) -> list[str]:
    if args.schema:
        return dedupe_preserving_order(args.schema)
    if args.preset:
        return list(PRESET_SCHEMAS[args.preset])
    return list(DEFAULT_SCHEMAS)


def cache_path_for_schema(export_dir: Path, schema: str) -> Path:
    filename = schema.replace("/", "_") + ".xml"
    return export_dir / filename


def load_cached_export(schema: str, path: Path) -> ExportedTable | ExportFailure:
    xml = path.read_text(errors="replace")
    if contains_export_payload(xml):
        return ExportedTable(
            schema=schema,
            xml=xml,
            export_source="cache",
            export_status="ok",
            cache_path=str(path),
        )
    return ExportFailure(
        schema=schema,
        export_status="cached-empty",
        export_source="cache",
        export_error="Cached export does not contain table rows.",
        cache_path=str(path),
    )


def contains_export_payload(xml: str) -> bool:
    return "<row>" in xml or "<node " in xml


def export_table(
    trace: Path,
    schema: str,
    *,
    export_dir: Path | None,
    timeout_secs: float | None,
) -> ExportedTable | ExportFailure:
    cache_path = cache_path_for_schema(export_dir, schema) if export_dir else None
    if cache_path and cache_path.exists():
        return load_cached_export(schema, cache_path)

    xpath = f'/trace-toc/run[@number="1"]/data/table[@schema="{schema}"]'
    try:
        proc = subprocess.run(
            ["xcrun", "xctrace", "export", "--input", str(trace), "--xpath", xpath],
            capture_output=True,
            text=True,
            timeout=timeout_secs if timeout_secs and timeout_secs > 0 else None,
            check=False,
        )
    except subprocess.TimeoutExpired as exc:
        partial_stdout = exc.stdout or ""
        if isinstance(partial_stdout, bytes):
            partial_stdout = partial_stdout.decode("utf-8", errors="replace")
        partial_cache_path = None
        if cache_path and partial_stdout:
            cache_path.parent.mkdir(parents=True, exist_ok=True)
            partial_path = cache_path.with_suffix(".partial.xml")
            partial_path.write_text(partial_stdout)
            partial_cache_path = str(partial_path)
        return ExportFailure(
            schema=schema,
            export_status="timed_out",
            export_source="xctrace",
            export_error=f"xctrace export timed out after {timeout_secs:g}s",
            cache_path=str(cache_path) if cache_path and cache_path.exists() else None,
            partial_cache_path=partial_cache_path,
        )

    if proc.returncode != 0:
        return ExportFailure(
            schema=schema,
            export_status="error",
            export_source="xctrace",
            export_error=proc.stderr.strip() or proc.stdout.strip(),
            cache_path=str(cache_path) if cache_path and cache_path.exists() else None,
        )

    if cache_path:
        cache_path.parent.mkdir(parents=True, exist_ok=True)
        cache_path.write_text(proc.stdout)

    if not contains_export_payload(proc.stdout):
        return ExportFailure(
            schema=schema,
            export_status="empty",
            export_source="xctrace",
            export_error="xctrace export returned no table rows.",
            cache_path=str(cache_path) if cache_path and cache_path.exists() else None,
        )

    return ExportedTable(
        schema=schema,
        xml=proc.stdout,
        export_source="xctrace",
        export_status="ok",
        cache_path=str(cache_path) if cache_path and cache_path.exists() else None,
    )


def build_id_index(node: ET.Element) -> dict[str, ET.Element]:
    index: dict[str, ET.Element] = {}
    for elem in node.iter():
        elem_id = elem.attrib.get("id")
        if elem_id:
            index[elem_id] = elem
    return index


def resolve_elem(elem: ET.Element, index: dict[str, ET.Element]) -> ET.Element:
    ref = elem.attrib.get("ref")
    if ref:
        return resolve_elem(index[ref], index)
    return elem


def cell_fmt(elem: ET.Element, index: dict[str, ET.Element]) -> str | None:
    resolved = resolve_elem(elem, index)
    if "fmt" in resolved.attrib:
        return resolved.attrib["fmt"]
    text = (resolved.text or "").strip()
    return text or None


def cell_raw_int(elem: ET.Element, index: dict[str, ET.Element]) -> int | None:
    resolved = resolve_elem(elem, index)
    text = (resolved.text or "").strip()
    if not text:
        return None
    try:
        return int(text)
    except ValueError:
        return None


def first_present(row_map: dict[str, ET.Element], *names: str) -> ET.Element | None:
    for name in names:
        elem = row_map.get(name)
        if elem is not None:
            return elem
    return None


def extract_fmt(
    row_map: dict[str, ET.Element],
    index: dict[str, ET.Element],
    *names: str,
) -> str | None:
    elem = first_present(row_map, *names)
    if elem is None:
        return None
    return cell_fmt(elem, index)


def extract_int(
    row_map: dict[str, ET.Element],
    index: dict[str, ET.Element],
    *names: str,
) -> int | None:
    elem = first_present(row_map, *names)
    if elem is None:
        return None
    return cell_raw_int(elem, index)


def load_store_schema_xml(path: Path) -> str:
    raw = path.read_bytes()
    try:
        return zlib.decompress(raw).decode("utf-8", errors="replace")
    except zlib.error:
        return raw.decode("utf-8", errors="replace")


def load_zlib_or_raw(path: Path) -> bytes:
    raw = path.read_bytes()
    try:
        return zlib.decompress(raw)
    except zlib.error:
        return raw


def parse_store_schema(path: Path) -> tuple[str, list[str]]:
    xml_text = load_store_schema_xml(path)
    root = ET.fromstring(xml_text)
    schema_name = root.attrib.get("name") or path.parent.name
    columns = [column.attrib.get("mnemonic") or "" for column in root.findall("column")]
    return schema_name, columns


def list_store_schemas(trace: Path, filters: list[str]) -> list[dict[str, Any]]:
    lowered_filters = [item.lower() for item in filters if item.strip()]
    rows: list[dict[str, Any]] = []
    for schema_path in sorted(trace.glob("corespace/run*/core/stores/indexed-store-*/schema.xml")):
        schema_name, columns = parse_store_schema(schema_path)
        if lowered_filters and not any(token in schema_name.lower() for token in lowered_filters):
            continue
        rows.append(
            {
                "store": schema_path.parent.name,
                "schema": schema_name,
                "columns": columns,
                "path": str(schema_path),
            }
        )
    return rows


def iter_row_maps(rows: Iterable[ET.Element], columns: list[str]) -> Iterable[dict[str, ET.Element]]:
    for row in rows:
        cells = list(row)
        yield {columns[ix]: cells[ix] for ix in range(min(len(columns), len(cells)))}


def merge_filtered_summary(summary: dict[str, Any], filtered: dict[str, Any]) -> None:
    for key, value in filtered.items():
        if key in FILTER_SUMMARY_EXCLUDED_KEYS:
            continue
        summary[f"filtered_{key}"] = value


def summarize_table(
    exported: ExportedTable,
    process_contains: str | None,
    pid_equals: int | None,
) -> dict[str, Any]:
    root = ET.fromstring(exported.xml)
    node = root.find(".//node")
    if node is None:
        return {
            "schema": exported.schema,
            "present": False,
            "export_status": exported.export_status,
            "export_source": exported.export_source,
            "cache_path": exported.cache_path,
        }

    schema = node.find("schema")
    if schema is None:
        return {
            "schema": exported.schema,
            "present": False,
            "export_status": exported.export_status,
            "export_source": exported.export_source,
            "cache_path": exported.cache_path,
        }

    columns = [col.findtext("mnemonic") or f"col_{ix}" for ix, col in enumerate(schema.findall("col"))]
    rows = node.findall("row")
    index = build_id_index(node)

    accumulator = TableAccumulator()
    filter_requested = process_contains is not None or pid_equals is not None
    filtered_accumulator = TableAccumulator() if filter_requested else None
    lowered_process_filter = process_contains.lower() if process_contains else None
    filter_match_count = 0

    for row_map in iter_row_maps(rows, columns):
        process_name = extract_fmt(row_map, index, "process")
        pid_value = extract_int(row_map, index, "pid")
        accumulator.observe(row_map, index, process_name=process_name)
        if filtered_accumulator is not None:
            process_matches = True
            pid_matches = True
            if lowered_process_filter is not None:
                process_matches = process_name is not None and lowered_process_filter in process_name.lower()
            if pid_equals is not None:
                pid_matches = pid_value == pid_equals
            if process_matches and pid_matches:
                filtered_accumulator.observe(row_map, index, process_name=process_name)
                filter_match_count += 1

    summary = accumulator.render(exported.schema, columns)
    summary["export_status"] = exported.export_status
    summary["export_source"] = exported.export_source
    if exported.cache_path:
        summary["cache_path"] = exported.cache_path

    filter_applied = filter_requested
    filter_reasons: list[str] = []
    if process_contains:
        summary["process_filter_contains"] = process_contains
        process_filter_applied = "process" in columns
        summary["process_filter_applied"] = process_filter_applied
        if not process_filter_applied:
            filter_applied = False
            summary["process_filter_reason"] = "Table has no process column."
            filter_reasons.append(summary["process_filter_reason"])
    if pid_equals is not None:
        summary["pid_filter_equals"] = pid_equals
        pid_filter_applied = "pid" in columns
        summary["pid_filter_applied"] = pid_filter_applied
        if not pid_filter_applied:
            filter_applied = False
            summary["pid_filter_reason"] = "Table has no pid column."
            filter_reasons.append(summary["pid_filter_reason"])

    if filter_requested:
        summary["filter_applied"] = filter_applied
        if filter_applied and filtered_accumulator is not None:
            summary["filter_match_count"] = filter_match_count
            if process_contains:
                summary["process_filter_match_count"] = filter_match_count
            if pid_equals is not None:
                summary["pid_filter_match_count"] = filter_match_count
            merge_filtered_summary(summary, filtered_accumulator.render(exported.schema, columns))
        elif filter_reasons:
            summary["filter_reason"] = "; ".join(filter_reasons)

    return summary


def decode_nskeyed_archive(data: bytes) -> Any:
    archive = plistlib.loads(data)
    objects = archive.get("$objects", [])

    def decode(value: Any) -> Any:
        if isinstance(value, plistlib.UID):
            return decode(objects[value.data])
        if isinstance(value, list):
            return [decode(item) for item in value]
        if isinstance(value, dict):
            return {key: decode(item) for key, item in value.items()}
        return value

    top = archive.get("$top", {})
    if "root" in top:
        return decode(top["root"])
    return decode(top)


def materialize_ns_object(value: Any) -> Any:
    if isinstance(value, list):
        return [materialize_ns_object(item) for item in value]
    if isinstance(value, dict):
        if "NS.keys" in value and "NS.objects" in value:
            return {
                str(key): materialize_ns_object(obj)
                for key, obj in zip(value["NS.keys"], value["NS.objects"], strict=False)
            }
        if "NS.objects" in value and len({key for key in value if key != "$class"}) == 1:
            return [materialize_ns_object(item) for item in value["NS.objects"]]
        if "$0" in value and len(value) <= 2:
            return materialize_ns_object(value["$0"])
        return {
            str(key): materialize_ns_object(obj)
            for key, obj in value.items()
            if key != "$class"
        }
    return value


def find_store_dir_for_schema(trace: Path, schema_name: str) -> Path | None:
    for schema_path in sorted(trace.glob("corespace/run*/core/stores/indexed-store-*/schema.xml")):
        candidate_name, _columns = parse_store_schema(schema_path)
        if candidate_name == schema_name:
            return schema_path.parent
    return None


def read_direct_store_descriptor(store_dir: Path) -> dict[str, Any]:
    descriptor_path = store_dir / "bulkstore_descriptor"
    decoded = decode_nskeyed_archive(load_zlib_or_raw(descriptor_path))
    materialized = materialize_ns_object(decoded)
    if not isinstance(materialized, dict):
        raise RuntimeError(f"Unexpected descriptor payload for {store_dir}")
    return materialized


def read_direct_store_spec(store_dir: Path) -> dict[str, Any]:
    spec_path = store_dir / "spec.plist"
    decoded = decode_nskeyed_archive(load_zlib_or_raw(spec_path))
    materialized = materialize_ns_object(decoded)
    if not isinstance(materialized, dict):
        raise RuntimeError(f"Unexpected spec payload for {store_dir}")
    return materialized


def read_direct_store_row_count(descriptor: dict[str, Any]) -> int:
    props = descriptor.get("_props") or {}
    row_candidates = [value for value in [props.get("next_event_id"), props.get("mono_event_id")] if isinstance(value, int)]
    return max(row_candidates) if row_candidates else 0


def compute_direct_field_layout(descriptor: dict[str, Any]) -> dict[str, tuple[int, int]]:
    fields = descriptor.get("_fields")
    if not isinstance(fields, list):
        raise RuntimeError("Descriptor is missing _fields")

    offset = 0
    layout: dict[str, tuple[int, int]] = {}
    for item in fields:
        if not isinstance(item, dict):
            raise RuntimeError("Unexpected field descriptor entry")
        name = item.get("_name")
        field_type = item.get("_type")
        if not isinstance(name, str) or not isinstance(field_type, int):
            raise RuntimeError("Field descriptor is missing _name/_type")
        field_size = DIRECT_STORE_FIELD_SIZES.get(field_type)
        if field_size is None:
            raise RuntimeError(f"Unsupported direct-store field type {field_type} for {name}")
        layout[name] = (offset, field_size)
        offset += field_size

    expected_size = descriptor.get("_maxEventSize")
    if isinstance(expected_size, int) and offset != expected_size:
        raise RuntimeError(
            f"Computed field layout size {offset} does not match descriptor size {expected_size}"
        )
    return layout


def read_u32(data: bytes, offset: int) -> int:
    return struct.unpack_from("<I", data, offset)[0]


def read_u64(data: bytes, offset: int) -> int:
    return struct.unpack_from("<Q", data, offset)[0]


def normalize_optional_u64(value: int) -> int | None:
    if value == 0xFFFFFFFFFFFFFFFF:
        return None
    return value


def candidate_direct_store_offsets(bulkstore: bytes) -> list[int]:
    candidates: list[int] = []
    if len(bulkstore) >= 16:
        candidates.append(read_u32(bulkstore, 12))
    candidates.extend([4096, 0])
    return [int(item) for item in dedupe_preserving_order(str(item) for item in candidates)]


def detect_direct_store_data_offset_from_predicate(
    bulkstore: bytes,
    row_count: int,
    record_size: int,
    predicate: Any,
    schema_name: str,
) -> int:
    for candidate in candidate_direct_store_offsets(bulkstore):
        if predicate(bulkstore, candidate, row_count, record_size):
            return candidate

    scan_limit = min(len(bulkstore), 16384)
    for offset in range(0, scan_limit, 8):
        if predicate(bulkstore, offset, row_count, record_size):
            return offset

    raise RuntimeError(f"Unable to detect direct-store data offset for {schema_name}")


def is_plausible_metal_current_allocated_record_block(
    bulkstore: bytes,
    data_offset: int,
    row_count: int,
    record_size: int,
    layout: dict[str, tuple[int, int]],
) -> bool:
    if data_offset < 0 or data_offset + row_count * record_size > len(bulkstore):
        return False

    topology_offset, topology_size = layout.get("topology", (-1, -1))
    end_offset, _end_size = layout.get("end", (-1, -1))
    current_offset, _current_size = layout.get("current-allocated-size", (-1, -1))
    if topology_offset < 0 or topology_size != 24 or end_offset < 0 or current_offset < 0:
        return False

    previous_start = None
    current_values: list[int] = []
    for index in range(min(6, row_count)):
        record = bulkstore[data_offset + index * record_size : data_offset + (index + 1) * record_size]
        start_ns = read_u64(record, topology_offset)
        duration_ns = read_u64(record, topology_offset + 8)
        end_ns = read_u64(record, end_offset)
        current_size = read_u64(record, current_offset)
        if end_ns != start_ns + duration_ns:
            return False
        if current_size <= 0:
            return False
        current_values.append(current_size)
        if previous_start is not None and start_ns < previous_start:
            return False
        previous_start = start_ns

    return len(set(current_values)) <= 2


def detect_direct_store_data_offset(
    bulkstore: bytes,
    row_count: int,
    record_size: int,
    layout: dict[str, tuple[int, int]],
) -> int:
    return detect_direct_store_data_offset_from_predicate(
        bulkstore,
        row_count,
        record_size,
        lambda data, offset, rows, size: is_plausible_metal_current_allocated_record_block(
            data,
            offset,
            rows,
            size,
            layout,
        ),
        "metal-current-allocated-size",
    )


def render_direct_store_empty_summary(
    schema_name: str,
    columns: list[str],
    store_dir: Path,
    record_size: int,
    bulkstore: bytes,
) -> dict[str, Any]:
    return {
        "schema": schema_name,
        "present": True,
        "columns": columns,
        "row_count": 0,
        "has_process_column": "process" in columns,
        "has_thread_column": "thread" in columns,
        "direct_store_fallback": True,
        "export_source": "direct-store",
        "export_status": "fallback-direct-store-empty",
        "direct_store_path": str(store_dir),
        "direct_store_record_size_bytes": record_size,
        "direct_store_bulkstore_bytes": len(bulkstore),
        "note": "Direct indexed-store fallback found a header-only store with zero rows.",
    }


def summarize_metal_current_allocated_size_from_store(
    trace: Path,
    process_contains: str | None,
    pid_equals: int | None,
) -> dict[str, Any] | None:
    store_dir = find_store_dir_for_schema(trace, "metal-current-allocated-size")
    if store_dir is None:
        return None

    schema_name, columns = parse_store_schema(store_dir / "schema.xml")
    descriptor = read_direct_store_descriptor(store_dir)
    spec = read_direct_store_spec(store_dir)
    layout = compute_direct_field_layout(descriptor)

    record_size = descriptor.get("_maxEventSize")
    if not isinstance(record_size, int):
        raise RuntimeError("Descriptor is missing _maxEventSize")
    row_count = read_direct_store_row_count(descriptor)

    bulkstore = load_zlib_or_raw(store_dir / "bulkstore")
    if row_count <= 0:
        summary = render_direct_store_empty_summary(schema_name, columns, store_dir, record_size, bulkstore)
        if process_contains:
            summary["process_filter_contains"] = process_contains
            summary["process_filter_applied"] = False
            summary["process_filter_reason"] = "Direct-store fallback found zero rows for this schema."
        if pid_equals is not None:
            summary["pid_filter_equals"] = pid_equals
            summary["pid_filter_applied"] = False
            summary["pid_filter_reason"] = "Direct-store fallback found zero rows for this schema."
        return summary

    data_offset = detect_direct_store_data_offset(bulkstore, row_count, record_size, layout)

    topology_offset, _ = layout["topology"]
    end_offset, _ = layout["end"]
    process_offset, _ = layout["process"]
    current_offset, _ = layout["current-allocated-size"]
    label_offset, _ = layout["label"]
    track_offset, _ = layout["track-name"]
    color_offset, _ = layout["color"]

    start_values: list[int] = []
    duration_values: list[int] = []
    end_values: list[int] = []
    current_values: list[int] = []
    topology_extra_values: list[int] = []
    process_refs: Counter[str] = Counter()
    label_refs: Counter[str] = Counter()
    track_refs: Counter[str] = Counter()
    color_refs: Counter[str] = Counter()
    topology_end_mismatch_count = 0

    for index in range(row_count):
        record = bulkstore[data_offset + index * record_size : data_offset + (index + 1) * record_size]
        if len(record) != record_size:
            raise RuntimeError("Unexpected truncated record while parsing direct store")
        start_ns = read_u64(record, topology_offset)
        duration_ns = read_u64(record, topology_offset + 8)
        topology_extra = read_u64(record, topology_offset + 16)
        end_ns = read_u64(record, end_offset)
        if end_ns != start_ns + duration_ns:
            topology_end_mismatch_count += 1
        start_values.append(start_ns)
        duration_values.append(duration_ns)
        end_values.append(end_ns)
        topology_extra_values.append(topology_extra)
        current_values.append(read_u64(record, current_offset))
        process_refs[str(read_u32(record, process_offset))] += 1
        label_refs[str(read_u32(record, label_offset))] += 1
        track_refs[str(read_u32(record, track_offset))] += 1
        color_refs[str(read_u32(record, color_offset))] += 1

    first_ts = start_values[0] if start_values else None
    last_ts = start_values[-1] if start_values else None
    duration_secs = None
    approx_hz = None
    if first_ts is not None and last_ts is not None and last_ts > first_ts and len(start_values) > 1:
        duration_secs = (last_ts - first_ts) / 1_000_000_000.0
        approx_hz = (len(start_values) - 1) / duration_secs

    target_pid_mode = None
    mergeable_keys = spec.get("mergeableAttributeKeys")
    attributes = spec.get("attributes")
    if isinstance(attributes, dict):
        target_pid_mode = attributes.get("target-pid")

    summary: dict[str, Any] = {
        "schema": schema_name,
        "present": True,
        "columns": columns,
        "row_count": row_count,
        "has_process_column": True,
        "has_thread_column": False,
        "timestamp_first_ns": first_ts,
        "timestamp_last_ns": last_ts,
        "duration_secs": duration_secs,
        "approx_hz": approx_hz,
        "current_allocated_size_bytes_first": current_values[0],
        "current_allocated_size_bytes_last": current_values[-1],
        "current_allocated_size_bytes_min": min(current_values),
        "current_allocated_size_bytes_max": max(current_values),
        "current_allocated_size_unique_values_head": sorted(set(current_values))[:12],
        "direct_store_fallback": True,
        "export_source": "direct-store",
        "export_status": "fallback-direct-store",
        "direct_store_path": str(store_dir),
        "direct_store_record_size_bytes": record_size,
        "direct_store_data_offset_bytes": data_offset,
        "direct_store_process_ref_ids_head": dict(process_refs.most_common(12)),
        "direct_store_label_ref_ids_head": dict(label_refs.most_common(12)),
        "direct_store_track_name_ref_ids_head": dict(track_refs.most_common(12)),
        "direct_store_color_ids_head": dict(color_refs.most_common(12)),
        "direct_store_duration_ns_first": duration_values[0],
        "direct_store_duration_ns_last": duration_values[-1],
        "direct_store_duration_ns_max": max(duration_values),
        "direct_store_end_first_ns": end_values[0],
        "direct_store_end_last_ns": end_values[-1],
        "direct_store_topology_extra_unique_values_head": sorted(set(topology_extra_values))[:12],
        "direct_store_end_mismatch_count": topology_end_mismatch_count,
        "note": "Direct indexed-store fallback parsed numeric fields from bulkstore because xctrace export timed out on this schema.",
    }
    if isinstance(mergeable_keys, list):
        summary["direct_store_mergeable_attribute_keys"] = mergeable_keys
    if target_pid_mode is not None:
        summary["direct_store_target_pid_mode"] = target_pid_mode

    if process_contains:
        summary["process_filter_contains"] = process_contains
        summary["process_filter_applied"] = False
        summary["process_filter_reason"] = (
            "Direct-store fallback does not resolve process names; inference only."
        )
        if target_pid_mode == "SINGLE":
            summary["process_filter_inference"] = (
                "Store spec marks target-pid as SINGLE, so rows likely belong to one target process."
            )
    if pid_equals is not None:
        summary["pid_filter_equals"] = pid_equals
        summary["pid_filter_applied"] = False
        summary["pid_filter_reason"] = "This schema does not expose a numeric pid field in the direct-store fallback."

    return summary


def is_plausible_virtual_memory_record_block(
    bulkstore: bytes,
    data_offset: int,
    row_count: int,
    record_size: int,
    layout: dict[str, tuple[int, int]],
) -> bool:
    if data_offset < 0 or data_offset + row_count * record_size > len(bulkstore):
        return False

    topology_offset, topology_size = layout.get("topology", (-1, -1))
    process_offset, process_size = layout.get("process", (-1, -1))
    address_offset, address_size = layout.get("address", (-1, -1))
    size_offset, size_size = layout.get("size", (-1, -1))
    backtrace_offset, backtrace_size = layout.get("backtrace", (-1, -1))
    if (
        topology_offset < 0
        or topology_size != 24
        or process_offset < 0
        or process_size != 4
        or address_offset < 0
        or address_size != 8
        or size_offset < 0
        or size_size != 8
        or backtrace_offset < 0
        or backtrace_size != 4
    ):
        return False

    previous_start = None
    for index in range(min(6, row_count)):
        record = bulkstore[data_offset + index * record_size : data_offset + (index + 1) * record_size]
        start_ns = read_u64(record, topology_offset)
        duration_ns = read_u64(record, topology_offset + 8)
        operation_ref = read_u32(record, topology_offset + 16)
        thread_ref = read_u32(record, topology_offset + 20)
        process_ref = read_u32(record, process_offset)
        address = read_u64(record, address_offset)
        size_bytes = read_u64(record, size_offset)
        _backtrace_ref = read_u32(record, backtrace_offset)
        if start_ns <= 0:
            return False
        if previous_start is not None and start_ns < previous_start:
            return False
        if duration_ns < 0 or operation_ref == 0 or thread_ref == 0 or process_ref == 0:
            return False
        if address == 0 or address % 4096 != 0:
            return False
        if size_bytes == 0 or size_bytes > (1 << 30):
            return False
        previous_start = start_ns

    return True


def summarize_virtual_memory_from_store(
    trace: Path,
    process_contains: str | None,
    pid_equals: int | None,
) -> dict[str, Any] | None:
    store_dir = find_store_dir_for_schema(trace, "virtual-memory")
    if store_dir is None:
        return None

    schema_name, columns = parse_store_schema(store_dir / "schema.xml")
    descriptor = read_direct_store_descriptor(store_dir)
    spec = read_direct_store_spec(store_dir)
    layout = compute_direct_field_layout(descriptor)

    record_size = descriptor.get("_maxEventSize")
    if not isinstance(record_size, int):
        raise RuntimeError("Descriptor is missing _maxEventSize")
    row_count = read_direct_store_row_count(descriptor)
    bulkstore = load_zlib_or_raw(store_dir / "bulkstore")
    if row_count <= 0:
        summary = render_direct_store_empty_summary(schema_name, columns, store_dir, record_size, bulkstore)
        if process_contains:
            summary["process_filter_contains"] = process_contains
            summary["process_filter_applied"] = False
            summary["process_filter_reason"] = "Direct-store fallback found zero rows for this schema."
        if pid_equals is not None:
            summary["pid_filter_equals"] = pid_equals
            summary["pid_filter_applied"] = False
            summary["pid_filter_reason"] = "Table has no pid column."
        return summary

    data_offset = detect_direct_store_data_offset_from_predicate(
        bulkstore,
        row_count,
        record_size,
        lambda data, offset, rows, size: is_plausible_virtual_memory_record_block(
            data,
            offset,
            rows,
            size,
            layout,
        ),
        schema_name,
    )

    topology_offset, _ = layout["topology"]
    process_offset, _ = layout["process"]
    cputime_offset, _ = layout["cputime"]
    waittime_offset, _ = layout["waittime"]
    address_offset, _ = layout["address"]
    size_offset, _ = layout["size"]
    backtrace_offset, _ = layout["backtrace"]

    start_values: list[int] = []
    duration_values: list[int] = []
    cputime_values: list[int] = []
    waittime_values: list[int] = []
    size_values: list[int] = []
    process_refs: Counter[str] = Counter()
    thread_refs: Counter[str] = Counter()
    operation_refs: Counter[str] = Counter()
    backtrace_refs: Counter[str] = Counter()
    address_alignment_counts: Counter[str] = Counter()

    for index in range(row_count):
        record = bulkstore[data_offset + index * record_size : data_offset + (index + 1) * record_size]
        if len(record) != record_size:
            raise RuntimeError("Unexpected truncated record while parsing direct store")
        start_ns = read_u64(record, topology_offset)
        duration_ns = read_u64(record, topology_offset + 8)
        operation_ref = read_u32(record, topology_offset + 16)
        thread_ref = read_u32(record, topology_offset + 20)
        address = read_u64(record, address_offset)
        size_bytes = read_u64(record, size_offset)
        start_values.append(start_ns)
        duration_values.append(duration_ns)
        cputime_values.append(read_u64(record, cputime_offset))
        waittime_values.append(read_u64(record, waittime_offset))
        size_values.append(size_bytes)
        process_refs[str(read_u32(record, process_offset))] += 1
        thread_refs[str(thread_ref)] += 1
        operation_refs[str(operation_ref)] += 1
        backtrace_refs[str(read_u32(record, backtrace_offset))] += 1
        address_alignment_counts[str(address % 4096)] += 1

    first_ts = start_values[0] if start_values else None
    last_ts = start_values[-1] if start_values else None
    duration_secs = None
    approx_hz = None
    if first_ts is not None and last_ts is not None and last_ts > first_ts and len(start_values) > 1:
        duration_secs = (last_ts - first_ts) / 1_000_000_000.0
        approx_hz = (len(start_values) - 1) / duration_secs

    target_pid_mode = None
    attributes = spec.get("attributes")
    if isinstance(attributes, dict):
        target_pid_mode = attributes.get("target-pid")

    summary: dict[str, Any] = {
        "schema": schema_name,
        "present": True,
        "columns": columns,
        "row_count": row_count,
        "has_process_column": True,
        "has_thread_column": True,
        "timestamp_first_ns": first_ts,
        "timestamp_last_ns": last_ts,
        "duration_secs": duration_secs,
        "approx_hz": approx_hz,
        "size_bytes_sum": sum(size_values),
        "size_bytes_max": max(size_values),
        "size_bytes_min": min(size_values),
        "cputime_ns_sum": sum(cputime_values),
        "cputime_ns_max": max(cputime_values),
        "waittime_ns_sum": sum(waittime_values),
        "waittime_ns_max": max(waittime_values),
        "direct_store_fallback": True,
        "export_source": "direct-store",
        "export_status": "fallback-direct-store",
        "direct_store_path": str(store_dir),
        "direct_store_record_size_bytes": record_size,
        "direct_store_data_offset_bytes": data_offset,
        "direct_store_process_ref_ids_head": dict(process_refs.most_common(12)),
        "direct_store_thread_ref_ids_head": dict(thread_refs.most_common(12)),
        "direct_store_operation_ref_ids_head": dict(operation_refs.most_common(12)),
        "direct_store_backtrace_ref_ids_head": dict(backtrace_refs.most_common(12)),
        "direct_store_address_alignment_mod_4096": dict(address_alignment_counts.most_common(4)),
        "note": "Event stream: virtual-memory rows are VM operations, not a direct current-footprint snapshot.",
    }
    if target_pid_mode is not None:
        summary["direct_store_target_pid_mode"] = target_pid_mode
    if process_contains:
        summary["process_filter_contains"] = process_contains
        summary["process_filter_applied"] = False
        summary["process_filter_reason"] = "Direct-store fallback does not resolve per-row process names for this schema."
        if target_pid_mode == "SINGLE":
            summary["process_filter_inference"] = (
                "Store spec marks target-pid as SINGLE, so rows likely belong to one target process."
            )
    if pid_equals is not None:
        summary["pid_filter_equals"] = pid_equals
        summary["pid_filter_applied"] = False
        summary["pid_filter_reason"] = "Table has no pid column."

    return summary


def is_plausible_metal_io_surface_access_record_block(
    bulkstore: bytes,
    data_offset: int,
    row_count: int,
    record_size: int,
    layout: dict[str, tuple[int, int]],
) -> bool:
    if data_offset < 0 or data_offset + row_count * record_size > len(bulkstore):
        return False

    topology_offset, topology_size = layout.get("topology", (-1, -1))
    surface_offset, surface_size = layout.get("surface-id", (-1, -1))
    pixel_format_offset, pixel_format_size = layout.get("pixel-format", (-1, -1))
    width_offset, width_size = layout.get("width", (-1, -1))
    height_offset, height_size = layout.get("height", (-1, -1))
    access_offset, access_size = layout.get("access-type", (-1, -1))
    pid_offset, pid_size = layout.get("pid", (-1, -1))
    process_offset, process_size = layout.get("process", (-1, -1))
    if (
        topology_offset < 0
        or topology_size != 16
        or surface_offset < 0
        or surface_size != 8
        or pixel_format_offset < 0
        or pixel_format_size != 4
        or width_offset < 0
        or width_size != 4
        or height_offset < 0
        or height_size != 4
        or access_offset < 0
        or access_size != 4
        or pid_offset < 0
        or pid_size != 8
        or process_offset < 0
        or process_size != 4
    ):
        return False

    previous_start = None
    for index in range(min(6, row_count)):
        record = bulkstore[data_offset + index * record_size : data_offset + (index + 1) * record_size]
        start_ns = read_u64(record, topology_offset)
        surface_id = read_u64(record, surface_offset)
        pixel_format = read_u32(record, pixel_format_offset)
        width = read_u32(record, width_offset)
        height = read_u32(record, height_offset)
        access_type = read_u32(record, access_offset)
        pid = read_u64(record, pid_offset)
        _process_ref = read_u32(record, process_offset)
        if start_ns <= 0:
            return False
        if previous_start is not None and start_ns < previous_start:
            return False
        if surface_id == 0 or pixel_format == 0:
            return False
        if width == 0 or height == 0 or width > 65536 or height > 65536:
            return False
        if access_type > 16 or pid == 0:
            return False
        previous_start = start_ns

    return True


def summarize_metal_io_surface_access_from_store(
    trace: Path,
    process_contains: str | None,
    pid_equals: int | None,
) -> dict[str, Any] | None:
    store_dir = find_store_dir_for_schema(trace, "metal-io-surface-access")
    if store_dir is None:
        return None

    schema_name, columns = parse_store_schema(store_dir / "schema.xml")
    descriptor = read_direct_store_descriptor(store_dir)
    layout = compute_direct_field_layout(descriptor)
    record_size = descriptor.get("_maxEventSize")
    if not isinstance(record_size, int):
        raise RuntimeError("Descriptor is missing _maxEventSize")
    row_count = read_direct_store_row_count(descriptor)
    bulkstore = load_zlib_or_raw(store_dir / "bulkstore")
    if row_count <= 0:
        summary = render_direct_store_empty_summary(schema_name, columns, store_dir, record_size, bulkstore)
        if process_contains:
            summary["process_filter_contains"] = process_contains
            summary["process_filter_applied"] = False
            summary["process_filter_reason"] = "Direct-store fallback found zero rows for this schema."
        if pid_equals is not None:
            summary["pid_filter_equals"] = pid_equals
            summary["pid_filter_applied"] = True
            summary["pid_filter_match_count"] = 0
        return summary

    data_offset = detect_direct_store_data_offset_from_predicate(
        bulkstore,
        row_count,
        record_size,
        lambda data, offset, rows, size: is_plausible_metal_io_surface_access_record_block(
            data,
            offset,
            rows,
            size,
            layout,
        ),
        schema_name,
    )

    topology_offset, _ = layout["topology"]
    cmdbuffer_offset, _ = layout["cmdbuffer-id"]
    surface_offset, _ = layout["surface-id"]
    pixel_format_offset, _ = layout["pixel-format"]
    width_offset, _ = layout["width"]
    height_offset, _ = layout["height"]
    access_offset, _ = layout["access-type"]
    pid_offset, _ = layout["pid"]
    process_offset, _ = layout["process"]

    timestamp_values: list[int] = []
    topology_extra_values: list[int] = []
    surface_ids: Counter[str] = Counter()
    surface_extents: Counter[str] = Counter()
    pixel_format_codes: Counter[str] = Counter()
    access_types: Counter[str] = Counter()
    pid_counts: Counter[str] = Counter()
    process_refs: Counter[str] = Counter()
    command_buffers: Counter[str] = Counter()

    filtered_timestamp_values: list[int] = []
    filtered_surface_ids: Counter[str] = Counter()
    filtered_surface_extents: Counter[str] = Counter()
    filtered_pixel_format_codes: Counter[str] = Counter()
    filtered_access_types: Counter[str] = Counter()
    filtered_pid_counts: Counter[str] = Counter()
    filtered_process_refs: Counter[str] = Counter()
    filtered_command_buffers: Counter[str] = Counter()
    pid_filter_match_count = 0

    for index in range(row_count):
        record = bulkstore[data_offset + index * record_size : data_offset + (index + 1) * record_size]
        if len(record) != record_size:
            raise RuntimeError("Unexpected truncated record while parsing direct store")
        timestamp_ns = read_u64(record, topology_offset)
        topology_extra = read_u64(record, topology_offset + 8)
        cmdbuffer_id = read_u64(record, cmdbuffer_offset)
        surface_id = read_u64(record, surface_offset)
        pixel_format = read_u32(record, pixel_format_offset)
        width = read_u32(record, width_offset)
        height = read_u32(record, height_offset)
        access_type = read_u32(record, access_offset)
        pid = read_u64(record, pid_offset)
        process_ref = read_u32(record, process_offset)

        extent_key = f"{width}x{height} code:{pixel_format}"
        timestamp_values.append(timestamp_ns)
        topology_extra_values.append(topology_extra)
        surface_ids[str(surface_id)] += 1
        surface_extents[extent_key] += 1
        pixel_format_codes[str(pixel_format)] += 1
        access_types[str(access_type)] += 1
        pid_counts[str(pid)] += 1
        process_refs[str(process_ref)] += 1
        command_buffers[str(cmdbuffer_id)] += 1

        if pid_equals is not None and pid == pid_equals:
            filtered_timestamp_values.append(timestamp_ns)
            filtered_surface_ids[str(surface_id)] += 1
            filtered_surface_extents[extent_key] += 1
            filtered_pixel_format_codes[str(pixel_format)] += 1
            filtered_access_types[str(access_type)] += 1
            filtered_pid_counts[str(pid)] += 1
            filtered_process_refs[str(process_ref)] += 1
            filtered_command_buffers[str(cmdbuffer_id)] += 1
            pid_filter_match_count += 1

    first_ts = timestamp_values[0] if timestamp_values else None
    last_ts = timestamp_values[-1] if timestamp_values else None
    duration_secs = None
    approx_hz = None
    if first_ts is not None and last_ts is not None and last_ts > first_ts and len(timestamp_values) > 1:
        duration_secs = (last_ts - first_ts) / 1_000_000_000.0
        approx_hz = (len(timestamp_values) - 1) / duration_secs

    summary: dict[str, Any] = {
        "schema": schema_name,
        "present": True,
        "columns": columns,
        "row_count": row_count,
        "has_process_column": True,
        "has_thread_column": False,
        "timestamp_first_ns": first_ts,
        "timestamp_last_ns": last_ts,
        "duration_secs": duration_secs,
        "approx_hz": approx_hz,
        "surface_ids_count": len(surface_ids),
        "surface_ids_head": dict(surface_ids.most_common(12)),
        "surface_extents_head": dict(surface_extents.most_common(12)),
        "pixel_format_codes_head": dict(pixel_format_codes.most_common(12)),
        "access_types_head": dict(access_types.most_common(12)),
        "pid_counts": dict(pid_counts.most_common(12)),
        "command_buffers_count": len(command_buffers),
        "direct_store_fallback": True,
        "export_source": "direct-store",
        "export_status": "fallback-direct-store",
        "direct_store_path": str(store_dir),
        "direct_store_record_size_bytes": record_size,
        "direct_store_data_offset_bytes": data_offset,
        "direct_store_process_ref_ids_head": dict(process_refs.most_common(12)),
        "direct_store_topology_extra_unique_values_head": sorted(set(topology_extra_values))[:12],
        "note": "Direct indexed-store fallback parsed numeric fields from bulkstore because xctrace export timed out on this schema.",
    }
    if process_contains:
        summary["process_filter_contains"] = process_contains
        summary["process_filter_applied"] = False
        summary["process_filter_reason"] = "Direct-store fallback does not resolve per-row process names for this schema."
    if pid_equals is not None:
        summary["pid_filter_equals"] = pid_equals
        summary["pid_filter_applied"] = True
        summary["pid_filter_match_count"] = pid_filter_match_count
        summary["filter_applied"] = True
        summary["filter_match_count"] = pid_filter_match_count
        if filtered_timestamp_values:
            filtered_first_ts = filtered_timestamp_values[0]
            filtered_last_ts = filtered_timestamp_values[-1]
            filtered_duration_secs = None
            filtered_approx_hz = None
            if (
                filtered_last_ts > filtered_first_ts
                and len(filtered_timestamp_values) > 1
            ):
                filtered_duration_secs = (filtered_last_ts - filtered_first_ts) / 1_000_000_000.0
                filtered_approx_hz = (len(filtered_timestamp_values) - 1) / filtered_duration_secs
            merge_filtered_summary(
                summary,
                {
                    "schema": schema_name,
                    "present": True,
                    "columns": columns,
                    "row_count": pid_filter_match_count,
                    "has_process_column": True,
                    "has_thread_column": False,
                    "timestamp_first_ns": filtered_first_ts,
                    "timestamp_last_ns": filtered_last_ts,
                    "duration_secs": filtered_duration_secs,
                    "approx_hz": filtered_approx_hz,
                    "surface_ids_count": len(filtered_surface_ids),
                    "surface_ids_head": dict(filtered_surface_ids.most_common(12)),
                    "surface_extents_head": dict(filtered_surface_extents.most_common(12)),
                    "pixel_format_codes_head": dict(filtered_pixel_format_codes.most_common(12)),
                    "access_types_head": dict(filtered_access_types.most_common(12)),
                    "pid_counts": dict(filtered_pid_counts.most_common(12)),
                    "command_buffers_count": len(filtered_command_buffers),
                    "direct_store_process_ref_ids_head": dict(filtered_process_refs.most_common(12)),
                },
            )
        else:
            summary["filtered_row_count"] = 0

    return summary


def is_plausible_metal_resource_allocations_record_block(
    bulkstore: bytes,
    data_offset: int,
    row_count: int,
    record_size: int,
    layout: dict[str, tuple[int, int]],
) -> bool:
    if data_offset < 0 or data_offset + row_count * record_size > len(bulkstore):
        return False

    topology_offset, topology_size = layout.get("topology", (-1, -1))
    duration_offset, duration_size = layout.get("duration", (-1, -1))
    process_offset, process_size = layout.get("process", (-1, -1))
    thread_offset, thread_size = layout.get("thread", (-1, -1))
    resource_id_offset, resource_id_size = layout.get("resource-id", (-1, -1))
    resource_size_offset, resource_size_size = layout.get("resource-size", (-1, -1))
    event_type_offset, event_type_size = layout.get("event-type", (-1, -1))
    resource_type_offset, resource_type_size = layout.get("resource-type", (-1, -1))
    if (
        topology_offset < 0
        or topology_size != 16
        or duration_offset < 0
        or duration_size != 8
        or process_offset < 0
        or process_size != 4
        or thread_offset < 0
        or thread_size != 4
        or resource_id_offset < 0
        or resource_id_size != 8
        or resource_size_offset < 0
        or resource_size_size != 8
        or event_type_offset < 0
        or event_type_size != 4
        or resource_type_offset < 0
        or resource_type_size != 4
    ):
        return False

    for index in range(min(6, row_count)):
        record = bulkstore[data_offset + index * record_size : data_offset + (index + 1) * record_size]
        start_ns = read_u64(record, topology_offset)
        duration_ns = read_u64(record, duration_offset)
        process_ref = read_u32(record, process_offset)
        thread_ref = read_u32(record, thread_offset)
        resource_id = read_u64(record, resource_id_offset)
        resource_size = read_u64(record, resource_size_offset)
        event_type_ref = read_u32(record, event_type_offset)
        resource_type_ref = read_u32(record, resource_type_offset)
        if start_ns <= 0:
            return False
        if duration_ns > (1 << 40):
            return False
        if process_ref == 0 or thread_ref == 0 or resource_id == 0:
            return False
        if resource_size > (1 << 36):
            return False
        if event_type_ref == 0 or resource_type_ref == 0:
            return False

    return True


def summarize_metal_resource_allocations_from_store(
    trace: Path,
    process_contains: str | None,
    pid_equals: int | None,
) -> dict[str, Any] | None:
    store_dir = find_store_dir_for_schema(trace, "metal-resource-allocations")
    if store_dir is None:
        return None

    schema_name, columns = parse_store_schema(store_dir / "schema.xml")
    descriptor = read_direct_store_descriptor(store_dir)
    spec = read_direct_store_spec(store_dir)
    layout = compute_direct_field_layout(descriptor)
    record_size = descriptor.get("_maxEventSize")
    if not isinstance(record_size, int):
        raise RuntimeError("Descriptor is missing _maxEventSize")
    row_count = read_direct_store_row_count(descriptor)
    bulkstore = load_zlib_or_raw(store_dir / "bulkstore")
    if row_count <= 0:
        summary = render_direct_store_empty_summary(schema_name, columns, store_dir, record_size, bulkstore)
        if process_contains:
            summary["process_filter_contains"] = process_contains
            summary["process_filter_applied"] = False
            summary["process_filter_reason"] = "Direct-store fallback found zero rows for this schema."
        if pid_equals is not None:
            summary["pid_filter_equals"] = pid_equals
            summary["pid_filter_applied"] = False
            summary["pid_filter_reason"] = "Table has no pid column."
        return summary

    data_offset = detect_direct_store_data_offset_from_predicate(
        bulkstore,
        row_count,
        record_size,
        lambda data, offset, rows, size: is_plausible_metal_resource_allocations_record_block(
            data,
            offset,
            rows,
            size,
            layout,
        ),
        schema_name,
    )

    topology_offset, _ = layout["topology"]
    duration_offset, _ = layout["duration"]
    process_offset, _ = layout["process"]
    thread_offset, _ = layout["thread"]
    resource_id_offset, _ = layout["resource-id"]
    parent_resource_id_offset, _ = layout["parent-resource-id"]
    label_offset, _ = layout["label"]
    vidmem_offset, _ = layout["vidmem-bytes"]
    sysmem_offset, _ = layout["sysmem-bytes"]
    resource_size_offset, _ = layout["resource-size"]
    event_label_offset, _ = layout["event-label"]
    gpu_offset, _ = layout["gpu"]
    resource_type_offset, _ = layout["resource-type"]
    event_type_offset, _ = layout["event-type"]
    backtrace_offset, _ = layout["backtrace"]
    event_icon_offset, _ = layout["event-icon"]

    timestamp_values: list[int] = []
    topology_extra_values: list[int] = []
    duration_values: list[int] = []
    vidmem_values: list[int] = []
    sysmem_values: list[int] = []
    resource_size_values: list[int] = []
    resource_ids: Counter[str] = Counter()
    parent_resource_ids: Counter[str] = Counter()
    process_refs: Counter[str] = Counter()
    thread_refs: Counter[str] = Counter()
    label_refs: Counter[str] = Counter()
    event_label_refs: Counter[str] = Counter()
    gpu_refs: Counter[str] = Counter()
    resource_type_refs: Counter[str] = Counter()
    event_type_refs: Counter[str] = Counter()
    backtrace_refs: Counter[str] = Counter()
    event_icon_refs: Counter[str] = Counter()

    for index in range(row_count):
        record = bulkstore[data_offset + index * record_size : data_offset + (index + 1) * record_size]
        if len(record) != record_size:
            raise RuntimeError("Unexpected truncated record while parsing direct store")
        timestamp_values.append(read_u64(record, topology_offset))
        topology_extra_values.append(read_u64(record, topology_offset + 8))
        duration_values.append(read_u64(record, duration_offset))
        vidmem_value = normalize_optional_u64(read_u64(record, vidmem_offset))
        sysmem_value = normalize_optional_u64(read_u64(record, sysmem_offset))
        resource_size_value = normalize_optional_u64(read_u64(record, resource_size_offset))
        if vidmem_value is not None:
            vidmem_values.append(vidmem_value)
        if sysmem_value is not None:
            sysmem_values.append(sysmem_value)
        if resource_size_value is not None:
            resource_size_values.append(resource_size_value)
        resource_ids[str(read_u64(record, resource_id_offset))] += 1
        parent_resource_id = normalize_optional_u64(read_u64(record, parent_resource_id_offset))
        if parent_resource_id is not None:
            parent_resource_ids[str(parent_resource_id)] += 1
        process_refs[str(read_u32(record, process_offset))] += 1
        thread_refs[str(read_u32(record, thread_offset))] += 1
        label_refs[str(read_u32(record, label_offset))] += 1
        event_label_refs[str(read_u32(record, event_label_offset))] += 1
        gpu_refs[str(read_u32(record, gpu_offset))] += 1
        resource_type_refs[str(read_u32(record, resource_type_offset))] += 1
        event_type_refs[str(read_u32(record, event_type_offset))] += 1
        backtrace_refs[str(read_u32(record, backtrace_offset))] += 1
        event_icon_refs[str(read_u32(record, event_icon_offset))] += 1

    first_ts = min(timestamp_values) if timestamp_values else None
    last_ts = max(timestamp_values) if timestamp_values else None
    duration_secs = None
    approx_hz = None
    if first_ts is not None and last_ts is not None and last_ts > first_ts and len(timestamp_values) > 1:
        duration_secs = (last_ts - first_ts) / 1_000_000_000.0
        approx_hz = (len(timestamp_values) - 1) / duration_secs

    target_pid_mode = None
    attributes = spec.get("attributes")
    if isinstance(attributes, dict):
        target_pid_mode = attributes.get("target-pid")

    summary: dict[str, Any] = {
        "schema": schema_name,
        "present": True,
        "columns": columns,
        "row_count": row_count,
        "has_process_column": True,
        "has_thread_column": True,
        "timestamp_first_ns": first_ts,
        "timestamp_last_ns": last_ts,
        "duration_secs": duration_secs,
        "approx_hz": approx_hz,
        "resource_ids_count": len(resource_ids),
        "resource_ids_head": dict(resource_ids.most_common(12)),
        "parent_resource_ids_head": dict(parent_resource_ids.most_common(12)),
        "direct_store_fallback": True,
        "export_source": "direct-store",
        "export_status": "fallback-direct-store",
        "direct_store_path": str(store_dir),
        "direct_store_record_size_bytes": record_size,
        "direct_store_data_offset_bytes": data_offset,
        "direct_store_process_ref_ids_head": dict(process_refs.most_common(12)),
        "direct_store_thread_ref_ids_head": dict(thread_refs.most_common(12)),
        "direct_store_label_ref_ids_head": dict(label_refs.most_common(12)),
        "direct_store_event_label_ref_ids_head": dict(event_label_refs.most_common(12)),
        "direct_store_gpu_ref_ids_head": dict(gpu_refs.most_common(12)),
        "direct_store_resource_type_ref_ids_head": dict(resource_type_refs.most_common(12)),
        "direct_store_event_type_ref_ids_head": dict(event_type_refs.most_common(12)),
        "direct_store_backtrace_ref_ids_head": dict(backtrace_refs.most_common(12)),
        "direct_store_event_icon_ref_ids_head": dict(event_icon_refs.most_common(12)),
        "direct_store_duration_ns_max": max(duration_values),
        "direct_store_duration_ns_sum": sum(duration_values),
        "direct_store_topology_extra_unique_values_head": sorted(set(topology_extra_values))[:12],
        "note": "Event stream: *_bytes_sum values add allocation event payloads and are not equivalent to live residency by themselves.",
    }
    if resource_size_values:
        summary["resource_size_bytes_sum"] = sum(resource_size_values)
        summary["resource_size_bytes_max"] = max(resource_size_values)
        summary["resource_size_bytes_min"] = min(resource_size_values)
    if vidmem_values:
        summary["vidmem_bytes_sum"] = sum(vidmem_values)
        summary["vidmem_bytes_max"] = max(vidmem_values)
    if sysmem_values:
        summary["sysmem_bytes_sum"] = sum(sysmem_values)
        summary["sysmem_bytes_max"] = max(sysmem_values)
    if target_pid_mode is not None:
        summary["direct_store_target_pid_mode"] = target_pid_mode
    if process_contains:
        summary["process_filter_contains"] = process_contains
        summary["process_filter_applied"] = False
        summary["process_filter_reason"] = "Direct-store fallback does not resolve per-row process names for this schema."
        if target_pid_mode == "SINGLE":
            summary["process_filter_inference"] = (
                "Store spec marks target-pid as SINGLE, so rows likely belong to one target process."
            )
    if pid_equals is not None:
        summary["pid_filter_equals"] = pid_equals
        summary["pid_filter_applied"] = False
        summary["pid_filter_reason"] = "Table has no pid column."

    return summary


def summarize_from_indexed_store(
    trace: Path,
    schema_name: str,
    process_contains: str | None,
    pid_equals: int | None,
) -> dict[str, Any] | None:
    if schema_name == "metal-current-allocated-size":
        return summarize_metal_current_allocated_size_from_store(trace, process_contains, pid_equals)
    if schema_name == "metal-io-surface-access":
        return summarize_metal_io_surface_access_from_store(trace, process_contains, pid_equals)
    if schema_name == "metal-resource-allocations":
        return summarize_metal_resource_allocations_from_store(trace, process_contains, pid_equals)
    if schema_name == "virtual-memory":
        return summarize_virtual_memory_from_store(trace, process_contains, pid_equals)
    return None


def summarize_direct_store_metadata(
    trace: Path,
    schema_name: str,
    process_contains: str | None,
    pid_equals: int | None,
) -> dict[str, Any] | None:
    store_dir = find_store_dir_for_schema(trace, schema_name)
    if store_dir is None:
        return None

    schema, columns = parse_store_schema(store_dir / "schema.xml")
    descriptor = read_direct_store_descriptor(store_dir)
    spec = read_direct_store_spec(store_dir)
    props = descriptor.get("_props") or {}
    row_count = read_direct_store_row_count(descriptor)
    bulkstore = load_zlib_or_raw(store_dir / "bulkstore")
    fields = descriptor.get("_fields")
    field_names = None
    if isinstance(fields, list):
        field_names = [item.get("_name") for item in fields if isinstance(item, dict)]

    summary: dict[str, Any] = {
        "schema": schema,
        "present": True,
        "columns": columns,
        "row_count": row_count,
        "has_process_column": "process" in columns,
        "has_thread_column": "thread" in columns,
        "export_source": "direct-store",
        "export_status": "fallback-direct-store-metadata",
        "direct_store_path": str(store_dir),
        "direct_store_bulkstore_bytes": len(bulkstore),
        "direct_store_record_size_bytes": descriptor.get("_maxEventSize"),
        "direct_store_props": props,
        "direct_store_field_names": field_names,
    }

    attributes = spec.get("attributes")
    if isinstance(attributes, dict):
        summary["direct_store_attributes"] = attributes
    if row_count == 0:
        summary["note"] = "Direct-store metadata fallback found a header-only or zero-row store."
    else:
        summary["note"] = "Direct-store metadata fallback recovered store row counts, but no row parser is implemented for this schema yet."
    if process_contains:
        summary["process_filter_contains"] = process_contains
        summary["process_filter_applied"] = False
        summary["process_filter_reason"] = "Metadata fallback does not decode per-row process values."
    if pid_equals is not None:
        summary["pid_filter_equals"] = pid_equals
        summary["pid_filter_applied"] = False
        summary["pid_filter_reason"] = "Metadata fallback does not decode per-row pid values."
    return summary


def main() -> int:
    args = parse_args()
    trace = Path(args.trace)
    result: dict[str, Any]
    if args.list_store_schemas:
        result = {
            "trace": str(trace),
            "store_schemas": list_store_schemas(trace, args.store_schema_filter),
        }
    else:
        timeout_secs = args.export_timeout_secs if args.export_timeout_secs and args.export_timeout_secs > 0 else None
        export_dir = Path(args.export_dir) if args.export_dir else None
        schemas = resolve_schemas(args)
        tables = []
        for schema_name in schemas:
            exported = export_table(
                trace,
                schema_name,
                export_dir=export_dir,
                timeout_secs=timeout_secs,
            )
            if isinstance(exported, ExportFailure):
                fallback = None
                if schema_name in DIRECT_STORE_FALLBACK_SCHEMAS:
                    try:
                        fallback = summarize_from_indexed_store(
                            trace,
                            schema_name,
                            args.process_contains,
                            args.pid_equals,
                        )
                    except Exception as exc:
                        fallback = {
                            "schema": schema_name,
                            "present": False,
                            "export_status": "fallback-error",
                            "export_source": "direct-store",
                            "export_error": str(exc),
                        }
                if fallback is None:
                    try:
                        fallback = summarize_direct_store_metadata(
                            trace,
                            schema_name,
                            args.process_contains,
                            args.pid_equals,
                        )
                    except Exception as exc:
                        fallback = {
                            "schema": schema_name,
                            "present": False,
                            "export_status": "fallback-error",
                            "export_source": "direct-store",
                            "export_error": str(exc),
                        }
                if fallback is not None:
                    fallback["fallback_from_export_status"] = exported.export_status
                    fallback["fallback_from_export_source"] = exported.export_source
                    if exported.export_error:
                        fallback["fallback_from_export_error"] = exported.export_error
                    if exported.cache_path:
                        fallback["fallback_cache_path"] = exported.cache_path
                    if exported.partial_cache_path:
                        fallback["fallback_partial_cache_path"] = exported.partial_cache_path
                    tables.append(fallback)
                    continue

                table: dict[str, Any] = {
                    "schema": schema_name,
                    "present": False,
                    "export_status": exported.export_status,
                    "export_source": exported.export_source,
                }
                if exported.export_error:
                    table["export_error"] = exported.export_error
                if exported.cache_path:
                    table["cache_path"] = exported.cache_path
                if exported.partial_cache_path:
                    table["partial_cache_path"] = exported.partial_cache_path
                tables.append(table)
                continue
            tables.append(summarize_table(exported, args.process_contains, args.pid_equals))
        result = {
            "trace": str(trace),
            "preset": args.preset,
            "requested_schemas": schemas,
            "process_contains": args.process_contains,
            "export_dir": str(export_dir) if export_dir else None,
            "export_timeout_secs": timeout_secs,
            "tables": tables,
        }
    rendered = json.dumps(result, indent=2, sort_keys=True)
    if args.output:
        output = Path(args.output)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(rendered + "\n")
    else:
        print(rendered)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
