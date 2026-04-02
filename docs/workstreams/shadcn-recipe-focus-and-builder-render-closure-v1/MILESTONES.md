# shadcn recipe focus and builder render closure v1 - milestones

Tracking doc: `docs/workstreams/shadcn-recipe-focus-and-builder-render-closure-v1/DESIGN.md`

TODO board: `docs/workstreams/shadcn-recipe-focus-and-builder-render-closure-v1/TODO.md`

This file tracks the narrow execution order for the landed recipe closure and its remaining
maintenance follow-on.

## Phase A - Text-entry active chrome closure

Status: Completed

Goal:

- classify text-entry controls as a distinct recipe category,
- keep pointer-focused text fields visually active,
- and stop teaching the Todo proof surface through per-app chrome overrides.

Current landed slice:

- `Input`, `Textarea`, and `InputGroup` now drive active border/ring chrome from `focused`,
- pointer focus keeps the active editing chrome visible,
- and the Todo demo now reflects the shared recipe outcome rather than hiding drift in app code.

Deliverables:

- one explicit text-entry rule in the workstream docs,
- landed recipe changes in the three shared text-entry surfaces,
- focused unit tests plus Todo diag proof surfaces.

Exit gates:

- pointer-focused text-entry controls no longer look inactive,
- the shared recipe surfaces behave consistently,
- and the Todo demo no longer needs local compensation.

## Phase B - Builder single-render discipline closure

Status: Completed

Goal:

- remove speculative same-frame builder probe renders,
- keep local state callsites stable,
- and preserve the intended hover/focus behavior of the affected recipe surface.

Current landed slice:

- `SidebarMenuItem::into_element_with_children(...)` no longer probe-renders its child builder,
- `focus_within` is derived from the real rendered root,
- the `use_state called multiple times per frame` warning path is closed for that surface,
- and hover-only menu actions still reveal correctly on hover or focus-within.

Deliverables:

- one concrete builder-discipline rule in the workstream docs,
- one landed recipe fix in `sidebar.rs`,
- focused regression coverage for single-render and hover/focus visibility.

Exit gates:

- the builder runs once per frame,
- local-state collisions are no longer triggered by recipe internals,
- and sidebar action visibility stays aligned with the intended desktop behavior.

## Phase C - Narrow follow-on audits

Status: Active

Goal:

- audit only the nearby recipe surfaces that may still share the same failure modes,
- keep the lane narrow and evidence-driven,
- and avoid turning a concrete closeout into a vague whole-library rewrite.

Deliverables:

- a short audit pass over remaining text-entry wrappers when concrete evidence warrants it,
- a small authoring checklist for builder callbacks,
- and any additional gates only where a real mismatch is found.

Exit gates:

- no remaining known text-entry wrapper in this lane shows inactive pointer-focus chrome,
- no known recipe builder in this lane relies on same-frame speculative rendering,
- and follow-on work is either closed or split into a new, narrower workstream.
