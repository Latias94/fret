# shadcn Part Surface Alignment v1 (Milestones)

This workstream is designed to be incremental and refactor-friendly: land thin wrapper parts first,
then tighten behavior and add gates.

## Milestone 0 — Audit + tracker baseline

**Deliverables**

- `DESIGN.md`, `TODO.md`, `MILESTONES.md` in this directory.
- A first-pass table listing target components and part surface gaps.

**Acceptance criteria**

- Tracker table includes at least: `card`, `avatar`, `dialog`, `sheet`, `dropdown-menu`, `menubar`.

## Milestone 1 — Provider footgun sweep (size/variant scopes)

**Scope**

- `avatar` (size scope) and any other confirmed “constructed outside provider” cases.

**Deliverables**

- Scoped builder helper(s) (e.g. `avatar_sized(...)`).
- Part-level explicit override(s) where needed.
- Unit tests locking inherited vs explicit behavior.

**Acceptance criteria**

- At least one unit test proves: default vs inherited vs explicit size produces the same spacing
  outcomes under a parent size scope.

## Milestone 2 — Overlay family part surface parity

**Scope**

- `dialog`, `alert-dialog`, `sheet` part surfaces:
  - add `Trigger`, `Portal`, `Overlay` parts as thin wrappers/adapters.

**Deliverables**

- New public parts in `ecosystem/fret-ui-shadcn` that map cleanly to upstream examples.
- At least one scripted gate per component (open/close + focus + dismissal).

**Acceptance criteria**

- UI Gallery (or a small demo) can be written in the upstream part-based shape without workarounds.
- `fretboard diag run <script>` passes for each migrated component.

## Milestone 3 — Menu family part surface parity

**Scope**

- `dropdown-menu` and `menubar`:
  - add `Trigger/Content/Portal/Sub*` parts and reconcile existing facades.

**Deliverables**

- Thin parts + adapters; keep existing API working during transition.
- Scripted gates for:
  - open/close,
  - keyboard navigation (roving focus),
  - submenu open/close,
  - outside press dismissal.

**Acceptance criteria**

- A copy/paste example from upstream can be expressed with minimal translation (layout constraints may
  still need explicit `min_w_0()` etc, but the part boundaries match).

## Milestone 4 — “Optional” surface alignment

**Scope**

- `drawer` (`DrawerTitle/DrawerDescription`)
- `carousel` wrapper parts (`Content/Item/Prev/Next`) if we decide it improves ergonomics.
- style helpers (`tabsListVariants`, `navigationMenuTriggerStyle`) only if they become useful.

**Acceptance criteria**

- Every item marked “Done” in `TODO.md` has at least one gate (unit test and/or diag script).

## Milestone 5 — Combobox v4 part surface convergence

`combobox` is the largest remaining surface gap because upstream is Base UI-rooted and the shadcn
v4 part surface is not yet represented in Rust. The prior naming conflict (`ComboboxItem` as a
data model) has been resolved by moving option structs to `ComboboxOption` /
`ComboboxOptionGroup`.

**Scope**

- Stage 1 (done): publish the option data model (`ComboboxOption`, `ComboboxOptionGroup`) and
  migrate in-tree call sites to `options(...)` + `combobox_option(...)` style construction.
- Stage 2 (workstream-scoped): introduce v4-aligned parts (`ComboboxInput/Content/List/Item/...`)
  and provide an `into_element_parts(...)` adapter over the existing Popover + Command recipe.
- Stage 3 (gates): lock at least one high-signal invariant:
  - clear button visibility rules,
  - responsive drawer vs popover switch (viewport breakpoint),
  - and a deterministic "empty state" layout.

**Acceptance criteria**

- The upstream docs “Usage” snippet shape can be expressed in Rust with a part-based API (even if
  some Tailwind constraints map to explicit `.w_full()` / `.min_w_0()` calls).

## Milestone 6 — Select v4 part surface convergence (Deferred)

`select` is currently implemented as a configuration + entries surface. This is productive, but it
is structurally different from shadcn/ui v4’s part-based composition model (Trigger/Value/Content
as real nested parts).

This milestone is intentionally **deferred** because it is higher-risk and touches a wide surface
area (focus, scroll buttons, typeahead, list virtualization constraints, and entry modeling).

**Scope**

- Stage 1: document the desired Rust part surface mirroring upstream parts (Trigger/Value/Content/
  Group/Item/Label/Separator/ScrollUpButton/ScrollDownButton), including which parts are “thin
  adapters” vs “real elements”.
- Stage 2: migrate `Select` internals to support true part composition while preserving the
  existing high-level `Select` facade as a compatibility layer.
- Stage 3: add at least one deterministic gate that fails if the part-based usage snippet cannot
  be expressed (copy/paste parity), plus keep the current unit tests as behavior baselines.

**Acceptance criteria**

- The upstream docs “Usage” snippet shape can be expressed in Rust with part-based composition.
- The existing `Select` tests still pass (or are intentionally superseded with equivalent gates).
