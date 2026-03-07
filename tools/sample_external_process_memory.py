#!/usr/bin/env python3
"""Launch an external macOS GUI process, capture one or more footprint/vmmap snapshots, and stop it."""

from __future__ import annotations

import argparse
import json
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
) -> dict[str, Any]:
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
) -> dict[str, Any]:
    out_dir.mkdir(parents=True, exist_ok=True)
    footprint_json_path = out_dir / 'resource.macos_footprint.json'
    vmmap_summary_path = out_dir / 'resource.vmmap_summary.txt'
    vmmap_regions_path = out_dir / 'resource.vmmap_regions_sorted.txt'
    sample_errors: list[str] = []
    started_unix_ms = int(time.time() * 1000)

    try:
        subprocess.run(
            ['/usr/bin/footprint', '-j', str(footprint_json_path), '-p', str(pid)],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except Exception as exc:  # noqa: BLE001
        sample_errors.append(f'footprint:{exc!r}')

    try:
        vmmap_out = subprocess.run(
            ['/usr/bin/vmmap', '-summary', str(pid)],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
            text=True,
        )
        vmmap_summary_path.write_text(vmmap_out.stdout)
    except Exception as exc:  # noqa: BLE001
        sample_errors.append(f'vmmap-summary:{exc!r}')

    if capture_vmmap_regions:
        try:
            vmmap_regions_out = subprocess.run(
                ['/usr/bin/vmmap', '-sortBySize', '-wide', '-interleaved', '-noCoalesce', str(pid)],
                check=True,
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

    return {
        'start_unix_ms': started_unix_ms,
        'end_unix_ms': int(time.time() * 1000),
        'sample_error': '; '.join(sample_errors) if sample_errors else None,
        'artifacts': {
            'footprint_json': str(footprint_json_path) if footprint_json_path.is_file() else None,
            'vmmap_summary': str(vmmap_summary_path) if vmmap_summary_path.is_file() else None,
            'vmmap_regions_sorted': str(vmmap_regions_path) if vmmap_regions_path.is_file() else None,
        },
        'footprint': footprint_summary,
        'vmmap': vmmap_summary,
        'key_metrics': build_key_metrics(footprint_summary, vmmap_summary),
    }


def compact_sample_summary(sample: dict[str, Any]) -> dict[str, Any]:
    return {
        'offset_secs': sample['offset_secs'],
        'sample_error': sample['sample_error'],
        'key_metrics': sample['key_metrics'],
    }


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
            'vmmap_summary': (last_sample or {}).get('artifacts', {}).get('vmmap_summary'),
            'vmmap_regions_sorted': (last_sample or {}).get('artifacts', {}).get('vmmap_regions_sorted'),
            'samples_dir': str(out_dir / 'samples') if len(sample_offsets_secs) > 1 else None,
        },
        'footprint': (last_sample or {}).get('footprint'),
        'vmmap': (last_sample or {}).get('vmmap'),
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
    return 0 if sample_error is None else 1


if __name__ == '__main__':
    raise SystemExit(main())
