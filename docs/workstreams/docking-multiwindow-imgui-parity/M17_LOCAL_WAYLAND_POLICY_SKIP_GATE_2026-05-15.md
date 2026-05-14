# M17 Local Wayland Policy-Skip Gate - 2026-05-15

Status: local policy-skip gate; no Wayland acceptance claim.

This note records the local, non-interactive gate for the Wayland degradation campaign admission
path. The goal is narrow: a non-Wayland platform-capabilities sidecar must stop
`imui-p3-wayland-real-host` at environment admission, emit policy-skip evidence, and never execute
the Wayland degradation script. This does not close `DW-P1-linux-003`; only the real Linux Wayland
compositor run in `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md` can do that.

## Assumptions-First Resume

1. Confident: `imui-p3-wayland-real-host` remains the canonical Wayland acceptance wrapper.
   Evidence: `WORKSTREAM.json`, `M5_WAYLAND_COMPOSITOR_ACCEPTANCE_RUNBOOK_2026-04-21.md`, and the
   campaign manifest all point to the same campaign id. If wrong, this gate should move to the new
   campaign owner instead of duplicating policy checks.
2. Confident: local non-Linux maintenance should prove admission behavior, not acceptance.
   Evidence: `M11_LOCAL_NON_LINUX_CONTINUATION_BOUNDARY_2026-04-29.md` and
   `M15_LOCAL_WAYLAND_BOUNDARY_REFRESH_2026-05-14.md` both keep real-host acceptance open. If wrong,
   a non-Wayland host could accidentally bless the degradation path.
3. Likely: the right local regression surface is a runtime gate rather than another source marker.
   Evidence: `M16_SOURCE_DRIFT_GUARD_2026-05-14.md` already parses the campaign and script sources;
   the remaining local risk is report interpretation after `diag campaign run`. If wrong, the gate
   can be folded back into source-only checks.
4. Confident: no Fret UI or docking runtime API should change for this slice.
   Evidence: the existing campaign admission implementation already emits `skipped_policy` and
   `check.environment.json`; this slice only makes that behavior repeatable. If wrong, the owning
   diagnostics contract lane should be reopened as a narrow follow-on.

## What Changed

- Added `tools/diag_gate_docking_wayland_policy_skip.py`.
- The gate writes a local probe directory under
  `target/fret-diag/docking-multiwindow-imgui-parity/wayland-policy-skip-local/`.
- The probe sidecars include:
  - `capabilities.json` with `diag.script_v2`, so capability preflight passes.
  - `environment.sources.json` publishing `platform.capabilities`.
  - `environment.source.platform.capabilities.json` simulating a Windows host with
    `ui.multi_window=true`, `ui.window_tear_off=true`, `ui.window_hover_detection=platform_win32`,
    and `ui.window_z_level=reliable`.
- The gate runs `diag campaign run imui-p3-wayland-real-host --json` without `--launch`, then accepts
  the non-zero command outcome only when the JSON report proves an environment policy skip.

## Guarded Invariants

- `capabilities_check_path` stays null because `diag.script_v2` is available.
- The campaign status is `skipped_policy`, with `reason_code` equal to
  `environment.requirement_unsatisfied`.
- `check.environment.json` is present, uses `acquisition=existing_filesystem`, and records
  `environment.platform_capabilities.platform_ne`.
- Campaign counters keep `campaigns_skipped_policy=1`, `items_failed=0`, and `scripts_total=1`.
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
- `python tools/diag_gate_docking_wayland_policy_skip.py --reuse-built` passed and produced
  `check.environment.json` with `environment.platform_capabilities.platform_ne`.
- `python tools/diag_gate_docking_wayland_policy_skip.py` passed through `cargo run -p
  fretboard-dev`, proving the committed diagnostics path preserves the same policy-skip behavior.

## Verdict

This is a local policy-skip proof, not platform acceptance. It prevents non-Wayland hosts from
misreporting the Wayland degradation campaign as accepted, while keeping the true closure path on
the Linux Wayland compositor runbook.
