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
- Which authoring surface is drifting: `fret` app-facing snippets, direct `fret_ui_shadcn` usage, or internal recipe code?
- Is this actually a public-surface drift (upstream prop-driven API vs Fret model-only authoring surface)?
- Is this actually a conversion-surface drift (`Ui` / `UiChild` / unified component conversion trait) rather than a widget recipe mismatch?
- Which layer should own the change (mechanism vs policy vs recipe)?
- What regression protection is required: unit test, parity harness case, and/or diag script?
- Do we need a new stable `test_id` surface for automation?
- What platforms and input types must match (native/web; mouse/touch/pen)?
- Does parity include accessibility outcomes?
- Does the component rely on responsive breakpoints or container queries?

Defaults if unclear:

- Treat interaction semantics as Radix truth.
- Treat composition/sizing/tokens as shadcn truth.
- Treat first-party UI Gallery snippets as the in-tree exemplar surface when the mismatch is about how Fret code should be authored or taught.
- Treat default-style ownership as a first-class decision: keep recipe defaults only for intrinsic component chrome/slot spacing, and keep page/container negotiation (`w-full`, `min-w-0`, `max-w-*`, `flex-1`, centering) caller-owned unless upstream puts it in the component source itself.
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

Default-style ownership check before changing a recipe:

- If upstream puts the class on the *example call site* (`<Card className="w-full max-w-sm">`), keep it caller-owned in Fret too.
- If upstream puts the class on the *component source* itself (for example card radius/border/shadow/slot padding), it is a recipe default.
- Do not bake layout-negotiation defaults into the recipe root just because one gallery page needs them; fix the page/grid/flex container first.

## Smallest starting point (one command)

- `cargo run -p fretboard -- dev native --bin components_gallery`

## Quick start

1. Identify whether the mismatch is layout policy, mechanism, or public-surface parity before touching code.
2. Decide which Fret authoring surface is the target (`fret` facade vs direct `fret_ui_shadcn` vs recipe internals) before copying imports or helper patterns.
3. Compare against upstream docs/source (shadcn for composition + sizing; Radix for semantics).
4. If app code is paying for per-row `Model<T>` or surrogate buttons just to keep the intended widget, run `references/public-surface-parity.md` before widening helpers.
5. Land a gate: a small invariant test and/or a `tools/diag-scripts/*.json` scripted repro with stable `test_id`.
6. Compare against a mature in-tree exemplar when available.

## Workflow

### 0) Read the right reference note first

Use these notes to keep the main skill lean:

Before doing token tweaks or adding goldens, consult:

- `.agents/skills/fret-shadcn-source-alignment/references/layout-parity-footguns.md`
- `.agents/skills/fret-shadcn-source-alignment/references/ui-gallery-exemplar-and-evidence.md`

### 0.25) Decide the Fret authoring surface before copying patterns

Do not mix the repo's app-facing `fret` facade guidance with direct-crate `fret_ui_shadcn` guidance.
Check the intended surface first:

- App-facing samples and starter docs should align with the current `fret` facade guidance in `docs/crate-usage-guide.md`.
- First-party direct-crate shadcn examples should prefer:
  - `use fret_ui_shadcn::{facade as shadcn, prelude::*};`
- Raw escape hatches should stay explicit:
  - `shadcn::raw::*`
- Canonical declarative shadcn migration status and authoring golden path live in:
  - `docs/shadcn-declarative-progress.md`

If the mismatch is “our example code teaches the wrong import/build pattern”, fix the exemplar surface
first, then the component internals if they still block the intended authoring flow.

If the mismatch is really about helper return types or explicit conversion trait names showing up
in curated examples, consult:

- `docs/workstreams/into-element-surface-fearless-refactor-v1/DESIGN.md`
- `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`

Current target:

- app-facing teaching prefers `Ui` / `UiChild`,
- reusable generic helpers should converge on one public component conversion trait,
- advanced/manual-assembly reusable helpers should use `IntoUiElement<H>` (for example
  `IntoUiElement<KernelApp>`) rather than child-pipeline traits,
- `AnyElement` stays explicit only for raw/advanced seams.

Do not expand first-party shadcn teaching surfaces by reintroducing `UiIntoElement`,
`UiChildIntoElement`, `UiHostBoundIntoElement`, or `UiBuilderHostBoundIntoElementExt` unless the
task is explicitly about migrating them away.

Current implementation status to remember while aligning examples:

- `UiHostBoundIntoElement` and `UiBuilderHostBoundIntoElementExt` are already deleted from code.
- `UiChildIntoElement` still exists only as the thin heterogeneous-child bridge.
- If a first-party snippet/helper still teaches those names, the problem is authoring-surface drift,
  not shadcn parity success.

### 0.5) Audit public surface parity before inventing helpers

When the app authoring surface feels heavier than upstream, the problem may not be layout or
mechanism at all — it may be a **public-surface drift** where a prop-driven shadcn/Radix widget
was ported as a model-only Fret authoring surface.

Run this check before adding app-side helpers or broad `IntoModel<T>` conversions:

- `.agents/skills/fret-shadcn-source-alignment/references/public-surface-parity.md`

If the complaint is "this snippet/helper still has to spell legacy conversion trait names", treat
that as conversion-surface drift first, not widget parity.

### 0) Run the mechanism checklist first (don’t chase pixels yet)

When shadcn “looks almost right”, the remaining drift is usually **mechanism** (overlay routing,
dismissal/focus, hit-testing, breakpoints), not styling. Before adding/adjusting web goldens, run:

- `.agents/skills/fret-shadcn-source-alignment/references/mechanism-parity-checklist.md`
- `.agents/skills/fret-shadcn-source-alignment/references/style-parity-checklist.md`
- Reference stack, renderer translation, and semantic-conflict notes:
  - `references/reference-stack-and-renderer-notes.md`
