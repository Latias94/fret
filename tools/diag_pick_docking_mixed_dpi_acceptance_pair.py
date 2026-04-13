#!/usr/bin/env python3
"""Pick the best mixed-DPI acceptance pair from a docking diagnostics run."""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from datetime import date
from pathlib import Path

SCHEMA_VERSION = 1
PRE_LABEL = "multiwindow-drag-back-outer-sweep-after-tearoff"
POST_LABELS = (
    "multiwindow-drag-back-outer-sweep-after-outer-move-pos-x",
    "multiwindow-drag-back-outer-sweep-after-outer-move-neg-x",
)
ALL_LABELS = (PRE_LABEL, *POST_LABELS)
SESSION_MARKERS = (
    "script.result.json",
    "latest.txt",
    "diag.config.json",
    "trigger.touch",
    "ready.touch",
    "exit.touch",
)


@dataclass(frozen=True)
class BundleRef:
    label: str
    timestamp: int
    bundle_dir: Path
    session_dir: Path


@dataclass(frozen=True)
class CandidateSummary:
    label: str
    bundle_dir: Path
    session_dir: Path
    timestamp: int
    score: int
    mixed_dpi_signal_observed: bool
    distinct_scale_factor_count: int
    scale_factors_seen_x1000: list[int]
    current_window_scale_factors_x1000: list[int]
    moving_window_scale_factors_x1000: list[int]
    runner_scale_factors_x1000: list[int]
    windows_touched_total: int
    entries_total: int
    cross_window_hover_observed: bool
    origin_platform_observed: bool
    clamped_observed: bool
    summary_line: str

    def to_json(self) -> dict[str, object]:
        return {
            "label": self.label,
            "bundle_dir": self.bundle_dir.as_posix(),
            "session_dir": self.session_dir.as_posix(),
            "timestamp": self.timestamp,
            "score": self.score,
            "mixed_dpi_signal_observed": self.mixed_dpi_signal_observed,
            "distinct_scale_factor_count": self.distinct_scale_factor_count,
            "scale_factors_seen_x1000": self.scale_factors_seen_x1000,
            "current_window_scale_factors_x1000": self.current_window_scale_factors_x1000,
            "moving_window_scale_factors_x1000": self.moving_window_scale_factors_x1000,
            "runner_scale_factors_x1000": self.runner_scale_factors_x1000,
            "windows_touched_total": self.windows_touched_total,
            "entries_total": self.entries_total,
            "cross_window_hover_observed": self.cross_window_hover_observed,
            "origin_platform_observed": self.origin_platform_observed,
            "clamped_observed": self.clamped_observed,
            "summary_line": self.summary_line,
        }


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Summarize the docking mixed-DPI outer-position sweep capture and recommend "
            "one pre-crossing + one post-crossing bundle."
        ),
        formatter_class=argparse.ArgumentDefaultsHelpFormatter,
    )
    parser.add_argument(
        "path",
        help="Diagnostics out dir, session dir, run dir, or candidate bundle dir to inspect.",
    )
    parser.add_argument(
        "--fretboard-bin",
        help=(
            "Optional path to a prebuilt fretboard-dev executable. When omitted, the script uses "
            "`cargo run -p fretboard-dev -- diag dock-routing ...`."
        ),
    )
    parser.add_argument(
        "--warmup-frames",
        type=int,
        default=0,
        help="Forwarded to `diag dock-routing`.",
    )
    parser.add_argument(
        "--json-out",
        help="Optional path to write the bounded selection summary as JSON.",
    )
    parser.add_argument(
        "--note-out",
        help="Optional path to write a Markdown evidence note draft.",
    )
    parser.add_argument(
        "--note-date",
        default=date.today().isoformat(),
        help="Date stamp to embed in the generated note.",
    )
    parser.add_argument(
        "--windows-version",
        help="Optional Windows version string for the note host summary.",
    )
    parser.add_argument(
        "--monitor-arrangement",
        help="Optional monitor arrangement summary for the note host summary.",
    )
    parser.add_argument(
        "--scale-factors-used",
        help="Optional host scale factor summary for the note host summary.",
    )
    parser.add_argument(
        "--canonical-command",
        help="Optional exact `diag run` command string to embed in the note.",
    )
    parser.add_argument(
        "--automation-followup",
        help="Optional follow-on automation verdict to embed in the note.",
    )
    return parser.parse_args(argv)


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def looks_like_session_root(path: Path) -> bool:
    return any((path / marker).exists() for marker in SESSION_MARKERS)


