---
name: fret-shadcn-source-alignment
description: "Align shadcn/ui v4 + Radix behavior and composition to Fret. Use when user says \"align shadcn\", \"parity mismatch\", \"match Radix\", \"port shadcn v4\", or reports issues like \"items-stretch\", \"w-full\", \"hit box too big\", or layout/interaction drift. Maps fixes to the correct layer (mechanism vs policy vs recipe) and locks outcomes with focused tests and `fretboard diag` scripts."
---

# Shadcn / Radix source alignment

## When to use

- A shadcn/Radix-inspired component doesn’t behave like upstream.
- You need to decide whether a fix belongs in `crates/fret-ui` vs `ecosystem/fret-ui-kit` vs `ecosystem/fret-ui-shadcn`.
- You fixed a mismatch once and want to lock it with tests and/or `fretboard diag` scripted repros.

## Choose this vs adjacent skills

- Use this skill when the goal is **upstream parity** (Radix semantics / shadcn composition) plus a regression gate.
- Use `fret-app-ui-builder` when you just need a good recipe for building UI.
- Use `fret-diag-workflow` when the main deliverable is a repro/gate for a bug.
- Use `fret-ui-review` when the request is an audit rather than a concrete parity mismatch.

## Inputs to collect (ask the user)

- Which component + mismatch class (dismiss/focus/keyboard nav/placement/style)?
- Which mechanism axis is likely involved (overlay dismissal/focus restore/hit-testing/transform/clipping/breakpoints)?
- What is the upstream source of truth (Radix docs vs shadcn composition/source)?
- Which layer should own the change (mechanism vs policy vs recipe)?
- What regression protection is required: unit test, parity harness case, and/or diag script?
- Do we need a new stable `test_id` surface for automation?
- What platforms and input types must match (native/web; mouse/touch/pen)?
- Does parity include accessibility outcomes?
- Does the component rely on responsive breakpoints or container queries?

Defaults if unclear:

- Treat interaction semantics as Radix truth.
- Treat composition/sizing/tokens as shadcn truth.
- Add at least one gate.
- When DOM-focused assumptions are involved, consult Base UI as an additional headless reference.

## Quick layout sanity check

Most “my shadcn page port looks totally different” reports come from missing constraints, not missing tokens.

Start here first:

- `references/layout-parity-footguns.md`

Common mappings:

- `w-full` / `h-full` → `.ui().w_full()` / `.ui().h_full()`
- `flex-1` → `.ui().flex_1()`
- `items-stretch` → explicit stretch on the flex container
- `min-w-0` → `.ui().min_w_0()`
- `truncate` → `.ui().truncate()`

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin components_gallery`

## Quick start

1. Decide the layer before touching code.
2. Read the relevant reference notes below.
3. Compare against upstream docs/source and a mature in-tree exemplar.
4. Leave a gate (unit test, invariant check, or diag script) with stable `test_id`.

## Workflow

### 0) Read the right reference note first

Use these notes to keep the main skill lean:

- Layout / sizing mismatches:
  - `references/layout-parity-footguns.md`
- Mechanism mismatches (dismiss/focus/placement/hit-testing):
  - `references/mechanism-parity-checklist.md`
- Style/chrome mismatches:
  - `references/style-parity-checklist.md`
- Reference stack, renderer translation, and semantic-conflict notes:
  - `references/reference-stack-and-renderer-notes.md`
- A11y, responsive drivers, and gate strategy:
  - `references/a11y-responsive-and-gates.md`

### 1) Map the mismatch to the right layer

- `crates/fret-ui`: mechanisms/contracts
- `ecosystem/fret-ui-kit`: headless policy + reusable infra
- `ecosystem/fret-ui-shadcn`: shadcn v4 taxonomy + recipes

If the mismatch is interaction policy (dismiss rules, focus restore, hover intent, menu navigation), it almost never belongs in `crates/fret-ui`.

### 2) Pick the upstream reference stack explicitly

Use the right source for the right part of parity work:

- APG → keyboard/composite widget semantics
- Radix → overlay and interaction outcomes
- Floating UI → placement vocabulary and geometry outcomes
- cmdk → command palette details
- Base UI → additional headless/accessibility reference when DOM assumptions need translation

See `references/reference-stack-and-renderer-notes.md` for the detailed mapping and renderer guidance.

### 3) Align the outcome, not just the implementation shape

- Match semantics, dismissal, focus, typeahead, and sizing outcomes first.
- Translate DOM/CSS assumptions deliberately into Fret’s GPU-first model.
- Choose viewport vs container as the single source of truth for each responsive decision.
- Verify semantics and input-modality outcomes before chasing pixel polish.

### 4) Lock the change with the smallest gate

- unit tests for deterministic logic/invariants
- geometry/chrome assertions for layout/style outcomes
- `tools/diag-scripts/*.json` for interaction state machines and resize/dismiss flows
- semantics/a11y assertions when accessibility is involved

See `references/a11y-responsive-and-gates.md` for detailed gate guidance and high-value target areas.

## Definition of done (what to leave behind)

- Minimum deliverables (3-pack): Repro, Gate, Evidence. See `fret-skills-playbook`.
- A clear layer mapping in the change.
- At least one regression artifact:
  - state-machine mismatch ⇒ `tools/diag-scripts/*.json` repro with stable `test_id`
  - layout/style mismatch ⇒ deterministic invariant test
- Evidence anchors in the PR/commit message: upstream links + in-tree owner paths + test/script paths.

## Practical gates (what to actually run)

Prefer bounded, fast gates:

- layout-only / default-policy fixes:
  - focused unit test in the component file
  - a small `tools/diag-scripts/**.json` geometry predicate gate when needed
- constrained machines:
  - prefer package/lib-level test filters over full-workspace builds

## Evidence anchors

- Layers and contracts: `docs/architecture.md`, `docs/runtime-contract-matrix.md`
- Reference stack: `docs/reference-stack-ui-behavior.md`
- Canonical shadcn parity tracker: `docs/shadcn-declarative-progress.md`
- This skill’s references:
  - `references/layout-parity-footguns.md`
  - `references/mechanism-parity-checklist.md`
  - `references/style-parity-checklist.md`
  - `references/reference-stack-and-renderer-notes.md`
  - `references/a11y-responsive-and-gates.md`
- Local implementations: `ecosystem/fret-ui-shadcn/src/`
- Policy primitives: `ecosystem/fret-ui-kit/src/primitives/`

## Examples

- Example: align a component with upstream behavior
  - User says: "Our Select/Popover differs from Radix—match the behavior."
  - Actions: choose the upstream source of truth, implement in the correct Fret layer, and lock with scripts/tests.
  - Result: parity improvement with a regression gate.

## Common pitfalls

- Fixing policy mismatches by adding runtime knobs in `crates/fret-ui`.
- Relying on goldens alone for state-machine behavior.
- Missing stable `test_id` targets, causing scripts to rot.
- Mixing parity work and new design work without leaving regression protection behind.
- Treating Base UI as a 1:1 implementation port instead of a headless reference.

## Troubleshooting

- Symptom: upstream behavior is subtle (focus/keyboard/ARIA).
  - Fix: gate semantics and interaction flows before chasing pixels.
- Symptom: a “visual” mismatch keeps reappearing.
  - Fix: make it a token- or invariant-level gate instead of another ad-hoc tweak.

## Related skills

- `fret-app-ui-builder`
- `fret-diag-workflow`
- `fret-ui-review`
- `fret-material-source-alignment`
