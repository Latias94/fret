# A11y, responsive, and gate notes

Use this note when the parity drift is about semantics, input modality, breakpoints, or deciding which regression artifact to leave behind.

## 1) Pointer / hit-testing / drag / cursor parity

Check these GPU-first gotchas first:

- hit-testing follows layout bounds; transforms and clipping are explicit
- pointer capture and observer passes matter for overlays and drags
- touch is not mouse: outside-press often happens on pointer-up with slop
- cursor icons are a contract surface for resize/drag handles

Start points:

- `crates/fret-ui/src/tree/hit_test.rs`
- `crates/fret-ui/src/tree/ui_tree_outside_press.rs`
- `crates/fret-ui/src/tree/dispatch/window.rs`
- `crates/fret-core/src/cursor.rs`

## 2) A11y parity means semantics outcomes

High-signal invariants include:

- roles (`SemanticsRole`) and flags (disabled/selected/expanded/checked)
- relations (`labelled_by`, `described_by`, `controls`)
- composite widgets (`active_descendant`)
- collections (`pos_in_set` / `set_size`) for menu/listbox-like surfaces

Start points:

- `crates/fret-core/src/semantics.rs`
- `ecosystem/fret-ui-kit/src/primitives/trigger_a11y.rs`
- `docs/a11y-acceptance-checklist.md`

## 3) Responsive / breakpoint parity

When upstream uses responsive classes or container queries, prefer the in-tree helpers:

- container queries: `ecosystem/fret-ui-kit/src/declarative/container_queries.rs`
- viewport/device queries: `ecosystem/fret-ui-kit/src/declarative/viewport_queries.rs`

Always decide whether the truth is viewport-driven or container-driven before coding.

## 4) Visual / token parity

Prefer token- and vocabulary-level alignment over per-component literals:

- theme ingestion/conversion: `ecosystem/fret-ui-shadcn/src/shadcn_themes.rs`
- style vocabulary: `ecosystem/fret-ui-kit/src/style/`
- focus-visible + rings: `docs/adr/0061-focus-rings-and-focus-visible.md`
- pixel snapping: `crates/fret-ui/src/pixel_snap.rs`

## 5) Align + protect with tests

Pick the smallest gate that locks the invariant:

- unit tests for deterministic logic/invariants
- deterministic Fret-side geometry/chrome assertions for style/layout outcomes
- diag scripts for interaction state machines and resize/dismiss/focus outcomes
- semantics assertions or a11y-focused checks when accessibility is in scope

Motion token guidance:

- shadcn durations/easing numeric scale:
  - `duration.shadcn.motion.{100|200|300|500}`
  - `easing.shadcn.motion`
- semantic keys are preferred for long-term authoring
- Material 3 spring tokens stay distinct and should not be flattened carelessly into shadcn taxonomies

## 6) High-value regression targets

Start with:

- overlay families: `dropdown-menu`, `select`, `context-menu`, `tooltip`/`hover-card`, `dialog`/`sheet`, `navigation-menu`
- listbox-ish behavior: roving focus, typeahead, active-descendant semantics, scroll clamping
- responsive decisions: viewport vs container driver, hysteresis, constrained viewport max-height/scroll outcomes
