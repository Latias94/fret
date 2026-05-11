---
title: Shadcn Parity Discovery Seventh Sweep Audit
status: active
date: 2026-05-11
scope: shadcn parity discovery, input otp, slot geometry harness
---

# Seventh Sweep Audit

This audit records the Input OTP docs-demo static slice from the v2 sweep. The slice did not find a
new component mismatch, but it promoted a high-risk input surface into the fixture-driven suite and
recorded a diagnostics evidence-source constraint.

## Objective Criteria

The slice required:

1. Capture the Input OTP docs demo with layout sidecar, bundle, and screenshot evidence.
2. Compare stable shadcn `new-york-v4` slot geometry facts against the upstream DOM snapshot.
3. Classify every non-passing result by layer.
4. Promote the slice into the v2 suite as a reusable regression gate.

## Findings

### No new Input OTP geometry mismatch

- Report:
  `docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/input_otp_demo_mismatch_report_v2.json`
- Result:
  `1 pass_known` part, `4 pass_known` recipe checks, `0 mismatch`, `0 blocked`, `0 top findings`.
- Locked geometry:
  36px visible slots, contiguous 36px intra-group offsets, 76px cross-group separator offset, and
  hidden input 36px height/top alignment.
- Owner and layer:
  `component_recipe` / `recipe`.

### Layout sidecar selector blind spot

- The captured `layout.taffy.v1.json` had the expected taffy tree but did not expose the stable
  `ui-gallery-input-otp-simple.*` test ids.
- The same capture's `bundle.schema2.json` did expose those semantics test ids with global bounds.
- Classification:
  diagnostics/evidence-source constraint, not an Input OTP mismatch.
- Gate choice:
  the fixture explicitly requests `bundle_schema2_semantics` for Fret predicates.

## Evidence

- Diag script:
  `tools/diag-scripts/ui-gallery/input/ui-gallery-input-otp-demo-layout.json`
- Layout sidecar:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-input-otp-demo-layout/sessions/1778493012437-151896/1778493018878-ui-gallery-input-otp-demo.layout/layout.taffy.v1.json`
- AI packet:
  `target/fret-diag/shadcn-parity-discovery-harness-v2-input-otp-demo-layout/sessions/1778493012437-151896/1778493017440/ai.packet`
- Fixture:
  `tools/parity-discovery/fixtures/input_otp_demo_parts_v1.json`
- Suite:
  `tools/parity-discovery/suites/shadcn_parity_discovery_v2.json`

## Gate Result

- Diag run:
  `target/debug/fretboard-dev.exe diag run tools/diag-scripts/ui-gallery/input/ui-gallery-input-otp-demo-layout.json --dir target/fret-diag/shadcn-parity-discovery-harness-v2-input-otp-demo-layout --session-auto --pack --ai-packet --include-screenshots --launch -- target/debug/fret-ui-gallery.exe`
  passed.
- Fixture report:
  `python tools/parity-discovery/shadcn_parity_discovery.py --mapping tools/parity-discovery/fixtures/input_otp_demo_parts_v1.json --fret-layout-sidecar-dir target/fret-diag/shadcn-parity-discovery-harness-v2-input-otp-demo-layout --upstream-dom-snapshot F:/SourceCodes/Rust/fret/goldens/shadcn-web/v4/new-york-v4/input-otp-demo.json --output docs/workstreams/shadcn-parity-discovery-harness-v2/artifacts/input_otp_demo_mismatch_report_v2.json`
  generated a passing report.

## Residual Follow-Ups

- Add an interactive Input OTP focus/typing/mobile slice if the next sweep wants policy and input
  method coverage. This slice intentionally locks static docs-demo geometry first.
- Consider improving layout sidecar selector export so this fixture can eventually use
  `layout_sidecar` evidence directly instead of semantics fallback.
