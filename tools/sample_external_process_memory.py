#!/usr/bin/env python3
"""Launch an external macOS GUI process, capture one or more footprint/vmmap snapshots, and stop it."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import time
from pathlib import Path
from typing import Any

SIZE_UNITS = {
    "B": 1,
    "K": 1024,
    "M": 1024 ** 2,
    "G": 1024 ** 3,
    "T": 1024 ** 4,
}

try:
    PAGE_BYTES = int(os.sysconf('SC_PAGE_SIZE'))
except (AttributeError, OSError, ValueError):
    PAGE_BYTES = 16 * 1024

FOOTPRINT_VERBOSE_ROW_RE = re.compile(
    r'^\s*(?P<start>[0-9a-fA-F]+|unmapped)\s*-\s*(?P<end>[0-9a-fA-F]+|unmapped)\s+'
    r'\[(?P<object_id>[0-9a-fA-F]+)\]\s+'
    r'(?P<virtual_pages>\d+)\s+'
    r'(?P<dirty_pages>\d+)\s+'
    r'(?P<clean_pages>\d+)\s+'
    r'(?P<reclaimable_pages>\d+)\s+'
    r'(?P<tag>.+?)\s*$'
)

FOOTPRINT_VERBOSE_FOCUS_FAMILIES = [
    'Owned physical footprint (unmapped) (graphics)',
    'Owned physical footprint (unmapped)',
    'IOSurface CAMetalLayer Display Drawable',
    'IOSurface',
    'IOAccelerator (graphics)',
]


CAPTURE_COMMAND_RETRIES = 3
CAPTURE_COMMAND_RETRY_SLEEP_SECS = 0.2


def run_capture_command(
    cmd: list[str],
    *,
    retries: int = CAPTURE_COMMAND_RETRIES,
    retry_sleep_secs: float = CAPTURE_COMMAND_RETRY_SLEEP_SECS,
    **kwargs: Any,
) -> subprocess.CompletedProcess[Any]:
    attempts = max(1, retries)
    last_exc: subprocess.CalledProcessError | OSError | None = None
    for attempt in range(attempts):
        try:
            return subprocess.run(cmd, check=True, **kwargs)
        except (subprocess.CalledProcessError, OSError) as exc:
            last_exc = exc
            if attempt + 1 >= attempts:
                raise
            time.sleep(retry_sleep_secs)
    raise RuntimeError(f"unreachable capture retry loop for command: {cmd!r}") from last_exc


def parse_size_token(token: str) -> int | None:
    token = token.strip().rstrip(',')
    if not token:
        return None
    token = token.replace('(', '').replace(')', '')
    if token in {'-', '--', '0'}:
        return 0
    unit = token[-1].upper()
    if unit in SIZE_UNITS:
        number = token[:-1]
    else:
        unit = 'B'
        number = token
    try:
        value = float(number)
    except ValueError:
        return None
    return int(round(value * SIZE_UNITS[unit]))


def parse_u64_token(token: str) -> int | None:
    token = token.strip().rstrip(',').replace('_', '')
    if not token:
        return None
    try:
        return int(token)
    except ValueError:
        return None


def parse_percent_token(token: str) -> float | None:
    token = token.strip().rstrip('%')
    if not token:
        return None
    try:
        return float(token)
    except ValueError:
        return None


def parse_size_phrase(text: str) -> int | None:
    tokens = text.strip().split()
    if len(tokens) >= 2 and tokens[0] and tokens[0][0].isdigit():
        return parse_size_token(f"{tokens[0]}{tokens[1][0]}")
    if tokens:
        return parse_size_token(tokens[0])
    return None


def footprint_verbose_tag_family(tag: str) -> str:
    if tag.startswith('Owned physical footprint (unmapped) (graphics)'):
        return 'Owned physical footprint (unmapped) (graphics)'
    if tag.startswith('Owned physical footprint (unmapped)'):
        return 'Owned physical footprint (unmapped)'
    if tag.startswith('IOSurface SID:'):
        if 'CAMetalLayer Display Drawable' in tag:
            return 'IOSurface CAMetalLayer Display Drawable'
        if 'CoreUI image IOSurface' in tag:
            return 'IOSurface CoreUI image'
        return 'IOSurface SID'
    if tag.startswith('IOSurface'):
        return 'IOSurface'
    if tag.startswith('IOAccelerator (graphics)'):
        return 'IOAccelerator (graphics)'
    if tag.startswith('mapped file'):
        return 'mapped file'
    if tag.startswith('untagged ('):
        return 'untagged'
    if tag.startswith('MALLOC_'):
        return tag.split()[0]
    if tag.startswith('__'):
        return tag.split()[0]
    if ' .../' in tag:
        return tag.split(' .../', 1)[0]
    return tag


def parse_footprint_verbose_rows(stdout: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    in_table = False
    for line in stdout.splitlines():
        text = line.rstrip()
        stripped = text.strip()
        if '[object-id]' in text and 'tag (detail)' in text:
            in_table = True
            continue
        if not in_table:
            continue
        if not stripped:
            continue
        if stripped.startswith('Auxiliary data:'):
            break
        match = FOOTPRINT_VERBOSE_ROW_RE.match(text)
        if not match:
            continue
        virtual_pages = parse_u64_token(match.group('virtual_pages')) or 0
        dirty_pages = parse_u64_token(match.group('dirty_pages')) or 0
        clean_pages = parse_u64_token(match.group('clean_pages')) or 0
        reclaimable_pages = parse_u64_token(match.group('reclaimable_pages')) or 0
        tag = match.group('tag').strip()
        rows.append({
            'start': match.group('start'),
            'end': match.group('end'),
            'object_id': match.group('object_id'),
            'virtual_pages': virtual_pages,
            'dirty_pages': dirty_pages,
            'clean_pages': clean_pages,
            'reclaimable_pages': reclaimable_pages,
            'virtual_bytes': virtual_pages * PAGE_BYTES,
            'dirty_bytes': dirty_pages * PAGE_BYTES,
            'clean_bytes': clean_pages * PAGE_BYTES,
            'reclaimable_bytes': reclaimable_pages * PAGE_BYTES,
            'tag': tag,
            'family': footprint_verbose_tag_family(tag),
        })
    return rows


def parse_footprint_verbose_auxiliary(stdout: str) -> dict[str, Any]:
    aux: dict[str, Any] = {}
    for line in stdout.splitlines():
        text = line.strip()
        if text.startswith('phys_footprint:'):
            aux['phys_footprint_bytes'] = parse_size_phrase(text.removeprefix('phys_footprint:').strip())
        elif text.startswith('phys_footprint_peak:'):
            aux['phys_footprint_peak_bytes'] = parse_size_phrase(text.removeprefix('phys_footprint_peak:').strip())
    return aux


def summarize_footprint_verbose_group(rows: list[dict[str, Any]], *, top: int = 12) -> dict[str, Any]:
    limit = max(1, min(top, 32))

    def summarize_page_buckets(page_key: str, bytes_key: str) -> list[dict[str, Any]]:
        buckets: dict[int, dict[str, Any]] = {}
        for row in rows:
            pages = int(row.get(page_key) or 0)
            if pages <= 0:
                continue
            bucket = buckets.setdefault(
                pages,
                {
                    'pages': pages,
                    'bytes_per_row': pages * PAGE_BYTES,
                    'rows_total': 0,
                    'bytes_total': 0,
                },
            )
            bucket['rows_total'] += 1
            bucket['bytes_total'] += int(row.get(bytes_key) or 0)
        return sorted(
            buckets.values(),
            key=lambda bucket: (bucket['bytes_total'], bucket['rows_total'], bucket['pages']),
            reverse=True,
        )[:limit]

    top_rows_by_dirty = sorted(
        rows,
        key=lambda row: (row['dirty_bytes'], row['virtual_bytes'], row['clean_bytes']),
        reverse=True,
    )[:limit]
    top_rows_by_virtual = sorted(
        rows,
        key=lambda row: (row['virtual_bytes'], row['dirty_bytes'], row['clean_bytes']),
        reverse=True,
    )[:limit]
    return {
        'rows_total': len(rows),
        'virtual_bytes_total': sum(int(row.get('virtual_bytes') or 0) for row in rows),
        'dirty_bytes_total': sum(int(row.get('dirty_bytes') or 0) for row in rows),
        'clean_bytes_total': sum(int(row.get('clean_bytes') or 0) for row in rows),
        'reclaimable_bytes_total': sum(int(row.get('reclaimable_bytes') or 0) for row in rows),
        'dirty_page_buckets': summarize_page_buckets('dirty_pages', 'dirty_bytes'),
        'virtual_page_buckets': summarize_page_buckets('virtual_pages', 'virtual_bytes'),
        'top_rows_by_dirty': top_rows_by_dirty,
        'top_rows_by_virtual': top_rows_by_virtual,
    }


def summarize_footprint_verbose(stdout: str, top: int = 12) -> dict[str, Any]:
    rows = parse_footprint_verbose_rows(stdout)
    limit = max(1, min(top, 32))
    family_rows: dict[str, list[dict[str, Any]]] = {}
    for row in rows:
        family_rows.setdefault(row['family'], []).append(row)

    family_totals = [
        {
            'family': family,
            'rows_total': len(group_rows),
            'virtual_bytes_total': sum(int(row.get('virtual_bytes') or 0) for row in group_rows),
            'dirty_bytes_total': sum(int(row.get('dirty_bytes') or 0) for row in group_rows),
            'clean_bytes_total': sum(int(row.get('clean_bytes') or 0) for row in group_rows),
            'reclaimable_bytes_total': sum(int(row.get('reclaimable_bytes') or 0) for row in group_rows),
        }
        for family, group_rows in family_rows.items()
    ]
    focus_families = {
        family: summarize_footprint_verbose_group(family_rows[family], top=top)
        for family in FOOTPRINT_VERBOSE_FOCUS_FAMILIES
        if family in family_rows
    }
    return {
        'collector': 'footprint -v',
        'page_size_bytes': PAGE_BYTES,
        'rows_total': len(rows),
        'auxiliary': parse_footprint_verbose_auxiliary(stdout),
        'top_families_by_dirty': sorted(
            family_totals,
            key=lambda row: (row['dirty_bytes_total'], row['virtual_bytes_total'], row['rows_total']),
            reverse=True,
        )[:limit],
        'top_families_by_virtual': sorted(
            family_totals,
            key=lambda row: (row['virtual_bytes_total'], row['dirty_bytes_total'], row['rows_total']),
            reverse=True,
        )[:limit],
        'focus_families': focus_families,
        'note': 'best-effort parsed from `footprint -v`; row sizes are expressed in bytes using the local page size.',
    }


def parse_vmmap_interleaved_regions(stdout: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    in_regions = False
    for line in stdout.splitlines():
        text = line.rstrip()
        if not text:
            continue
        if 'REGION TYPE' in text and 'START - END' in text and '[' in text:
            in_regions = True
            continue
        if not in_regions:
            continue
        if text.startswith('====') or text.startswith('vmmap:'):
            break
        if text.startswith('REGION TYPE'):
            continue

        bracket_open = text.find('[')
        if bracket_open < 0:
            continue
        bracket_close = text.find(']', bracket_open)
        if bracket_close < 0:
            continue

        pre = text[:bracket_open].rstrip()
        sizes = text[bracket_open + 1:bracket_close].strip()
        post = text[bracket_close + 1:].strip()
        pre_tokens = pre.split()
        if not pre_tokens:
            continue
        start_end = pre_tokens[-1]
        region_type = pre[: -len(start_end)].strip() if pre.endswith(start_end) else pre.strip()
        if not region_type:
            continue

        size_tokens = sizes.split()
        if len(size_tokens) < 4:
            continue

        rows.append({
            'region_type': region_type,
            'start_end': start_end,
            'virtual_bytes': parse_size_token(size_tokens[0]) or 0,
            'resident_bytes': parse_size_token(size_tokens[1]) or 0,
            'dirty_bytes': parse_size_token(size_tokens[2]) or 0,
            'swapped_bytes': parse_size_token(size_tokens[3]) or 0,
            'detail': post,
        })
    return rows


def summarize_vmmap_regions_sorted(stdout: str, top: int = 24) -> dict[str, Any]:
    rows = parse_vmmap_interleaved_regions(stdout)
    limit = max(1, min(top, 64))
    top_dirty = sorted(
        rows,
        key=lambda row: (row['dirty_bytes'], row['resident_bytes'], row['virtual_bytes']),
        reverse=True,
    )[:limit]
    top_resident = sorted(
        rows,
        key=lambda row: (row['resident_bytes'], row['dirty_bytes'], row['virtual_bytes']),
        reverse=True,
    )[:limit]
    return {
        'collector': 'vmmap -sortBySize -wide -interleaved -noCoalesce',
        'tables': {
            'regions': {
                'rows_total': len(rows),
                'top_dirty': top_dirty,
                'top_resident': top_resident,
            },
        },
        'note': 'best-effort parsed from raw vmmap regions output; useful for coarse region attribution, not exact live-allocation ownership.',
    }


def parse_vmmap_regions_table(stdout: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    in_table = False
    for line in stdout.splitlines():
        text = line.strip()
        if not text:
            continue
        if text.startswith('REGION TYPE'):
            in_table = True
            continue
        if not in_table:
            continue
        if text.startswith('MALLOC ZONE'):
            break
        if text.startswith('===========') or text.startswith('TOTAL'):
            continue
        tokens = text.split()
        first_numeric = next(
            (index for index, token in enumerate(tokens) if token and token[0].isdigit()),
            None,
        )
        if first_numeric is None or len(tokens) < first_numeric + 8:
            continue
        region_type = ' '.join(tokens[:first_numeric])
        row = {
            'region_type': region_type,
            'virtual_bytes': parse_size_token(tokens[first_numeric]) or 0,
            'resident_bytes': parse_size_token(tokens[first_numeric + 1]) or 0,
            'dirty_bytes': parse_size_token(tokens[first_numeric + 2]) or 0,
            'swapped_bytes': parse_size_token(tokens[first_numeric + 3]) or 0,
            'volatile_bytes': parse_size_token(tokens[first_numeric + 4]) or 0,
            'nonvol_bytes': parse_size_token(tokens[first_numeric + 5]) or 0,
            'empty_bytes': parse_size_token(tokens[first_numeric + 6]) or 0,
            'region_count': parse_u64_token(tokens[first_numeric + 7]) or 0,
        }
        rows.append(row)
    return rows


def parse_vmmap_malloc_zone_table(stdout: str) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    in_table = False
    for line in stdout.splitlines():
        text = line.strip()
        if not text:
            continue
        if text.startswith('MALLOC ZONE'):
            in_table = True
            continue
        if not in_table:
            continue
        if text.startswith('TOTAL') or text.startswith('==========='):
            continue
        tokens = text.split()
        first_numeric = next(
            (index for index, token in enumerate(tokens) if token and token[0].isdigit()),
            None,
        )
        if first_numeric is None or len(tokens) < first_numeric + 9:
            continue
        zone = ' '.join(tokens[:first_numeric])
        row = {
            'zone': zone,
            'virtual_bytes': parse_size_token(tokens[first_numeric]) or 0,
            'resident_bytes': parse_size_token(tokens[first_numeric + 1]) or 0,
            'dirty_bytes': parse_size_token(tokens[first_numeric + 2]) or 0,
            'swapped_bytes': parse_size_token(tokens[first_numeric + 3]) or 0,
            'allocation_count': parse_u64_token(tokens[first_numeric + 4]) or 0,
            'allocated_bytes': parse_size_token(tokens[first_numeric + 5]) or 0,
            'frag_bytes': parse_size_token(tokens[first_numeric + 6]) or 0,
            'frag_percent': parse_percent_token(tokens[first_numeric + 7]),
            'region_count': parse_u64_token(tokens[first_numeric + 8]) or 0,
        }
        rows.append(row)
    return rows


def parse_vmmap_summary(stdout: str) -> dict[str, Any]:
    physical_footprint_bytes = None
    physical_footprint_peak_bytes = None
    for line in stdout.splitlines():
        text = line.strip()
        if text.startswith('Physical footprint:'):
            token = text.removeprefix('Physical footprint:').strip().split()[0]
            physical_footprint_bytes = parse_size_token(token)
        elif text.startswith('Physical footprint (peak):'):
            token = text.removeprefix('Physical footprint (peak):').strip().split()[0]
            physical_footprint_peak_bytes = parse_size_token(token)
    regions = parse_vmmap_regions_table(stdout)
    malloc_zones = parse_vmmap_malloc_zone_table(stdout)

    def region_dirty_bytes(name: str) -> int | None:
        for row in regions:
            if row['region_type'] == name:
                return row['dirty_bytes']
        return None

    def region_dirty_bytes_sum_prefix(prefix: str) -> int:
        total = 0
        for row in regions:
            if row['region_type'].startswith(prefix):
                total += row['dirty_bytes']
        return total

    total_allocated = sum(row['allocated_bytes'] for row in malloc_zones)
    total_frag = sum(row['frag_bytes'] for row in malloc_zones)
    total_dirty = sum(row['dirty_bytes'] for row in malloc_zones)

    return {
        'physical_footprint_bytes': physical_footprint_bytes,
        'physical_footprint_peak_bytes': physical_footprint_peak_bytes,
        'regions': {
            'owned_unmapped_memory_dirty_bytes': region_dirty_bytes('owned unmapped memory'),
            'io_surface_dirty_bytes': region_dirty_bytes('IOSurface'),
            'io_accelerator_dirty_bytes': region_dirty_bytes_sum_prefix('IOAccelerator'),
            'malloc_small_dirty_bytes': region_dirty_bytes('MALLOC_SMALL'),
            'malloc_dirty_bytes_total': sum(
                row['dirty_bytes']
                for row in regions
                if row['region_type'].startswith('MALLOC')
            ),
        },
        'tables': {
            'malloc_zones': {
                'rows': malloc_zones,
                'total': {
                    'allocated_bytes': total_allocated,
                    'frag_bytes': total_frag,
                    'dirty_bytes': total_dirty,
                },
            },
            'regions_top_dirty': sorted(
                regions,
                key=lambda row: row['dirty_bytes'],
                reverse=True,
            )[:12],
        },
    }


def top_categories(categories: dict[str, Any], top: int = 12) -> list[dict[str, Any]]:
    rows = []
    for name, payload in categories.items():
        dirty = int(payload.get('dirty', 0) or 0)
        clean = int(payload.get('clean', 0) or 0)
        swapped = int(payload.get('swapped', 0) or 0)
        reclaimable = int(payload.get('reclaimable', 0) or 0)
        wired = int(payload.get('wired', 0) or 0)
        rows.append({
            'category': name,
            'dirty': dirty,
            'clean': clean,
            'swapped': swapped,
            'reclaimable': reclaimable,
            'wired': wired,
            'footprint_like': dirty + clean + swapped + reclaimable + wired,
            'regions': int(payload.get('regions', 0) or 0),
        })
    rows.sort(key=lambda row: (row['footprint_like'], row['dirty']), reverse=True)
    return rows[:top]


def stop_process(process: subprocess.Popen[bytes], grace_secs: float) -> dict[str, Any]:
    result = {'terminated': False, 'killed': False, 'returncode': process.poll()}
    if process.poll() is not None:
        result['returncode'] = process.returncode
        return result
    process.terminate()
    result['terminated'] = True
    deadline = time.time() + grace_secs
    while time.time() < deadline:
        if process.poll() is not None:
            result['returncode'] = process.returncode
            return result
        time.sleep(0.1)
    process.kill()
    result['killed'] = True
    process.wait(timeout=5)
    result['returncode'] = process.returncode
    return result


def parse_sample_offsets(args: argparse.Namespace) -> list[float]:
    if args.sample_at_secs:
        values = []
        for raw in args.sample_at_secs.split(','):
            raw = raw.strip()
            if not raw:
                continue
            values.append(float(raw))
    else:
        values = [float(args.warmup_secs)]
    if not values:
        raise SystemExit('no sample offsets configured')
    unique = sorted(set(values))
    if unique[0] < 0:
        raise SystemExit('sample offsets must be >= 0')
    return unique


def sample_dir_name(offset_secs: float, index: int) -> str:
    millis = int(round(offset_secs * 1000.0))
    seconds = millis // 1000
    frac = millis % 1000
    return f'sample_{index:02d}_t{seconds:03d}_{frac:03d}s'


def build_key_metrics(
    footprint_summary: dict[str, Any] | None,
    vmmap_summary: dict[str, Any] | None,
    vmmap_regions_sorted_summary: dict[str, Any] | None,
    footprint_verbose_summary: dict[str, Any] | None,
) -> dict[str, Any]:
    top_dirty_regions = ((((vmmap_regions_sorted_summary or {}).get('tables') or {}).get('regions') or {}).get('top_dirty') or [])
    top_dirty = top_dirty_regions[0] if top_dirty_regions else {}
    focus_families = (footprint_verbose_summary or {}).get('focus_families') or {}
    owned_unmapped_graphics = focus_families.get('Owned physical footprint (unmapped) (graphics)') or {}
    owned_unmapped = focus_families.get('Owned physical footprint (unmapped)') or {}
    iosurface_drawables = focus_families.get('IOSurface CAMetalLayer Display Drawable') or {}
    ioaccelerator_graphics = focus_families.get('IOAccelerator (graphics)') or {}
    return {
        'physical_footprint_bytes': (vmmap_summary or {}).get('physical_footprint_bytes'),
        'physical_footprint_peak_bytes': (vmmap_summary or {}).get('physical_footprint_peak_bytes'),
        'owned_unmapped_memory_dirty_bytes': ((vmmap_summary or {}).get('regions') or {}).get('owned_unmapped_memory_dirty_bytes'),
        'io_surface_dirty_bytes': ((vmmap_summary or {}).get('regions') or {}).get('io_surface_dirty_bytes'),
        'io_accelerator_dirty_bytes': ((vmmap_summary or {}).get('regions') or {}).get('io_accelerator_dirty_bytes'),
        'malloc_small_dirty_bytes': ((vmmap_summary or {}).get('regions') or {}).get('malloc_small_dirty_bytes'),
        'malloc_dirty_bytes_total': ((vmmap_summary or {}).get('regions') or {}).get('malloc_dirty_bytes_total'),
        'malloc_zones_total_allocated_bytes': ((((vmmap_summary or {}).get('tables') or {}).get('malloc_zones') or {}).get('total') or {}).get('allocated_bytes'),
        'malloc_zones_total_frag_bytes': ((((vmmap_summary or {}).get('tables') or {}).get('malloc_zones') or {}).get('total') or {}).get('frag_bytes'),
        'footprint_aux_phys_footprint': ((footprint_summary or {}).get('auxiliary') or {}).get('phys_footprint'),
        'footprint_aux_phys_footprint_peak': ((footprint_summary or {}).get('auxiliary') or {}).get('phys_footprint_peak'),
        'vmmap_regions_sorted_top_dirty_region_type': top_dirty.get('region_type'),
        'vmmap_regions_sorted_top_dirty_detail': top_dirty.get('detail'),
        'vmmap_regions_sorted_top_dirty_bytes': top_dirty.get('dirty_bytes'),
        'footprint_verbose_owned_unmapped_graphics_dirty_bytes': owned_unmapped_graphics.get('dirty_bytes_total'),
        'footprint_verbose_owned_unmapped_graphics_virtual_bytes': owned_unmapped_graphics.get('virtual_bytes_total'),
        'footprint_verbose_owned_unmapped_graphics_rows_total': owned_unmapped_graphics.get('rows_total'),
        'footprint_verbose_owned_unmapped_dirty_bytes': owned_unmapped.get('dirty_bytes_total'),
        'footprint_verbose_owned_unmapped_rows_total': owned_unmapped.get('rows_total'),
        'footprint_verbose_iosurface_drawables_dirty_bytes': iosurface_drawables.get('dirty_bytes_total'),
        'footprint_verbose_ioaccelerator_graphics_dirty_bytes': ioaccelerator_graphics.get('dirty_bytes_total'),
    }


def load_footprint_summary(footprint_json_path: Path) -> dict[str, Any] | None:
    if not footprint_json_path.is_file():
        return None
    payload = json.loads(footprint_json_path.read_text())
    first = (payload.get('processes') or [None])[0]
    if not isinstance(first, dict):
        return None
    return {
        'footprint': first.get('footprint'),
        'auxiliary': first.get('auxiliary', {}),
        'top_categories': top_categories(first.get('categories', {})),
    }


def capture_sample(
    pid: int,
    out_dir: Path,
    *,
    capture_vmmap_regions: bool = False,
    capture_footprint_verbose: bool = False,
) -> dict[str, Any]:
    out_dir.mkdir(parents=True, exist_ok=True)
    footprint_json_path = out_dir / 'resource.macos_footprint.json'
    footprint_verbose_path = out_dir / 'resource.footprint_verbose.txt'
    vmmap_summary_path = out_dir / 'resource.vmmap_summary.txt'
    vmmap_regions_path = out_dir / 'resource.vmmap_regions_sorted.txt'
    sample_errors: list[str] = []
    started_unix_ms = int(time.time() * 1000)

    try:
        run_capture_command(
            ['/usr/bin/footprint', '-j', str(footprint_json_path), '-p', str(pid)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except Exception as exc:  # noqa: BLE001
        sample_errors.append(f'footprint:{exc!r}')

    if capture_footprint_verbose:
        try:
            footprint_verbose_out = run_capture_command(
                ['/usr/bin/footprint', '-v', '-p', str(pid)],
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
            )
            footprint_verbose_path.write_text(footprint_verbose_out.stdout)
        except Exception as exc:  # noqa: BLE001
            sample_errors.append(f'footprint-verbose:{exc!r}')

    try:
        vmmap_out = run_capture_command(
            ['/usr/bin/vmmap', '-summary', str(pid)],
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
        vmmap_summary_path.write_text(vmmap_out.stdout)
    except Exception as exc:  # noqa: BLE001
        sample_errors.append(f'vmmap-summary:{exc!r}')

    if capture_vmmap_regions:
        try:
            vmmap_regions_out = run_capture_command(
                ['/usr/bin/vmmap', '-sortBySize', '-wide', '-interleaved', '-noCoalesce', str(pid)],
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
            )
            vmmap_regions_path.write_text(vmmap_regions_out.stdout)
        except Exception as exc:  # noqa: BLE001
            sample_errors.append(f'vmmap-regions:{exc!r}')

    footprint_summary = load_footprint_summary(footprint_json_path)
    vmmap_summary = None
    if vmmap_summary_path.is_file():
        vmmap_summary = parse_vmmap_summary(vmmap_summary_path.read_text())
    vmmap_regions_sorted_summary = None
    if vmmap_regions_path.is_file():
        vmmap_regions_sorted_summary = summarize_vmmap_regions_sorted(vmmap_regions_path.read_text())
    footprint_verbose_summary = None
    if footprint_verbose_path.is_file():
        footprint_verbose_summary = summarize_footprint_verbose(footprint_verbose_path.read_text())

    return {
        'start_unix_ms': started_unix_ms,
        'end_unix_ms': int(time.time() * 1000),
        'sample_error': '; '.join(sample_errors) if sample_errors else None,
        'artifacts': {
            'footprint_json': str(footprint_json_path) if footprint_json_path.is_file() else None,
            'footprint_verbose': str(footprint_verbose_path) if footprint_verbose_path.is_file() else None,
            'vmmap_summary': str(vmmap_summary_path) if vmmap_summary_path.is_file() else None,
            'vmmap_regions_sorted': str(vmmap_regions_path) if vmmap_regions_path.is_file() else None,
        },
        'footprint': footprint_summary,
        'footprint_verbose': footprint_verbose_summary,
        'vmmap': vmmap_summary,
        'vmmap_regions_sorted': vmmap_regions_sorted_summary,
        'key_metrics': build_key_metrics(footprint_summary, vmmap_summary, vmmap_regions_sorted_summary, footprint_verbose_summary),
    }


def compact_sample_summary(sample: dict[str, Any]) -> dict[str, Any]:
    return {
        'offset_secs': sample['offset_secs'],
        'sample_error': sample['sample_error'],
        'key_metrics': sample['key_metrics'],
    }


def is_nonfatal_sample_error(sample_error: str | None) -> bool:
    if not sample_error:
        return False
    parts = [part.strip() for part in sample_error.split(';') if part.strip()]
    nonfatal_prefixes = ('vmmap-regions:', 'footprint-verbose:')
    return bool(parts) and all(part.startswith(nonfatal_prefixes) for part in parts)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument('--out-dir', required=True)
    parser.add_argument('--label', required=True)
    parser.add_argument('--warmup-secs', type=float, default=6.0)
    parser.add_argument('--sample-at-secs', help='comma-separated sample offsets from process start, e.g. 2,6,12')
    parser.add_argument('--shutdown-grace-secs', type=float, default=5.0)
    parser.add_argument('--post-sample-wait-secs', type=float, default=0.0)
    parser.add_argument(
        '--capture-vmmap-regions',
        action='store_true',
        help='Also capture `vmmap -sortBySize -wide -interleaved -noCoalesce` raw output per sample.',
    )
    parser.add_argument(
        '--capture-footprint-verbose',
        action='store_true',
        help='Also capture `footprint -v` raw output per sample and summarize focus families.',
    )
    parser.add_argument('cmd', nargs=argparse.REMAINDER)
    args = parser.parse_args()

    cmd = args.cmd
    if cmd and cmd[0] == '--':
        cmd = cmd[1:]
    if not cmd:
        raise SystemExit('missing command after --')

    sample_offsets_secs = parse_sample_offsets(args)

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    stdout_path = out_dir / 'stdout.log'
    stderr_path = out_dir / 'stderr.log'
    summary_path = out_dir / 'summary.json'

    start_unix_ms = int(time.time() * 1000)
    samples: list[dict[str, Any]] = []

    with stdout_path.open('wb') as stdout_file, stderr_path.open('wb') as stderr_file:
        process = subprocess.Popen(cmd, stdout=stdout_file, stderr=stderr_file)
        sample_error = None
        try:
            launch_monotonic = time.monotonic()
            for index, offset_secs in enumerate(sample_offsets_secs):
                deadline = launch_monotonic + offset_secs
                remaining = deadline - time.monotonic()
                if remaining > 0:
                    time.sleep(remaining)
                sample_out_dir = out_dir if len(sample_offsets_secs) == 1 else out_dir / 'samples' / sample_dir_name(offset_secs, index)
                sample = capture_sample(
                    process.pid,
                    sample_out_dir,
                    capture_vmmap_regions=args.capture_vmmap_regions,
                    capture_footprint_verbose=args.capture_footprint_verbose,
                )
                sample['offset_secs'] = offset_secs
                sample['label'] = args.label if len(sample_offsets_secs) == 1 else f"{args.label}@{offset_secs:.3f}s"
                samples.append(sample)
                if sample['sample_error'] and sample_error is None:
                    sample_error = sample['sample_error']
            if args.post_sample_wait_secs > 0:
                time.sleep(args.post_sample_wait_secs)
        finally:
            stop = stop_process(process, args.shutdown_grace_secs)

    last_sample = samples[-1] if samples else None
    summary = {
        'schema_version': 2,
        'label': args.label,
        'command': cmd,
        'pid': process.pid,
        'warmup_secs': args.warmup_secs,
        'post_sample_wait_secs': args.post_sample_wait_secs,
        'sample_at_secs': sample_offsets_secs,
        'start_unix_ms': start_unix_ms,
        'end_unix_ms': int(time.time() * 1000),
        'sample_error': sample_error,
        'stop': stop,
        'artifacts': {
            'stdout': str(stdout_path),
            'stderr': str(stderr_path),
            'footprint_json': (last_sample or {}).get('artifacts', {}).get('footprint_json'),
            'footprint_verbose': (last_sample or {}).get('artifacts', {}).get('footprint_verbose'),
            'vmmap_summary': (last_sample or {}).get('artifacts', {}).get('vmmap_summary'),
            'vmmap_regions_sorted': (last_sample or {}).get('artifacts', {}).get('vmmap_regions_sorted'),
            'samples_dir': str(out_dir / 'samples') if len(sample_offsets_secs) > 1 else None,
        },
        'footprint': (last_sample or {}).get('footprint'),
        'footprint_verbose': (last_sample or {}).get('footprint_verbose'),
        'vmmap': (last_sample or {}).get('vmmap'),
        'vmmap_regions_sorted': (last_sample or {}).get('vmmap_regions_sorted'),
        'key_metrics': (last_sample or {}).get('key_metrics'),
        'samples': samples,
    }
    summary_path.write_text(json.dumps(summary, indent=2))

    if len(sample_offsets_secs) == 1:
        print(json.dumps(summary['key_metrics'], indent=2))
    else:
        print(json.dumps({
            'label': args.label,
            'sample_at_secs': sample_offsets_secs,
            'samples': [compact_sample_summary(sample) for sample in samples],
        }, indent=2))
    return 0 if sample_error is None or is_nonfatal_sample_error(sample_error) else 1


if __name__ == '__main__':
    raise SystemExit(main())
