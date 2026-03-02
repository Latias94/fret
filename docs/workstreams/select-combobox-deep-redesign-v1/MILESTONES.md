## Select + Combobox Deep Redesign v1 (Milestones)

This workstream is intentionally staged so we can land improvements without destabilizing the
ecosystem layer.

### M0 — Lock the contract we want (docs-first)

Acceptance:

- `DESIGN.md` + `TODO.md` exist and are referenced from the parent shadcn alignment tracker.
- For each component (`select`, `combobox`), we have:
  - upstream part surface mapping (file anchors),
  - an explicit “reference stack” list (APG/Radix/shadcn/Base UI),
  - a minimal gate plan (what we will test and how).

Deliverables:

- `docs/workstreams/select-combobox-deep-redesign-v1/DESIGN.md`
- `docs/workstreams/select-combobox-deep-redesign-v1/TODO.md`
- `docs/workstreams/select-combobox-deep-redesign-v1/MILESTONES.md` (this file)

### M1 — Shared headless substrate (kit)

Note: a meaningful portion of this substrate already exists today (see
`ecosystem/fret-ui-kit/src/primitives/select.rs`, `ecosystem/fret-ui-kit/src/primitives/combobox.rs`,
and `ecosystem/fret-ui-kit/src/primitives/active_descendant.rs`). M1 should be “reuse-first”:
only add new helpers when we can’t express an upstream outcome with the existing primitives.

Acceptance:

- A reusable substrate is available in `ecosystem/fret-ui-kit` that can drive both select and
  combobox without duplicating keyboard/typeahead/selection logic (either by reusing existing
  primitives or by adding a small missing helper).
- At least one unit test gates core invariants (selection model + typeahead + disabled items) at
  the substrate layer (kit or headless), not only via recipe tests.
- No `fret-ui` contract expansion for policy-only behavior.

Deliverables (example, may evolve):

- Prefer reusing:
  - `ecosystem/fret-ui-kit/src/primitives/select.rs`
  - `ecosystem/fret-ui-kit/src/primitives/combobox.rs`
  - `ecosystem/fret-ui-kit/src/primitives/active_descendant.rs`
  - `ecosystem/fret-ui-headless/src/{cmdk_selection.rs,typeahead.rs}`
- If needed, add a small missing helper in `ecosystem/fret-ui-kit/src/primitives/*` (e.g. a
  listbox active-descendant wiring helper), with unit tests in `ecosystem/fret-ui-kit`.

### M2 — Select v4 part surface parity (shadcn)

Acceptance:

- shadcn v4-named `Select*` parts are available for copy/paste ports.
- Overlay behavior outcomes match the reference stack:
  - open/close + outside press dismissal,
  - focus restore to trigger,
  - keyboard navigation and selection semantics.
- Stable `test_id` surface exists for trigger and active option(s).
- At least 2 unit tests gate the above outcomes.

Deliverables:

- `ecosystem/fret-ui-shadcn/src/select.rs` (part surface + recipe wiring)
- Focused unit tests (and optionally a diag script)

### M3 — Combobox v4 part surface parity (shadcn)

Acceptance:

- shadcn v4-named `Combobox*` parts are available and documented.
- Input + listbox interaction outcomes align to APG expectations (where applicable) and Radix-like
  overlay lifecycle outcomes.
- At least 2 unit tests gate:
  - input focus + active descendant/roving model,
  - filtering/typeahead behavior,
  - stable `test_id` surfaces.

Deliverables:

- `ecosystem/fret-ui-shadcn/src/combobox.rs` (+ related modules)
- Unit tests and/or scripted diags

### M4 — In-tree call sites migrate (gallery + snippets)

Acceptance:

- UI gallery and snippet code uses the v4 part surfaces (or explicit adapters).
- Existing demos continue to run; “legacy” surfaces are not required for new code.

Deliverables:

- `apps/fret-ui-gallery/src/ui/snippets/*` updates
- “How to port from upstream” notes (if needed)

### M5 — Deprecation / cleanup pass (optional, after stability)

Acceptance:

- Old surfaces are either removed (if in-tree only) or deprecated with clear guidance.
- The new gates provide sufficient regression protection to enable fearless refactors.
