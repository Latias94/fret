---
title: Mobile Contracts (v1) — TODO
status: draft
date: 2026-02-12
---

# Mobile Contracts (v1) — TODO

Workstream entry:

- `docs/workstreams/mobile-contracts-v1/design.md`

## ADR closure

- [ ] Review ADR 0261/0262/0263 for consistency with existing accepted ADRs (0012/0071/0232/0054).
- [ ] Add a short “Implementation status” section to each new ADR (what exists today vs gaps).
- [x] Decide whether to extend `ImeAllow` semantics further or introduce an explicit “show keyboard”
      effect for Android user-activation constraints:
      - Chosen: add `Effect::ImeRequestVirtualKeyboard` (best-effort) alongside `Effect::ImeAllow`
        (source of truth).
      - ADR: `docs/adr/0261-platform-text-input-client-interop-v1.md`

## Evidence & gates (diagnostics-first)

- [ ] Add/confirm a scripted diag scenario that simulates `OcclusionInsets.bottom > 0` and asserts a
      focused input remains visible (CI-friendly).
- [ ] Add a diag bundle snapshot field that prints:
  - committed safe-area + occlusion insets,
  - `focus_is_text_input` and composing state,
  - primary pointer type/capabilities.

## Contract backlog ADRs (draft outlines only)

- [x] Draft “Mobile file picker + sandbox handles (v1)” ADR:
  - `docs/adr/0264-mobile-file-picker-and-sandbox-handles-v1.md`
- [x] Draft “Share sheet / open-in intent (v1)” ADR:
  - `docs/adr/0265-mobile-share-sheet-and-open-in-intents-v1.md`
- [x] Draft “Mobile clipboard portability (v1)” ADR (text baseline, file/rich formats later):
  - `docs/adr/0266-mobile-clipboard-portability-v1.md`

## Cross-workstream alignment

- [x] Ensure `docs/workstreams/mobile-bringup-v1.md` references ADR 0261/0262/0263 in its contract anchors.
- [ ] Ensure `docs/workstreams/gesture-recognizers-v1.md` stays policy-only and does not redefine runtime invariants.
