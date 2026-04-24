# ImUi Interaction Inspector v1

Status: closed execution lane
Last updated: 2026-04-24

Status note (2026-04-24): this lane closed after the live response inspector landed in
`imui_interaction_showcase_demo`. Use `CLOSEOUT_AUDIT_2026-04-24.md` for the shipped verdict.

## Design Brief

- Product keywords: compact, technical, editor-grade, proof-backed.
- Surface priority: product-facing response inspector inside `imui_interaction_showcase_demo`.
- Constraints: keep `fret-ui-kit::imui` and `fret-imui` public contracts unchanged; do not move
  policy into `crates/fret-ui`; keep `imui_response_signals_demo` as the proof-first contract log.
- Differentiation hook: a live inspector rail that translates one-frame IMUI response flags into
  reviewable, human-readable state without turning the whole showcase into a raw diagnostics dump.
- Baseline style: existing shadcn New York light/slate shell used by the showcase.

## Why This Lane Exists

The IMUI response vocabulary now covers click variants, press hold, drag lifecycle, activation /
deactivation / after-edit lifecycle, menu/submenu trigger edges, tab trigger edges, combo lifecycle,
and context menu requests. The proof surface intentionally exposes those as a contract log in
`imui_response_signals_demo`.

The presentable showcase is better for demos, but it currently summarizes those interactions mostly
as counters and timeline text. That makes the lane look cleaner, but it also hides why the IMUI
surface is becoming Dear ImGui-class: response flags are the product.

This lane adds one live inspector surface to the showcase.

## Assumptions First

- Area: lane ownership
  - Assumption: this is a new narrow follow-on rather than a reopening of the closed item or
    active-trigger kernel lanes.
  - Evidence: `docs/workstreams/imui-item-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md` and
    `docs/workstreams/imui-active-trigger-behavior-kernel-v1/CLOSEOUT_AUDIT_2026-04-24.md`.
  - Confidence: Confident
  - Consequence if wrong: a demo/product surface could blur closed private-kernel cleanup scope.

- Area: proof vs showcase split
  - Assumption: `imui_response_signals_demo` remains the proof/contract surface; the showcase owns
    product readability.
  - Evidence: file-level comments in `apps/fret-examples/src/imui_response_signals_demo.rs` and
    `apps/fret-examples/src/imui_interaction_showcase_demo.rs`.
  - Confidence: Confident
  - Consequence if wrong: the product surface could become a second raw contract log.

- Area: contract boundary
  - Assumption: this slice should not widen `fret-ui-kit::imui`, `fret-imui`, or `crates/fret-ui`.
  - Evidence: ADR 0066 keeps runtime APIs mechanism-only, and recent IMUI follow-ons closed without
    public API widening.
  - Confidence: Confident
  - Consequence if wrong: app-owned inspection state could accidentally become a public response
    contract before two-surface proof exists.

- Area: regression surface
  - Assumption: a source-policy test plus native build is the right first gate; a diag script can be
    a later follow-on if automation needs a stable interaction recording path.
  - Evidence: existing `apps/fret-examples/src/lib.rs` tests already guard the teaching surface
    posture for these demos.
  - Confidence: Likely
  - Consequence if wrong: we may need a dedicated `fretboard diag` script in a narrower automation
    follow-on.

## Scope

In scope:

- Add a response inspector section/card to `apps/fret-examples/src/imui_interaction_showcase_demo.rs`.
- Capture the latest meaningful IMUI response flags from the existing lab and shell controls.
- Surface level-triggered state such as hold/drag activity alongside last edge-triggered response.
- Add stable `test_id` anchors for the inspector.
- Update source-policy tests so the showcase keeps teaching the intended product-facing response
  surface.

Out of scope:

- New public `fret-imui` or `fret-ui-kit::imui` response APIs.
- Runtime changes in `crates/fret-ui`.
- A new component crate, theme preset, or shadcn recipe.
- Replacing `imui_response_signals_demo` as the proof-first contract surface.
- Full diag automation for every response flag.

## Target Shape

The showcase should have:

- one live inspector card with stable `test_id`;
- a latest-response summary showing source, event, and response flags;
- a compact level-state matrix for hold, drag, autosave, selected tab, and pinned context state;
- no new compatibility fallback or duplicate proof surface;
- source-policy markers that keep this split from drifting.
