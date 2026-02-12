---
name: fret-shadcn-source-alignment
description: Align Fret’s shadcn/Radix-inspired components with upstream sources (public docs + GitHub source; optional local pinned snapshots under `repo-ref/`). Also consider Base UI (unstyled, accessibility-first primitives) as a reference for part composition and headless state machines. Map changes to the correct Fret layer and lock outcomes with tests + `fretboard diag` scripts.
---

# Shadcn / Radix source alignment

## When to use

- A shadcn/Radix-inspired component “doesn’t behave like upstream” (dismiss/focus/keyboard nav/placement).
- You need to decide whether a fix belongs in `crates/fret-ui` vs `ecosystem/fret-ui-kit` vs `ecosystem/fret-ui-shadcn`.
- You fixed a mismatch once and want to lock it with tests and/or `fretboard diag` scripted repros.

## Choose this vs adjacent skills

- Use this skill when the goal is **upstream parity** (Radix semantics / shadcn composition) plus a regression gate.
- Use `fret-app-ui-builder` when you just need a good recipe for building UI (not necessarily parity work).
- Use `fret-diag-workflow` when the main deliverable is a repro/gate for a bug (and parity is secondary).
- Use `fret-ui-review` when the request is an audit of app UI code quality and layering (not a specific parity mismatch).

## Inputs to collect (ask the user)

Ask these to keep the work scoped and landable:

- Which component + mismatch class (dismiss/focus/keyboard nav/placement/style)?
- What is the upstream source of truth (Radix docs vs shadcn composition/source)?
- Which layer should own the change (mechanism vs policy vs recipe)?
- What regression protection is required: unit test, parity harness case, and/or diag script?
- Do we need a new stable `test_id` surface for automation?

Defaults if unclear:

- Treat interaction semantics as Radix truth; treat composition/sizing/tokens as shadcn truth; add at least one gate.
- When DOM-focused assumptions are involved, consult Base UI as an additional headless reference for part composition and accessibility patterns.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin components_gallery`

## Quick start

1. Identify the layer (mechanism vs policy vs recipe) before touching code.
2. Compare against upstream docs/source (shadcn for composition + sizing; Radix for semantics).
3. Land a gate: a small invariant test and/or a `tools/diag-scripts/*.json` scripted repro with stable `test_id`.

## Workflow

### 1) Map the mismatch to the right layer

- `crates/fret-ui`: mechanisms/contracts (tree/layout/semantics/focus/overlay roots).
- `ecosystem/fret-ui-kit`: headless policy + reusable infra (roving focus, typeahead, overlay policy).
- `ecosystem/fret-ui-shadcn`: shadcn v4 taxonomy + recipes (composition + styling).

If the mismatch is “interaction policy” (dismiss rules, focus restore, hover intent, menu navigation),
it almost never belongs in `crates/fret-ui`.

### 2) Find the upstream reference (source of truth)

1. Start with public docs (good enough for most alignment work):
   - shadcn components: https://ui.shadcn.com/docs/components
   - Radix primitives: https://www.radix-ui.com/primitives/docs/components
   - Base UI (headless primitives): https://base-ui.com/react/overview/quick-start
2. If you need exact implementation details, use source code:
   - shadcn/ui v4 source (New York v4 registry): https://github.com/shadcn-ui/ui/tree/main/apps/v4/registry/new-york-v4/ui
   - Radix Primitives source: https://github.com/radix-ui/primitives/tree/main/packages/react
   - Base UI source: https://github.com/mui/base-ui/tree/main/packages/react/src
   - Optional local pinned snapshots (if your checkout includes `repo-ref/`; not necessarily present on GitHub):
     - shadcn recipe: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/<component>.tsx`
     - Radix primitive: `repo-ref/primitives/packages/react/<primitive>/src/*`
     - Base UI: `repo-ref/base-ui/packages/react/src/`

Use shadcn to learn the *composition + Tailwind contracts* (sizes, spacing, tokens), and Radix to
learn the *interaction semantics* (focus, dismiss, keyboard nav, portal layering).

Use Base UI as an additional reference for:

- part-based component composition (Root/Trigger/Content patterns),
- accessibility-first defaults (labels/fields/form patterns), and
- headless state machines that must be translated to Fret’s custom renderer (semantics/hit-testing/focus routing).

### 3) Align + protect with tests (goldens are not enough)

1. Prefer fast, targeted Rust unit tests for invariants.
   - Add tests next to the component in `ecosystem/fret-ui-shadcn/src/<component>.rs` (`#[cfg(test)]`).
   - Assert relationships (centering, overlap, clamping, tab order, active descendant, etc.).
2. Add web-vs-fret parity checks when the mismatch is layout/style outcome.
   - Existing harness lives under `ecosystem/fret-ui-shadcn/tests/`.
   - Web goldens live under `goldens/shadcn-web/v4/new-york-v4/*.json`.
3. Add an interaction repro script when state machines are involved.
   - Create `tools/diag-scripts/<scenario>.json` and gate it via `fretboard diag run`.
   - Always add/keep stable `test_id` targets in the Fret UI so scripts survive refactors.

### 4) When a golden is missing

1. Add a targeted invariant test first (so you stop bleeding regressions immediately).
2. If needed, generate the missing golden later:
   - Follow `docs/shadcn-web-goldens.md` (extraction from the upstream shadcn v4 app; local `repo-ref/ui` is optional).

### 5) High-value regression targets (start here)

- Overlay families: `dropdown-menu`, `select`, `context-menu`, `tooltip`/`hover-card`, `dialog`/`sheet`, `navigation-menu`.
- Listbox-ish behavior: roving focus, typeahead, active-descendant semantics, scroll clamping in constrained viewports.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro (smallest surface), Gate (script/test/parity), Evidence (upstream refs + anchors). See `fret-skills-playbook`.
- A clear layer mapping in the change (no “policy knobs” added to `crates/fret-ui` unless it is truly a mechanism).
- At least one regression artifact:
  - **state machine** mismatch ⇒ `tools/diag-scripts/*.json` repro with stable `test_id`,
  - **layout/style** mismatch ⇒ parity harness case and/or deterministic invariant test.
- Evidence anchors in the PR/commit message: upstream link(s) + in-tree file(s) + test/script path(s).

## Evidence anchors

- Layers and contracts: `docs/architecture.md`, `docs/runtime-contract-matrix.md`
- Local shadcn component implementations: `ecosystem/fret-ui-shadcn/src/`
- Policy primitives (roving/typeahead/overlays): `ecosystem/fret-ui-kit/src/primitives/`
- Web-vs-fret harness + goldens:
  - `ecosystem/fret-ui-shadcn/tests/`
  - `goldens/shadcn-web/v4/new-york-v4/*.json`
- Optional pinned snapshots (if present):
  - `repo-ref/ui/apps/v4/registry/new-york-v4/ui/`
  - `repo-ref/primitives/packages/react/`

## Common pitfalls

- Fixing policy mismatches by adding runtime knobs in `crates/fret-ui` (wrong layer most of the time).
- Relying on goldens alone for state-machine behavior (add a scripted repro).
- Missing stable `test_id` targets, causing scripts to rot during refactors.
- Mixing “parity work” and “new design work” without leaving any regression protection behind.
- Treating Base UI as a 1:1 “implementation port”: use it as a headless reference, then translate to Fret’s GPU-first renderer (semantics/hit-testing/focus routing).

## Related skills

- `fret-app-ui-builder` (recipes + stable `test_id` conventions)
- `fret-diag-workflow` (bundles + scripted repro gates)
- `fret-ui-review` (audit lens)
