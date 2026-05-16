# M18 Local Wayland Policy-Skip Matrix Refresh - 2026-05-16

Status: local policy-skip matrix refresh; no Wayland acceptance claim.

This note records the next local, non-interactive gate for the Wayland degradation campaign
admission path. The goal stays narrow: non-qualifying `platform.capabilities` sidecars must stop
`imui-p3-wayland-real-host` at environment admission, emit policy-skip evidence, and never execute
the Wayland degradation script. This still does not close `DW-P1-linux-003`; only the real Linux
Wayland compositor run in `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md` can do that.

## Assumptions-First Resume

1. Confident: the Wayland campaign should keep one canonical real-host acceptance path.
   Evidence: `WORKSTREAM.json`, `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`, and the
   campaign manifest still point to the same campaign id. If wrong, this gate should move to the new
   campaign owner instead of duplicating policy checks.
2. Confident: local non-Wayland / non-qualifying maintenance should prove admission behavior, not acceptance.
   Evidence: `M11_LOCAL_NON_LINUX_CONTINUATION_BOUNDARY_2026-04-29.md` and
   `M15_LOCAL_WAYLAND_BOUNDARY_REFRESH_2026-05-14.md` both keep real-host acceptance open. If wrong,
   a non-qualifying host or stale capability sidecar could accidentally bless the degradation path.
3. Likely: the right local regression surface is a policy-skip matrix, not another source marker.
   Evidence: `M16_SOURCE_DRIFT_GUARD_2026-05-14.md` already parses the campaign and script sources;
   the remaining local risk is admission behavior across distinct non-qualifying sidecars. If
   wrong, this note can fold back into source-only checks.
4. Confident: no Fret UI or docking runtime API should change for this slice.
   Evidence: the existing campaign admission implementation already emits `skipped_policy` and
   `check.environment.json`; this slice only makes that behavior more explicit. If wrong, the owning
   diagnostics contract lane should be reopened as a narrow follow-on.

## What Changed

- Expanded `tools/diag_gate_docking_wayland_policy_skip.py` into a five-case matrix.
- The gate now probes:
  - a Windows sidecar that must fail on `environment.platform_capabilities.platform_ne`;
  - a Linux Wayland-style sidecar with `ui.multi_window=false` that must fail on
    `environment.platform_capabilities.ui_multi_window_ne`;
  - a Linux/X11-style sidecar that must fail on
    `environment.platform_capabilities.ui_window_tear_off_ne`;
  - a Linux Wayland-style sidecar with hover detection still `best_effort` that must fail on
    `environment.platform_capabilities.ui_window_hover_detection_ne`;
  - a Linux Wayland-style sidecar with z-level still `best_effort` that must fail on
    `environment.platform_capabilities.ui_window_z_level_ne`.
- All probes still publish `capabilities.json` with `diag.script_v2`, still stop at campaign
  admission, and still produce no script item files.

## Guarded Invariants

- `capabilities_check_path` stays null because `diag.script_v2` is available.
- Each probe run ends with `status=skipped_policy` and `reason_code=environment.requirement_unsatisfied`.
- The Windows probe records `environment.platform_capabilities.platform_ne`.
- The Linux multi-window probe records `environment.platform_capabilities.ui_multi_window_ne`.
- The Linux/X11 probe records `environment.platform_capabilities.ui_window_tear_off_ne`.
- The Linux hover-detection probe records
  `environment.platform_capabilities.ui_window_hover_detection_ne`.
- The Linux z-level probe records `environment.platform_capabilities.ui_window_z_level_ne`.
- Campaign counters keep `campaigns_skipped_policy=1`, `items_failed=0`, and `scripts_total=1` for
  each probe.
- The Wayland script item is admitted as a campaign item but not executed; script item files are not
  produced under `script-results/` or `suite-results/`.

## Commands Run

```powershell
python -m py_compile tools/diag_gate_docking_wayland_policy_skip.py
python tools/diag_gate_docking_wayland_policy_skip.py --reuse-built
python tools/diag_gate_docking_wayland_policy_skip.py
```

## Results

- `python -m py_compile tools/diag_gate_docking_wayland_policy_skip.py` passed.
- `python tools/diag_gate_docking_wayland_policy_skip.py --reuse-built` passed and produced one
  `check.environment.json` for each admission predicate mismatch:
  `environment.platform_capabilities.platform_ne`,
  `environment.platform_capabilities.ui_multi_window_ne`,
  `environment.platform_capabilities.ui_window_tear_off_ne`,
  `environment.platform_capabilities.ui_window_hover_detection_ne`, and
  `environment.platform_capabilities.ui_window_z_level_ne`.
- `python tools/diag_gate_docking_wayland_policy_skip.py` passed through `cargo run -p fretboard-dev`
  and preserved the same five policy-skip outcomes.
- 2026-05-16 follow-up: `WORKSTREAM.json` now names both the cold-start policy-skip gate and the
  `--reuse-built` drift gate, so local maintenance can separate build cost from admission behavior.

## Verdict

This is still a local policy-skip proof, not platform acceptance. It blocks non-qualifying
platform-capability sidecars from misreporting the Wayland degradation campaign as accepted, while
keeping the true closure path on the Linux Wayland compositor runbook.