def resolve_bundle_artifact_path(bundle_dir: Path) -> Path | None:
    for relative in (
        Path("bundle.schema2.json"),
        Path("bundle.json"),
        Path("_root/bundle.schema2.json"),
        Path("_root/bundle.json"),
    ):
        candidate = bundle_dir / relative
        if candidate.is_file():
            return candidate
    return None


def parse_candidate_dir_name(name: str) -> tuple[int, str] | None:
    for label in ALL_LABELS:
        suffix = f"-{label}"
        if not name.endswith(suffix):
            continue
        timestamp = name[: -len(suffix)]
        if not timestamp.isdigit():
            continue
        return int(timestamp), label
    return None


def find_context_root(path: Path, search_root: Path) -> Path:
    current = path
    for _ in range(12):
        if looks_like_session_root(current):
            return current
        if current == search_root or current.parent == current:
            break
        current = current.parent
    if looks_like_session_root(search_root):
        return search_root
    return search_root if search_root.is_dir() else path.parent


def find_candidate_bundles(search_root: Path) -> dict[Path, list[BundleRef]]:
    groups: dict[Path, list[BundleRef]] = {}
    queue: list[Path] = [search_root]
    visited: set[Path] = set()
    while queue:
        current = queue.pop()
        if current in visited or not current.exists():
            continue
        visited.add(current)
        if current.is_dir():
            parsed = parse_candidate_dir_name(current.name)
            if parsed is not None and resolve_bundle_artifact_path(current) is not None:
                timestamp, label = parsed
                context_root = find_context_root(current, search_root)
                groups.setdefault(context_root, []).append(
                    BundleRef(
                        label=label,
                        timestamp=timestamp,
                        bundle_dir=current,
                        session_dir=context_root,
                    )
                )
                continue
            try:
                children = list(current.iterdir())
            except OSError:
                continue
            queue.extend(reversed(children))
    return groups


def normalize_search_root(input_root: Path) -> Path:
    if input_root.is_file():
        return input_root.parent
    parsed = parse_candidate_dir_name(input_root.name)
    if parsed is None or resolve_bundle_artifact_path(input_root) is None:
        return input_root
    parent = input_root.parent
    return find_context_root(input_root, parent)


def choose_context(groups: dict[Path, list[BundleRef]]) -> tuple[Path, list[BundleRef]]:
    if not groups:
        raise SystemExit("No mixed-DPI candidate bundle directories were found under the provided path.")

    ranked = sorted(
        groups.items(),
        key=lambda item: (
            has_label(item[1], PRE_LABEL) and any(has_label(item[1], label) for label in POST_LABELS),
            max(ref.timestamp for ref in item[1]),
            item[0].as_posix(),
        ),
        reverse=True,
    )
    return ranked[0]


def has_label(items: list[BundleRef], label: str) -> bool:
    return any(item.label == label for item in items)


def latest_for_label(items: list[BundleRef], label: str) -> BundleRef | None:
    matching = [item for item in items if item.label == label]
    if not matching:
        return None
    return max(matching, key=lambda item: (item.timestamp, item.bundle_dir.as_posix()))


def distinct_ints(values: list[int | None]) -> list[int]:
    return sorted({value for value in values if value is not None and value > 0})


def format_scale_factor(value_x1000: int) -> str:
    return f"{value_x1000 / 1000.0:.3f}"


def format_scale_factor_list(values_x1000: list[int]) -> str:
    if not values_x1000:
        return "(none)"
    return ", ".join(format_scale_factor(value) for value in values_x1000)


def format_point(point: dict[str, object] | None) -> str | None:
    if not isinstance(point, dict):
        return None
    x = point.get("x")
    y = point.get("y")
    if not isinstance(x, (int, float)) or not isinstance(y, (int, float)):
        return None
    return f"{x:.1f},{y:.1f}"


