# ADR 0222: Semantics Decorators and `attach_semantics` (v1)

Status: Accepted

## Context

Fret’s declarative authoring model is typed and layout-driven (ADR 0028 / ADR 0057). We also rely
on the semantics tree for:

- platform accessibility (ADR 0033),
- deterministic UI automation and diagnostics selectors (ADR 0159),
- stable `test_id` anchors for `fretboard diag` scripts and regression gates.

Today, the primary “generic” surface for stamping semantics is the `Semantics` element:

- it produces a semantics node (role/label/state/test_id),
- **and it also participates in layout** via `SemanticsProps.layout`.

This creates a recurring footgun:

- authors use `Semantics` as a wrapper solely to stamp `test_id` / role / label onto an existing
  layout node,
- but that wrapper **inserts a new layout node**, which can break common Tailwind/shadcn patterns
  like `flex-1` + `min-w-0` width propagation.

Motivating example (real-world):

- In the desktop Todo demo, wrapping the “title text” flex item in `Semantics` caused all item
  titles to render as ellipsis (`...`) because the wrapper broke the `flex` sizing chain.

We need a way to “attach semantics to an existing node” without adding a layout wrapper, similar
in spirit to DOM `asChild`/`Slot`, but consistent with Fret’s typed model and layering rules
(ADR 0115).

## Decision

### D1 — Introduce **semantics decorators** (layout-transparent)

We add a small, explicit mechanism to attach semantics to an existing declarative element without
adding a new layout node:

- `AnyElement::attach_semantics(SemanticsDecoration)`

The attached semantics are applied when producing semantics snapshots; they do **not** affect:

- layout,
- paint,
- hit-testing,
- focus traversal policy (beyond what the target element already supports).

### D2 — Semantics decorators are **restricted** (not a general “prop bag”)

This is intentionally not Radix `Slot`:

- we do not support merging arbitrary layout/interaction props across unrelated element kinds,
- we do not retarget action hooks at runtime,
- we do not introduce dynamic attribute maps.

Instead, v1 targets the minimum set needed for diagnostics and basic a11y stamping:

- `test_id`
- `role` override
- `label` override
- `value` override

Future versions may extend this surface with explicit fields (e.g. selected/expanded/checked,
relations like labelled-by) if needed, but the design must remain typed and small.

### D3 — Precedence: decorator overrides element-produced semantics

When a semantics decorator is present:

- any field set in the decorator overrides the field produced by the element kind itself
  (`Pressable`, `Text`, `TextInput`, etc.).

This makes “debug stamping” deterministic and avoids fragile “who wrote last” behavior across
layers.

### D4 — Guidance: use `Semantics` for structure, `attach_semantics` for stamping

- Use `Semantics` when you want to introduce an explicit semantics node boundary in the tree
  (grouping, relationships, roles that should be their own node).
- Use `attach_semantics` when you want to add `test_id` / label / role to an existing layout node
  without changing layout.

## Consequences

- Typed Fret UI can achieve DOM-like “attach a11y/test props to an arbitrary child” outcomes without
  a generic slot system.
- `fretboard diag` scripts become more stable because `test_id` can be attached to real layout
  nodes instead of wrapper nodes that accidentally change sizing behavior.
- The existing `Semantics` element remains valid, but authors must treat it as a layout wrapper.

## Alternatives Considered

### A) Make `Semantics` fully layout-transparent everywhere

Rejected for v1:

- `Semantics` currently carries `LayoutStyle` intentionally and is used as a real layout wrapper in
  multiple places.
- Making it globally layout-transparent would be a breaking change and may conflict with existing
  patterns (e.g. intentional wrappers, multi-child semantics wrappers).

### B) Implement a general Radix `Slot/asChild` mechanism

Rejected (ADR 0115):

- it would either collapse typed contracts into a dynamic prop map, or require pervasive runtime
  retargeting/merging logic.

## References

- Accessibility infrastructure: `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- Declarative layout semantics: `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- Trigger composition without `asChild`: `docs/adr/0115-trigger-composition-and-no-slot-aschild.md`
- Diagnostics + scripted tests: `docs/adr/0159-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`

