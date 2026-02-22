---
name: fret-material-source-alignment
description: "This skill should be used when the user asks to \"align Material 3 components\", \"port Material 3 (Expressive)\", \"match MUI/Compose Material behavior\", or \"build a Material design-system layer\" for Fret. It provides a source-of-truth workflow (Material spec + MUI + Compose Material3 + Base UI) that maps changes to the correct Fret layer and locks outcomes with targeted tests and `fretboard diag` scripted repros."
---

# Material 3 (Expressive) source alignment

## When to use

- A Material-ish component does not match expected Material 3 behavior (states, focus, dismiss, keyboard nav, motion, density).
- You want to port a Material 3 (or “Material 3 Expressive”) component recipe into Fret without leaking policy into `crates/*`.
- You need a repeatable parity workflow: upstream reference → Fret layer mapping → regression gate.

## Choose this vs adjacent skills

- Use this skill when the goal is **Material parity** (spec-driven outcomes) plus a regression gate.
- Use `fret-shadcn-source-alignment` for shadcn/ui v4 + Radix parity work.
- Use `fret-app-ui-builder` when the goal is “ship a good UI” (not strict upstream parity).
- Use `fret-diag-workflow` when the primary deliverable is a scripted repro/bundle (parity work is secondary).
- Use `fret-ui-review` when the request is an audit of an app UI’s layering/UX hygiene.

## Inputs to collect (ask the user)

- Which component(s)? (e.g. button, checkbox/switch, text field, menu, dialog, snackbar, tabs, nav drawer)
- What mismatch class?
  - interaction policy (dismiss/focus restore/keyboard nav)
  - layout/density/sizing
  - tokens (colors/typography/shape/elevation/state layers)
  - motion (durations/easing/springs, interruption rules)
  - accessibility semantics (roles, names, focus order)
- What is the upstream source of truth for this case?
  - Material 3 guidelines/spec, MUI behavior, Compose Material3 behavior, and/or Base UI headless patterns
- Which Fret layer should own the change (mechanism vs policy vs recipe)?
- What regression protection is required: unit test, parity harness case, and/or diag script?
- Do we need stable `test_id` targets for automation?

Defaults if unclear:

- Treat Material spec as the primary UX truth; treat platform-specific conventions as secondary constraints.
- Treat policy/state machines as `ecosystem/*` unless it is a true mechanism/contract.
- Add at least one gate (test or diag script) for any interaction/motion change.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin components_gallery`

## Quick start

1. Decide the source of truth (spec vs MUI vs Compose) before touching code.
2. Map the mismatch to the correct layer (mechanism vs policy vs recipe).
3. Land a gate (test and/or `tools/diag-scripts/*.json`) with stable `test_id`.

## Workflow

### 1) Pick upstream references (source of truth)

Use upstream docs first; use source code when details matter:

1. Material Design 3 guidelines/spec (UX + tokens + motion intent).
2. Compose Material3 source (good reference for non-DOM state machines + semantics).
3. MUI Material UI source (good reference for web interaction details + defaults).
4. Base UI source (headless accessibility-first part composition).

Local snapshots under `repo-ref/` are optional convenience only (do not make the workflow depend on them).

### 2) Map the work to the right Fret layer (non-negotiable)

- `crates/fret-ui` (and other `crates/*`): mechanisms/contracts (focus primitives, semantics routing, overlay roots, layout and hit-testing primitives).
- `ecosystem/fret-ui-kit`: headless policy + reusable infra (state machines, roving/typeahead, overlay dismiss/focus restore rules, motion drivers).
- **Recommended**: add/extend a Material recipe layer under `ecosystem/` (e.g. `ecosystem/fret-ui-material3`) to host:
  - Material 3 tokens mapping to Fret theme keys,
  - component recipes (composition + styling),
  - stable `test_id` surfaces for automation.

If the mismatch is “interaction policy” (dismiss rules, focus restore, keyboard nav rules), it almost never belongs in `crates/fret-ui`.

### 3) Translate “DOM/Compose assumptions” into GPU-first outcomes

Material upstreams often assume a platform UI toolkit runtime. In Fret:

- Treat upstream APIs as **spec**, not an implementation to port 1:1.
- Make hit-testing semantics explicit (visual-only transform vs interactive transform).
- Keep motion tunable via theme tokens (durations/easing/spring params), not hard-coded numbers.
- For ripple/state-layer/elevation:
  - prefer a policy primitive that can target multiple render backends,
  - only add new renderer mechanisms if the effect cannot be expressed otherwise.

### 4) Lock behavior with regression gates (goldens are not enough)

Pick the smallest gate that locks the invariant:

- Deterministic logic/invariants → unit tests near the owning component/policy primitive.
- Interaction state machines (dismiss/focus restore/keyboard nav/motion interruptions) → `tools/diag-scripts/*.json` + `fretboard diag run` with stable `test_id`.
- Motion-sensitive work → run diag scripts with a fixed timestep (`--fixed-frame-delta-ms 16`) so results are deterministic.

Always leave the 3-pack:

- Repro (smallest surface),
- Gate (test/script),
- Evidence (anchors + exact commands).

### 5) Evidence anchors (make review fast)

Record 1–3 anchors in the change/PR:

- upstream reference(s) (docs/source),
- in-tree owner path(s) + key symbol(s),
- test/script path(s) + command(s).

## Evidence anchors

- Layering and contracts: `docs/architecture.md`, `docs/runtime-contract-matrix.md`
- Diagnostics + scripts: `tools/diag-scripts/`, `docs/ui-diagnostics-and-scripted-tests.md`
- Policy primitives (overlays/focus/motion): `ecosystem/fret-ui-kit/src/`
- Gallery harness: `apps/fret-examples/src/components_gallery.rs`

## Examples

- Example: align a Material 3 component interaction
  - User says: "Make this switch/ripple/state-layer match Material 3."
  - Actions: pick a source of truth (spec/MUI/Compose), map to the correct Fret layer, then lock with a diag script.
  - Result: parity improvement with regression protection.

## Common pitfalls

- Porting Material policy into `crates/*` (wrong layer, hard-to-change).
- Hard-coding numbers (durations/radius/elevation) instead of tokenizing them.
- Relying on screenshots/goldens for state machines (add a diag script).
- Missing stable `test_id`, leading to flaky/rotting scripts.
- Mixing Material and shadcn taxonomies in one recipe layer (keep design systems separate).

## Troubleshooting

- Symptom: spec and implementation references disagree.
  - Fix: document the chosen source-of-truth ordering and add evidence anchors for the decision.
- Symptom: visual parity is hard to assess.
  - Fix: prefer behavior/semantics gates first; add screenshots only where they add signal.

## Related skills

- `fret-skills-playbook`
- `fret-diag-workflow`
- `fret-ui-review`
- `fret-app-ui-builder`
- `fret-shadcn-source-alignment`