def placeholder(value: str | None, *, text: str = "TODO") -> str:
    return value if value and value.strip() else text


def dock_routing_command(
    *,
    repo: Path,
    fretboard_bin: str | None,
    bundle_dir: Path,
    warmup_frames: int,
) -> list[str]:
    if fretboard_bin:
        return [
            str(resolve_tool_path(repo, fretboard_bin)),
            "diag",
            "dock-routing",
            str(bundle_dir),
            "--warmup-frames",
            str(warmup_frames),
            "--json",
        ]
    return [
        "cargo",
        "run",
        "-p",
        "fretboard-dev",
        "--",
        "diag",
        "dock-routing",
        str(bundle_dir),
        "--warmup-frames",
        str(warmup_frames),
        "--json",
    ]


def resolve_tool_path(repo: Path, value: str) -> Path:
    path = Path(value)
    if path.is_absolute():
        return path
    return (repo / path).resolve()


def load_dock_routing_json(
    *,
    repo: Path,
    fretboard_bin: str | None,
    bundle_dir: Path,
    warmup_frames: int,
) -> dict[str, object]:
    command = dock_routing_command(
        repo=repo,
        fretboard_bin=fretboard_bin,
        bundle_dir=bundle_dir,
        warmup_frames=warmup_frames,
    )
    proc = subprocess.run(
        command,
        cwd=str(repo),
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if proc.returncode != 0:
        raise SystemExit(
            "Failed to read dock-routing JSON.\n"
            f"bundle_dir: {bundle_dir}\n"
            f"command: {' '.join(command)}\n"
            f"exit_code: {proc.returncode}\n"
            f"stderr:\n{proc.stderr.strip()}"
        )
    try:
        payload = json.loads(proc.stdout)
    except json.JSONDecodeError as exc:
        raise SystemExit(
            "Failed to parse dock-routing JSON output.\n"
            f"bundle_dir: {bundle_dir}\n"
            f"command: {' '.join(command)}\n"
            f"stdout:\n{proc.stdout.strip()}"
        ) from exc
    if not isinstance(payload, dict):
        raise SystemExit(f"dock-routing output must be a JSON object: {bundle_dir}")
    return payload


def summarize_candidate(ref: BundleRef, routing: dict[str, object]) -> CandidateSummary:
    entries = routing.get("entries")
    entry_list = entries if isinstance(entries, list) else []
    observed_scale_factors = routing.get("observed_scale_factors_x1000")

    scale_factors_seen = distinct_ints(
        [
            value if isinstance(value, int) else None
            for value in observed_scale_factors
        ]
        if isinstance(observed_scale_factors, list)
        else []
    )
    drag_entries = iter_drag_entries(entry_list)
    current_scale_factors = distinct_ints(
        [
            drag.get("current_window_scale_factor_x1000")
            for drag in drag_entries
        ]
    )
    moving_scale_factors = distinct_ints(
        [drag.get("moving_window_scale_factor_x1000") for drag in drag_entries]
    )
    runner_scale_factors = distinct_ints(
        [
            drag.get("current_window_scale_factor_x1000_from_runner")
            for drag in drag_entries
        ]
    )
    mixed_dpi_signal_observed = bool(routing.get("mixed_dpi_signal_observed"))
    windows_touched_total = int(routing.get("windows_touched_total", 0) or 0)
    entries_total = int(routing.get("entries_total", len(entry_list)) or 0)
    cross_window_hover_observed = any(
        bool(drag.get("cross_window_hover")) for drag in drag_entries
    )
    origin_platform_observed = any(
        bool(drag.get("current_window_client_origin_source_platform"))
        for drag in drag_entries
    )
    clamped_observed = any(
        bool(drag.get("cursor_screen_pos_was_clamped")) for drag in drag_entries
    )

    scale_pair_diverged = False
    for drag in drag_entries:
        current = drag.get("current_window_scale_factor_x1000")
        moving = drag.get("moving_window_scale_factor_x1000")
        if isinstance(current, int) and isinstance(moving, int) and current > 0 and moving > 0:
            if current != moving:
                scale_pair_diverged = True
                break

    recent_drag = latest_drag_entry(entry_list)
    score = 0
    if mixed_dpi_signal_observed:
        score += 100
    score += 10 * len(scale_factors_seen)
    if scale_pair_diverged:
        score += 5
    if origin_platform_observed:
        score += 3
    if cross_window_hover_observed:
        score += 2
    if runner_scale_factors:
        score += 1

    summary_line = format_summary_line(
        mixed_dpi_signal_observed=mixed_dpi_signal_observed,
        scale_factors_seen=scale_factors_seen,
        recent_drag=recent_drag,
        clamped_observed=clamped_observed,
        cross_window_hover_observed=cross_window_hover_observed,
        origin_platform_observed=origin_platform_observed,
        entries_total=entries_total,
    )

    return CandidateSummary(
        label=ref.label,
        bundle_dir=ref.bundle_dir,
        session_dir=ref.session_dir,
        timestamp=ref.timestamp,
        score=score,
        mixed_dpi_signal_observed=mixed_dpi_signal_observed,
        distinct_scale_factor_count=len(scale_factors_seen),
        scale_factors_seen_x1000=scale_factors_seen,
        current_window_scale_factors_x1000=current_scale_factors,
        moving_window_scale_factors_x1000=moving_scale_factors,
        runner_scale_factors_x1000=runner_scale_factors,
        windows_touched_total=windows_touched_total,
        entries_total=entries_total,
        cross_window_hover_observed=cross_window_hover_observed,
        origin_platform_observed=origin_platform_observed,
        clamped_observed=clamped_observed,
        summary_line=summary_line,
    )


def iter_drag_entries(entries: list[object]) -> list[dict[str, object]]:
    out: list[dict[str, object]] = []
    for entry in entries:
        if not isinstance(entry, dict):
            continue
        drag = entry.get("dock_drag")
        if isinstance(drag, dict):
            out.append(drag)
    return out


def latest_drag_entry(entries: list[object]) -> dict[str, object] | None:
    for entry in reversed(entries):
        if not isinstance(entry, dict):
            continue
        drag = entry.get("dock_drag")
        if isinstance(drag, dict):
            return drag
    return None


def format_summary_line(
    *,
    mixed_dpi_signal_observed: bool,
    scale_factors_seen: list[int],
    recent_drag: dict[str, object] | None,
    clamped_observed: bool,
    cross_window_hover_observed: bool,
    origin_platform_observed: bool,
    entries_total: int,
) -> str:
    parts = [
        f"mixed_dpi={str(mixed_dpi_signal_observed).lower()}",
        f"scale_factors={format_scale_factor_list(scale_factors_seen)}",
        f"entries={entries_total}",
    ]
    if recent_drag is not None:
        for key, name in (
            ("current_window_scale_factor_x1000_from_runner", "sf_run"),
            ("current_window_scale_factor_x1000", "sf_cur"),
            ("moving_window_scale_factor_x1000", "sf_move"),
        ):
            value = recent_drag.get(key)
            if isinstance(value, int) and value > 0:
                parts.append(f"{name}={format_scale_factor(value)}")
        scr_used = format_point(
            recent_drag.get("cursor_screen_pos_used_physical_px")
            if isinstance(recent_drag.get("cursor_screen_pos_used_physical_px"), dict)
            else None
        )
        if scr_used is not None:
            parts.append(f"scr_used=({scr_used})")
        origin = format_point(
            recent_drag.get("current_window_client_origin_screen_physical_px")
            if isinstance(
                recent_drag.get("current_window_client_origin_screen_physical_px"), dict
            )
            else None
        )
        if origin is not None:
            parts.append(f"origin=({origin})")
    if origin_platform_observed:
        parts.append("origin_src=platform")
    if cross_window_hover_observed:
        parts.append("cross=1")
    if clamped_observed:
        parts.append("clamped=1")
    return " ".join(parts)


def choose_post_candidate(candidates: list[CandidateSummary]) -> CandidateSummary | None:
    if not candidates:
        return None
    return max(
        candidates,
        key=lambda item: (
            item.score,
            item.distinct_scale_factor_count,
            item.timestamp,
            item.label,
        ),
    )


def write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def crossing_direction_for_label(label: str | None) -> str:
    if label == "multiwindow-drag-back-outer-sweep-after-outer-move-pos-x":
        return "+X"
    if label == "multiwindow-drag-back-outer-sweep-after-outer-move-neg-x":
        return "-X"
    return "TODO"


def mixed_dpi_signal_presence(
    pre_candidate: CandidateSummary | None,
    post_candidate: CandidateSummary | None,
) -> str:
    pre_signal = bool(pre_candidate and pre_candidate.mixed_dpi_signal_observed)
    post_signal = bool(post_candidate and post_candidate.mixed_dpi_signal_observed)
    if pre_signal and post_signal:
        return "both bundles"
    if not pre_signal and post_signal:
        return "post-crossing only"
    if pre_signal and not post_signal:
        return "pre-crossing only"
    return "neither bundle"


def likely_failure_reason(
    pre_candidate: CandidateSummary | None,
    post_candidate: CandidateSummary | None,
    post_candidates: list[CandidateSummary],
) -> str:
    if post_candidate is None:
        return "No post-crossing candidate bundle was summarized; verify the run completed and captured the outer-move bundles."
    if post_candidate.mixed_dpi_signal_observed and post_candidate.distinct_scale_factor_count >= 2:
        return "No immediate failure classification from bounded routing evidence."
    if post_candidate.clamped_observed:
        return "Likely host/setup mismatch or Windows-clamped outer positions; verify monitor layout before reopening routing logic."
    if post_candidate.distinct_scale_factor_count < 2:
        return "Likely host/setup mismatch; the selected post-crossing bundle still exposed only one scale factor."
    if any(candidate.clamped_observed for candidate in post_candidates):
        return "Possible initial-placement or window-decoration drift (`DW-P1-win-002`) because one post candidate reported clamped cursor/outer-position evidence."
    if pre_candidate and pre_candidate.cross_window_hover_observed and not post_candidate.cross_window_hover_observed:
        return "Possible routing drift; cross-window hover evidence weakened after the outer move."
    return "Possible routing drift; inspect the losing post-crossing candidate and the raw `diag dock-routing --json` output."


def acceptance_checklist_lines(
    *,
    pre_candidate: CandidateSummary | None,
    post_candidate: CandidateSummary | None,
) -> list[str]:
    lines = [
        "- Drag-back completion to one canonical dock graph (`floatings=[]`): TODO (confirm from the full run or final bundle).",
        f"- Post-crossing bundle reports `mixed_dpi_signal_observed: true`: {'yes' if post_candidate and post_candidate.mixed_dpi_signal_observed else 'no'}.",
        f"- Post-crossing bundle reports at least two distinct scale factors: {'yes' if post_candidate and post_candidate.distinct_scale_factor_count >= 2 else 'no'}.",
        "- `dock-routing` keeps stable `scr/scr_used/origin` and `sf_cur/sf_move` evidence: review manually against the summary lines below.",
        "- No empty floating window or stuck-follow regression while crossing monitors: TODO (confirm from the full run).",
    ]
    if pre_candidate is None:
        lines.append("- Pre-crossing bundle was not available: no.")
    return lines


def render_note_markdown(
    *,
    note_date: str,
    input_path: Path,
    search_root: Path,
    selected_context: Path,
    pre_candidate: CandidateSummary | None,
    post_candidates: list[CandidateSummary],
    selected_post: CandidateSummary | None,
    acceptance_ready: bool,
    canonical_command: str | None,
    windows_version: str | None,
    monitor_arrangement: str | None,
    scale_factors_used: str | None,
    automation_followup: str | None,
) -> str:
    selected_post_label = selected_post.label if selected_post is not None else None
    candidate_lines = "\n".join(
        f"- `{candidate.label}`: {candidate.summary_line}" for candidate in post_candidates
        if selected_post is None or candidate.bundle_dir != selected_post.bundle_dir
    )
    if not candidate_lines:
        candidate_lines = "- `TODO`: no losing post-crossing candidate remained after selection."

    followup = automation_followup
    if not followup:
        if acceptance_ready:
            followup = (
                "No immediate automation follow-on is justified from this run alone; record the note and reopen automation only if repeated real-host captures disagree."
            )
        else:
            followup = (
                "Yes; review the failure classification below before deciding whether the next slice belongs in host setup, routing, or `DW-P1-win-002`."
            )

    lines = [
        f"# Windows Mixed-DPI Acceptance Evidence - {note_date}",
        "",
        "Status: draft evidence note generated from bounded routing summary",
        "",
        "Related:",
        "",
        "- `docs/workstreams/docking-multiwindow-imgui-parity/M2_WINDOWS_MIXED_DPI_CAPTURE_PLAN_2026-04-13.md`",
        "- `tools/diag_pick_docking_mixed_dpi_acceptance_pair.py`",
        "",
        "## Host summary",
        "",
        f"- Windows version: {placeholder(windows_version)}",
        f"- Monitor arrangement: {placeholder(monitor_arrangement)}",
        f"- Scale factors used: {placeholder(scale_factors_used)}",
        f"- Successful crossing direction: {crossing_direction_for_label(selected_post_label)}",
        "",
        "## Commands",
        "",
        f"- Canonical `diag run` command: `{placeholder(canonical_command)}`",
        f"- Helper input path: `{input_path.as_posix()}`",
        f"- Helper search root: `{search_root.as_posix()}`",
        "",
        "## Selected bundles",
        "",
        f"- Session directory: `{selected_context.as_posix()}`",
        f"- `pre-crossing`: `{pre_candidate.bundle_dir.as_posix() if pre_candidate else 'TODO'}`",
        f"- `post-crossing`: `{selected_post.bundle_dir.as_posix() if selected_post else 'TODO'}`",
        "",
        "## Dock-routing summary",
        "",
        f"- `pre-crossing`: {pre_candidate.summary_line if pre_candidate else 'TODO'}",
        f"- `post-crossing`: {selected_post.summary_line if selected_post else 'TODO'}",
        f"- `mixed_dpi_signal_observed` presence: {mixed_dpi_signal_presence(pre_candidate, selected_post)}",
        "",
        "## Acceptance checklist",
        "",
        *acceptance_checklist_lines(
            pre_candidate=pre_candidate,
            post_candidate=selected_post,
        ),
        "",
        "## Losing post-crossing candidates",
        "",
        candidate_lines,
        "",
        "## Follow-on automation verdict",
        "",
        f"- Automation follow-on still justified: {followup}",
        "",
        "## Failure classification (heuristic)",
        "",
        f"- {likely_failure_reason(pre_candidate, selected_post, post_candidates)}",
        "",
    ]
    return "\n".join(lines)


def render_summary(
    *,
    input_path: Path,
    search_root: Path,
    contexts_found: int,
    selected_context: Path,
    pre_candidate: CandidateSummary | None,
    post_candidates: list[CandidateSummary],
    selected_post: CandidateSummary | None,
    warnings: list[str],
) -> str:
    lines = [
        "mixed_dpi_acceptance_pair:",
        f"  schema_version: {SCHEMA_VERSION}",
        f"  input_path: {input_path.as_posix()}",
        f"  search_root: {search_root.as_posix()}",
        f"  contexts_found: {contexts_found}",
        f"  selected_context: {selected_context.as_posix()}",
    ]
    lines.append(
        "  selected_pre_crossing: "
        + (pre_candidate.bundle_dir.as_posix() if pre_candidate else "null")
    )
    lines.append(
        "  selected_post_crossing: "
        + (selected_post.bundle_dir.as_posix() if selected_post else "null")
    )
    acceptance_ready = bool(
        selected_post
        and selected_post.mixed_dpi_signal_observed
        and selected_post.distinct_scale_factor_count >= 2
    )
    lines.append(f"  acceptance_ready: {str(acceptance_ready).lower()}")
    lines.append("  candidates:")
    if pre_candidate is not None:
        lines.extend(render_candidate(pre_candidate))
    for candidate in post_candidates:
        lines.extend(render_candidate(candidate))
    if warnings:
        lines.append("  warnings:")
        for warning in warnings:
            lines.append(f"    - {warning}")
    return "\n".join(lines)


def render_candidate(candidate: CandidateSummary) -> list[str]:
    return [
        f"    - label: {candidate.label}",
        f"      bundle_dir: {candidate.bundle_dir.as_posix()}",
        f"      score: {candidate.score}",
        f"      mixed_dpi_signal_observed: {str(candidate.mixed_dpi_signal_observed).lower()}",
        f"      scale_factors_seen: {format_scale_factor_list(candidate.scale_factors_seen_x1000)}",
        f"      summary: {candidate.summary_line}",
    ]


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    repo = repo_root()
    input_root = Path(args.path).expanduser().resolve()
    if not input_root.exists():
        raise SystemExit(f"Path does not exist: {input_root}")
    search_root = normalize_search_root(input_root)

    groups = find_candidate_bundles(search_root)
    selected_context, bundle_refs = choose_context(groups)

    pre_ref = latest_for_label(bundle_refs, PRE_LABEL)
    post_refs = [
        ref for label in POST_LABELS if (ref := latest_for_label(bundle_refs, label)) is not None
    ]

    warnings: list[str] = []
    if pre_ref is None:
        warnings.append(f"Missing `{PRE_LABEL}` candidate under the selected context.")
    if not post_refs:
        warnings.append("No post-crossing candidate bundles were found under the selected context.")

    pre_candidate = (
        summarize_candidate(
            pre_ref,
            load_dock_routing_json(
                repo=repo,
                fretboard_bin=args.fretboard_bin,
                bundle_dir=pre_ref.bundle_dir,
                warmup_frames=args.warmup_frames,
            ),
        )
        if pre_ref is not None
        else None
    )
    post_candidates = [
        summarize_candidate(
            ref,
            load_dock_routing_json(
                repo=repo,
                fretboard_bin=args.fretboard_bin,
                bundle_dir=ref.bundle_dir,
                warmup_frames=args.warmup_frames,
            ),
        )
        for ref in post_refs
    ]
    selected_post = choose_post_candidate(post_candidates)

    if selected_post is None:
        warnings.append("Acceptance cannot be evaluated because no post-crossing candidate was summarized.")
    elif not selected_post.mixed_dpi_signal_observed:
        warnings.append(
            "The selected post-crossing bundle does not report `mixed_dpi_signal_observed: true`."
        )
    elif selected_post.distinct_scale_factor_count < 2:
        warnings.append(
            "The selected post-crossing bundle does not expose at least two distinct scale factors."
        )
    if len(groups) > 1:
        warnings.append(
            f"Found {len(groups)} candidate contexts; selected the latest complete context automatically."
        )

    acceptance_ready = bool(
        selected_post
        and selected_post.mixed_dpi_signal_observed
        and selected_post.distinct_scale_factor_count >= 2
    )

    payload = {
        "schema_version": SCHEMA_VERSION,
        "input_root": input_root.as_posix(),
        "search_root": search_root.as_posix(),
        "contexts_found": len(groups),
        "selected_context": selected_context.as_posix(),
        "selected_pre_crossing": pre_candidate.to_json() if pre_candidate else None,
        "selected_post_crossing": selected_post.to_json() if selected_post else None,
        "acceptance_ready": acceptance_ready,
        "candidates": (
            ([pre_candidate.to_json()] if pre_candidate is not None else [])
            + [candidate.to_json() for candidate in post_candidates]
        ),
        "warnings": warnings,
    }

    print(
        render_summary(
            input_path=input_root,
            search_root=search_root,
            contexts_found=len(groups),
            selected_context=selected_context,
            pre_candidate=pre_candidate,
            post_candidates=post_candidates,
            selected_post=selected_post,
            warnings=warnings,
        )
    )

    if args.json_out:
        write_json(Path(args.json_out).expanduser().resolve(), payload)

    if args.note_out:
        note_text = render_note_markdown(
            note_date=args.note_date,
            input_path=input_root,
            search_root=search_root,
            selected_context=selected_context,
            pre_candidate=pre_candidate,
            post_candidates=post_candidates,
            selected_post=selected_post,
            acceptance_ready=acceptance_ready,
            canonical_command=args.canonical_command,
            windows_version=args.windows_version,
            monitor_arrangement=args.monitor_arrangement,
            scale_factors_used=args.scale_factors_used,
            automation_followup=args.automation_followup,
        )
        write_text(Path(args.note_out).expanduser().resolve(), note_text)

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
