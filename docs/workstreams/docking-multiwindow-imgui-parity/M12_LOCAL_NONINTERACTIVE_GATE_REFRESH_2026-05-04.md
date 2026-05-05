# M12 Local Non-Interactive Gate Refresh - 2026-05-04

Status: local gate refresh; not a full hand-feel closeout.

This note records the local continuation allowed by
`M11_LOCAL_NON_LINUX_CONTINUATION_BOUNDARY_2026-04-29.md`: source-policy checks, campaign manifest
validation, and non-GUI behavior gates. It does not claim real-host OS-window hand-feel closure.

## What Was Verified

- Source-policy gate:
  - `python tools/gate_imui_workstream_source.py`
- Campaign manifest validation:
  - `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-multiwindow-parity.json --json`
  - `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-mixed-dpi-real-host.json --json`
  - `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-wayland-real-host.json --json`
  - `cargo run -p fretboard-dev -- diag campaign validate tools/diag-campaigns/imui-p3-windows-placement-real-host.json --json`
- Non-GUI behavior gates:
  - `cargo nextest run -p fret-docking --lib request_float_degrades_to_in_window_when_window_hover_detection_is_none --no-fail-fast`
  - `cargo nextest run -p fret-runtime -E 'test(opacity_request_degrades_when_unsupported) or test(opacity_request_records_when_supported) or test(capability_key_kind_matches_platform_capabilities_accessors)' --no-fail-fast --jobs 2`
  - `cargo nextest run -p fret-diag-protocol --test script_json_roundtrip script_v1_roundtrip_window_style_effective_hit_test --no-fail-fast --jobs 2`
  - `cargo nextest run -p fret-bootstrap --features ui-app-driver,diagnostics -E 'test(window_style_effective_matches_opacity_alpha)' --no-fail-fast --jobs 2`
  - `cargo nextest run -p fret-launch --lib linux_windowing_capability_posture --no-fail-fast`

All listed source-policy, manifest, and non-GUI behavior gates passed.

## Bounded Campaign Attempt

Attempted:

```bash
cargo run -p fretboard-dev -- diag campaign run imui-p3-multiwindow-parity --launch -- cargo run -p fret-demo --bin docking_arbitration_demo --release
```

Result:

- The command timed out locally after 6 minutes.
- The run did not reach script-result completion; it only created the campaign manifest and first
  script launch/config files under
  `target/fret-diag/campaigns/imui-p3-multiwindow-parity/1777897656531/`.
- Residual `cargo`, `fretboard-dev`, `cargo-nextest`, and `rustc` processes from this attempt were
  explicitly stopped.

This is not counted as a passing launched campaign.

## Verdict

- No source-policy or campaign-manifest drift was found.
- The Wayland fallback, window-style capability, script roundtrip, and diagnostics predicate gates
  remain green.
- Full docking multi-window hand-feel is still not closed by this note. It still needs a completed
  launched bounded campaign and/or platform-specific real-host acceptance evidence.
