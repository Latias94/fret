# A11y semantics closure (v1)

Status: In progress (pressed closed; required/invalid + busy queued)

Last updated: 2026-02-23

## Motivation

Fret already has a solid portable semantics skeleton (roles, boolean flags, relations, actions), but historically some
high-ROI widget semantics have been “string-only” or “component-local”. This workstream aims to close the next set of
**mechanismizable** (portable + structured + adapter-mappable) semantics so:

- platform accessibility adapters (AccessKit) can emit native semantics consistently,
- UI automation/diagnostics can assert behavior without parsing strings,
- ecosystem components (shadcn/Radix-aligned) stop reinventing per-widget conventions.

This is deliberately **contract-driven**: every semantic surface must be implemented end-to-end and guarded.

## Layer ownership

- `crates/fret-core`: portable contract surfaces (roles/flags/extras/actions + validation invariants).
- `crates/fret-ui`: snapshot production plumbing + host widget writers + runtime gating.
- `crates/fret-a11y-accesskit`: mapping + action decoding.
- `ecosystem/*`: policy + component adoption (shadcn first) + gates (snapshots / diag scripts).

## Closure checklist (definition of done)

A semantic surface is considered “closed” only when all boxes are checked:

1. **Contract**: portable field exists and is documented (ADR when it changes a hard-to-change surface).
2. **Adapter mapping**: AccessKit bridge emits it (or explicitly documents non-support).
3. **Production**: at least one real widget publishes it (shadcn/Radix-aligned preferred).
4. **Diagnostics**: snapshots/bundles include it (and fingerprints consider it when relevant).
5. **Gates**: a stable test exists (snapshot test and/or diag script).

## Inventory (v1)

### Already closed (landed + gated)

- Numeric/range semantics: `SemanticsNodeExtra.numeric` + invariants + AccessKit mapping + shadcn adoption (ADR 0288).
- Scroll semantics: `SemanticsNodeExtra.scroll` + `scroll_by` action + AccessKit mapping + shadcn gates (via viewport role).
- Orientation semantics: `SemanticsNodeExtra.orientation` + AccessKit mapping + shadcn slider/progress adoption.
- Tri-state checked semantics: `SemanticsFlags.checked_state` + AccessKit mapping + shadcn checkbox indeterminate gate
  (ADR 0289).
- Pressed/toggle-button semantics: `SemanticsFlags.pressed_state` + AccessKit mapping + shadcn toggle adoption
  (ADR 0290).
- Viewport semantics for scroll containers: `SemanticsRole::Viewport` mapping.

### Next P0 candidates (high ROI, low policy surface)

These are common across apps/editors and map directly into platform APIs:

1. **Required/invalid semantics** (forms and validation)
   - Goal: allow AT and automation to reason about validation state without parsing text.
2. **Busy/loading semantics**
   - Goal: mark regions/widgets as busy during async loads, decoupled from progress text.

### P1 candidates (valuable, but may require more policy decisions)

- Heading semantics (role + `level` adoption policy).
- More complete text field semantics (autofill hints, input purpose, spellcheck, etc.) depending on adapter support.
- Live region announcements beyond the current extras (needs explicit contract boundaries).

## Work plan (suggested sequence)

1. Close **pressed** first (smallest cross-cutting surface; easy shadcn adoption).
2. Close **required/invalid** next (immediately useful for real apps; easy gating).
3. Close **busy** next (ties into progress/async and reduces string-only “Loading…” patterns).