- A11y, responsive drivers, and gate strategy:
  - `references/a11y-responsive-and-gates.md`

### 1) Map the mismatch to the right layer

- `crates/fret-ui`: mechanisms/contracts
- `ecosystem/fret-ui-kit`: headless policy + reusable infra
- `ecosystem/fret-ui-shadcn`: shadcn v4 taxonomy + recipes
- `apps/fret-ui-gallery`: first-party exemplar + diagnostics-friendly teaching surface

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

### 3.5) Capture evidence before tweaking recipes/tokens

When a shadcn page or gallery sample looks wrong, follow this order:

1. Check the UI Gallery exemplar first:
   - snippet file = compiled preview + copyable code tab
   - page/driver glue should stay thin and avoid re-teaching alternative imports
2. Lock or add stable `test_id` hooks before writing automation.
3. Use deterministic geometry/layout evidence before screenshot churn:
   - in-tree geometry assertions (`apps/fret-ui-gallery/src/driver/render_flow.rs`)
   - `capture_layout_sidecar` when the dispute is layout-tree ownership or slot sizing
4. Add `capture_screenshot` when visual chrome or clipping needs proof.
5. Add `capture_bundle` for the interaction/state-machine record that survives refactors.

Do not jump straight to token edits when the real drift is missing caller-owned width, flex, or
overflow constraints.

### 4) Lock the change with the smallest gate

- unit tests for deterministic logic/invariants
- geometry/chrome assertions for layout/style outcomes
- `tools/diag-scripts/*.json` for interaction state machines and resize/dismiss flows
- `capture_layout_sidecar` when you need to prove layout-tree structure or size negotiation
- `capture_screenshot` when human-reviewable visual evidence is part of parity
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
- Reference stack (APG/Radix/Floating/cmdk): `docs/reference-stack-ui-behavior.md`
- Crate/layer usage map: `docs/crate-usage-guide.md`
- Shadcn parity tracker (canonical; treat older audits as historical): `docs/shadcn-declarative-progress.md`
- Mechanism checklist (this skill): `.agents/skills/fret-shadcn-source-alignment/references/mechanism-parity-checklist.md`
- Style checklist (this skill): `.agents/skills/fret-shadcn-source-alignment/references/style-parity-checklist.md`
- Layout footguns checklist (this skill): `.agents/skills/fret-shadcn-source-alignment/references/layout-parity-footguns.md`
- Public-surface parity checklist (this skill): `.agents/skills/fret-shadcn-source-alignment/references/public-surface-parity.md`
- UI Gallery exemplar + evidence note (this skill): `.agents/skills/fret-shadcn-source-alignment/references/ui-gallery-exemplar-and-evidence.md`
- Into-element conversion cleanup: `docs/workstreams/into-element-surface-fearless-refactor-v1/DESIGN.md`, `docs/workstreams/into-element-surface-fearless-refactor-v1/TARGET_INTERFACE_STATE.md`
- Action hooks (component-owned policy): `docs/action-hooks.md`
- Overlay ADRs:
  - `docs/adr/0067-overlay-policy-architecture-dismissal-focus-portal.md`
  - `docs/adr/0069-outside-press-and-dismissable-non-modal-overlays.md`
  - `docs/adr/0068-focus-traversal-and-focus-scopes.md`
- Queries:
  - Container queries (frame-lagged layout queries): `docs/adr/0231-container-queries-and-frame-lagged-layout-queries-v1.md`
  - Environment/viewport snapshots: `docs/adr/0232-environment-queries-and-viewport-snapshots-v1.md`
- A11y acceptance checklist: `docs/a11y-acceptance-checklist.md`
- Local shadcn component implementations: `ecosystem/fret-ui-shadcn/src/`
- Policy primitives (roving/typeahead/overlays): `ecosystem/fret-ui-kit/src/primitives/`
- UI Gallery authoring policy tests: `apps/fret-ui-gallery/src/lib.rs`
- UI Gallery snippet exemplars: `apps/fret-ui-gallery/src/ui/snippets/`
- UI Gallery geometry/test-id helpers: `apps/fret-ui-gallery/src/driver/render_flow.rs`
- Diag script corpus: `tools/diag-scripts/ui-gallery/`
- Layout sidecar writer: `ecosystem/fret-bootstrap/src/ui_diagnostics/script_steps.rs`
- Responsive helpers:
  - `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`
  - `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`
- Existing web-vs-fret harness (optional, for already-covered surfaces): `ecosystem/fret-ui-shadcn/tests/`

## Examples

- Example: align a component with upstream behavior
  - User says: "Our Select/Popover differs from Radix—match the behavior."
  - Actions: choose the upstream source of truth, implement in the correct Fret layer, and lock with scripts/tests.
  - Result: parity improvement with a regression gate.

## Common pitfalls

- Fixing policy mismatches by adding runtime knobs in `crates/fret-ui` (wrong layer most of the time).
- Baking caller-owned width/overflow/flex constraints into a shadcn recipe default because a single doc page or gallery composition needed them.
- Relying on goldens alone for state-machine behavior (add a scripted repro).
- Missing stable `test_id` targets, causing scripts to rot during refactors.
- Mixing “parity work” and “new design work” without leaving any regression protection behind.
- Treating Base UI as a 1:1 “implementation port”: use it as a headless reference, then translate to Fret’s GPU-first renderer (semantics/hit-testing/focus routing).
- Porting a prop-driven upstream widget as model-only in Fret, then normalizing the resulting per-row `Model<T>` or surrogate-button boilerplate in app code.
- Deriving `Clone` on types that store `AnyElement` (move-only by contract); prefer move-only builders or store inputs (models/ids) rather than elements.

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
