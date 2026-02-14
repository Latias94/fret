---
title: Mobile Share + Clipboard (v1) — TODO
status: draft
date: 2026-02-12
---

# Mobile Share + Clipboard (v1) — TODO

Workstream entry:

- `docs/workstreams/mobile-share-and-clipboard-v1/design.md`

## ADR refinement

- [ ] Review ADR 0265 for alignment with existing token-based read patterns (file dialog / external drop).
- [ ] Review ADR 0266 for consistency with the existing clipboard effect surfaces (ADR 0041).
- [ ] Decide whether share/open-in needs a new token type or can reuse `FileDialogToken` semantics (naming vs reuse tradeoff).

## Implementation (future; not required for mobile bring-up)

- [x] Implement share sheet effect plumbing for at least one runner (desktop or web) behind capabilities gating.
  - Web/WASM maps `Effect::ShareSheetShow` to `navigator.share` (best-effort): `crates/fret-launch/src/runner/web/effects.rs`
  - Web clamps capability via runtime detection: `crates/fret-launch/src/runner/web/mod.rs`
  - Web/WASM supports `ShareItem::Bytes` via Web Share Level 2 `files` (best-effort): `crates/fret-launch/src/runner/web/effects.rs`
- [x] Implement incoming-open token plumbing (surface “open with…” requests to app code) with bounded reads + explicit release.
  - Currently diag-only injection (OS plumbing still pending): `crates/fret-runtime/src/effect.rs`, `crates/fret-launch/src/runner/{desktop,web}/*`
- [ ] Extend `ClipboardCapabilities` if needed to expose mobile-specific read constraints (only if proven necessary).

## Diagnostics & gates

- [x] Add a scripted diag that exercises “paste request fails gracefully” (mobile privacy model simulation):
  - `tools/diag-scripts/ui-gallery-clipboard-paste-unavailable.json`
- [x] Add a scripted diag that exercises an incoming-open request (simulated token + bounded read):
  - `tools/diag-scripts/ui-gallery-incoming-open-inject-smoke.json`
