---
title: Mobile Contracts (v1)
status: draft
date: 2026-02-12
scope: contract-first mobile readiness (Android-first, iOS follow-up)
---

# Mobile Contracts (v1) — Workstream

This workstream exists to lock the **hard-to-change** mobile-facing contracts early (ADR-first),
so future Android/iOS support does not require cross-repo rewrites once the component ecosystem
scales.

It complements (not replaces) the device bring-up workstream:

- Bring-up (device packaging + smoke tests): `docs/workstreams/mobile-bringup-v1.md`

## Principles

- **Mechanism vs policy** is non-negotiable (ADR 0066):
  - `crates/fret-ui`: mechanism (routing, capture, layout, scroll primitives, environment queries).
  - `ecosystem/*`: interaction policy (gesture arena, keyboard avoidance helpers, hover intent, etc.).
  - runner/platform crates: platform glue (IME, insets, lifecycle, pickers).
- Prefer **portable handles** over platform paths/URIs (ADR 0053), especially for sandboxed mobile.
- Prefer **snapshot + query** seams over “callback soup” so diagnostics can capture state.

## Contract map (v1)

### Text input / IME / virtual keyboard

Locked by:

- Keyboard/IME event model: `docs/adr/0012-keyboard-ime-and-text-input.md`
- Multiline composition rules: `docs/adr/0071-text-input-multiline-composition-contract.md`
- Platform TextInputClient interop seam: `docs/adr/0261-platform-text-input-client-interop-v1.md`

Key invariants we must preserve:

- Separate channels (`KeyDown` vs `TextInput` vs `ImeEvent`).
- Preedit-first arbitration while composing.
- Explicit caret anchoring (`ImeSetCursorArea`) and window-logical coordinates.
- Platform-facing indices are UTF-16 over the composed view (ADR 0261).

### Insets (safe area vs occlusion / keyboard avoidance)

Locked by:

- Environment queries + insets seam: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`

Implementation policy belongs in ecosystem crates (e.g. “scroll focused input into view”), but the
runner must provide best-effort insets commits for mobile shells.

### Lifecycle + surface recreation

Locked by:

- Mobile lifecycle + surface policy: `docs/adr/0262-mobile-lifecycle-and-surface-policy-v1.md`

Key invariants:

- Suspend stops presenting and drops surfaces best-effort.
- Resume recreates surfaces eagerly and requests redraw so the first post-resume frame presents.
- Surface acquire failures recover deterministically (Lost/Outdated/Timeout) except OOM.

### Pointer & touch baseline

Locked by:

- Pointer/touch baseline: `docs/adr/0263-pointer-and-touch-semantics-baseline-v1.md`
- Supporting ADRs: `docs/adr/0150-pointer-identity-and-multi-pointer-capture.md`,
  `docs/adr/0151-multi-pointer-drag-sessions-and-routing-keys.md`,
  `docs/adr/0238-pointer-coordinate-spaces-and-element-local-mapping-v1.md`,
  `docs/adr/0243-pointer-motion-snapshots-and-move-coalescing-v1.md`,
  `docs/adr/0136-pointer-click-count-and-double-click.md`

Key invariants:

- Well-formed pointer streams with per-pointer cancel semantics.
- Hover is capability-gated and must not be assumed for the primary pointer.
- Click slop and click-count are runner-normalized signals.

## Contract backlog (next ADR candidates)

Not required for initial bring-up, but likely “hard to change” once apps depend on them:

- File picker + sandbox semantics (paths vs handles vs URIs; Android SAF / iOS UIDocumentPicker):
  `docs/adr/0264-mobile-file-picker-and-sandbox-handles-v1.md`
- Share sheet and “open in…” intent-style surfaces.
- Clipboard files / rich clipboard formats on mobile.
- Accessibility tree requirements (semantics stability; selection/composition exposure).
- Fonts and fallback strategy (CJK/emoji parity; packaging vs system discovery).
- Resource packaging strategy (rust-embed vs platform assets; large assets, streaming, updates).

## Tracking

- TODO list: `docs/workstreams/mobile-contracts-v1/todo.md`
- Milestones: `docs/workstreams/mobile-contracts-v1/milestones.md`
