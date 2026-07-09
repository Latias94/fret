# ADR 0333: Docking Product Placement Layout Contract v1

Status: Accepted

Scope: headless docking layout construction in `crates/fret-core` plus app-facing reset semantics in
`ecosystem/fret-docking`.

Related:

- ADR 0013: `docs/adr/0013-docking-ops-and-persistence.md`
- ADR 0017: `docs/adr/0017-multi-window-display-and-dpi.md`
- ADR 0083: `docs/adr/0083-multi-window-degradation-policy.md`
- ADR 0155: `docs/adr/0155-docking-tab-dnd-contract.md`

## Context

Applications need a concise way to declare a product default dock layout without manually building
tabs, splits, and root nodes. This is especially important for editor-style apps that want an
Open-GPUI/ImGui-like first-run layout while still keeping docking persistence and runtime graph
mutation headless.

The API is hard to change later because it becomes a public authoring surface for app starters,
examples, and persisted-layout fallback paths. It must therefore define deterministic behavior for
stack resolution, fallbacks, duplicate placements, and invalidation.

## Decision

### 1) Product placement is a headless core contract

`DockPanelPlacement` and `DockPanelPlacementTarget` are core vocabulary, not UI widget policy.

They construct a `DockGraph` from stable `PanelKey` values only. They do not depend on `fret-ui`,
`winit`, `wgpu`, component recipes, or renderer state. The UI facade may expose convenience reset
methods, but the graph construction semantics live in `fret-core` so import/export, tests, demos,
and future non-Fret-UI hosts share the same result.

### 2) Supported targets

The v1 placement target set is intentionally small:

- `Center`
- `LeftRail`
- `RightRail`
- `BottomRail`
- `Stack { anchor, insert_index }`

Rails are product defaults, not a general layout algorithm. They map to a generated work area:
left, center, and right form the horizontal work area; bottom wraps that work area in a vertical
split when present.

### 3) Deterministic stack resolution

Stack placements resolve against the full placement list, not only prior inputs. A placement may
stack with an anchor declared later in the list.

If several unresolved placements cannot resolve after a fixed-point pass, each unresolved placement
uses its explicit fallback when the fallback resolves; otherwise it falls back to center.

Unresolved-cycle fallback is per placement. A panel that falls back during this phase must not make
another still-unresolved panel ignore its own fallback by suddenly satisfying the original cyclic
stack target.

Multiple implicit `stacked_with(anchor)` placements targeting the same anchor preserve placement
input order after the anchor. Explicit `stacked_with_at(...)` remains an absolute stack insertion.

### 4) Duplicate panel placement is last-wins

If the same `PanelKey` appears more than once in the placement list, only the last placement is
used. This avoids duplicate ownership in the generated graph and matches the public API expectation
that later product declarations override earlier defaults.

### 5) Stack insertion is stack-specific

Explicit insertion index belongs only to stack targets. The public API exposes stack-specific
constructors (`stacked_with_at`) rather than a generic mutator that silently does nothing for
center/rail targets.

Insertion indexes are absolute within the resolved tab stack and clamp to the current stack length.
Without an explicit index, a stacked panel inserts immediately after the anchor's current position.

### 6) Fractions are sanitized and local

Rail fractions are optional hints. Non-finite, zero, negative, and >= 1.0 fractions fall back to
the existing default for that rail. Split normalization remains a core graph invariant.

### 7) App-facing reset invalidation

`DockSurface::set_panel_placements(...)` resets the graph to the generated layout and invalidates
the union of old and new graph windows.

The unchanged fast path must compare both:

- the exported semantic layout, and
- the graph window set.

This prevents rootless floating-only windows from being dropped silently because persistence export
only records windows with a root.

When a reset changes the graph, viewport-layout caches for invalidated windows are cleared and
those windows are redrawn.

### 8) Descriptor-only panel lifecycle

Placement reset can mention panels before a real renderer/viewport descriptor is registered. Those
panels become descriptor-only catalog entries so snapshots can report them.

Later `register_panel(...)` or `ensure_panel(...)` must promote a descriptor-only entry to a real
panel. Duplicate errors apply only to already-real panel entries.

### 9) Breaking surface cleanup

The previous public `DockPanelPlacement` enum name for docked-vs-floating locations is retired in
favor of `DockPanelLocationKind`. The product placement builder now owns the `DockPanelPlacement`
name. This is an intentional breaking change in the docking fearless refactor lane.

## Consequences

- Product default layouts can be declared without leaking UI/runtime details.
- The generated graph remains deterministic and portable.
- UI facade reset code must treat rootless floating windows as graph state even when they are not
  represented by `DockLayout::export_layout(...)`.
- Descriptor-only catalog entries are transient placeholders, not permanent registrations.
- Future placement targets require ADR updates because they change public authoring semantics.

## Implementation Notes (Evidence)

- Core placement API and resolver:
  - `crates/fret-core/src/dock/placement.rs`
  - `DockPanelPlacement`
  - `DockPanelPlacementTarget`
  - `DockGraph::from_panel_placements`
- Split fraction normalization:
  - `crates/fret-core/src/dock/mod.rs`
  - `crates/fret-core/src/dock/mutate.rs`
- App-facing reset and descriptor lifecycle:
  - `ecosystem/fret-docking/src/dock/manager.rs`
  - `ecosystem/fret-docking/src/facade.rs`
- Re-export surface:
  - `crates/fret-core/src/lib.rs`
  - `ecosystem/fret-docking/src/lib.rs`

Focused gates:

- `crates/fret-core/src/dock/placement.rs`
  - `panel_placements_stack_with_anchor_declared_later`
  - `panel_placements_duplicate_panel_uses_last_placement`
  - `panel_placements_stack_with_explicit_insert_index`
  - `panel_placements_same_anchor_siblings_preserve_input_order`
  - `panel_placements_mutual_stack_cycle_uses_each_fallback`
  - `panel_placements_use_fallback_when_stack_anchor_is_missing`
  - `panel_placements_default_missing_stack_anchor_to_center`
- `ecosystem/fret-docking/src/dock/manager.rs`
  - `panel_catalog_promotes_descriptor_only_entries`
- `ecosystem/fret-docking/src/facade/tests.rs`
  - `dock_surface_set_panel_placements_descriptor_panels_can_be_promoted`
  - `dock_surface_set_panel_placements_resets_rootless_floating_windows`
  - `dock_surface_set_panel_placements_invalidates_replaced_windows`
  - `dock_surface_set_panel_placements_reports_unchanged_for_same_layout`
