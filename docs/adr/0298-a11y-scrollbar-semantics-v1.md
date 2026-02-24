# ADR 0298: A11y scrollbar semantics (v1)

Status: Accepted

Date: 2026-02-23

## Context

Fret already publishes portable scroll container semantics via:

- `SemanticsRole::Viewport`
- `SemanticsNodeExtra.scroll` (`x/x_min/x_max/y/y_min/y_max`)
- `SemanticsActions.scroll_by`

However, the runtime also exposes a dedicated `Scrollbar` mechanism element (`crates/fret-ui`), used widely by
component ecosystems (Radix/shadcn-aligned scroll areas, code view, tables, etc.). Historically, this element did not
publish a dedicated scrollbar role, so assistive technologies could only infer scrolling state from the scroll view.

AccessKit supports `Role::ScrollBar`, and most accessibility stacks treat scrollbars as adjustable range controls.

## Decision

Add a portable scrollbar role and publish structured scroll metadata from the `Scrollbar` mechanism element.

### Contract

- Add `SemanticsRole::ScrollBar` to `crates/fret-core/src/semantics.rs`.
- Map to AccessKit `Role::ScrollBar` in `crates/fret-a11y-accesskit/src/roles.rs`.

### Production (declarative host widget)

For `ElementInstance::Scrollbar`:

- Set `role = SemanticsRole::ScrollBar`.
- Publish `extra.orientation` based on axis (`Horizontal` / `Vertical`).
- Publish `extra.scroll` for the relevant axis using the bound `ScrollHandle`:
  - `y/y_min/y_max` for vertical
  - `x/x_min/x_max` for horizontal
  - `min` is `0.0`, `max` is `ScrollHandle::max_offset()`, `value` is `ScrollHandle::offset()`.
- Publish `SemanticsActions.scroll_by` when the scrollbar has a meaningful range (`max > 0`).
- When `ScrollbarProps.scroll_target` is set, publish a `controls` relationship to the associated scroll node.

### Actions

To keep the surface end-to-end, the declarative host widget implements `scroll_by(...)` for `ElementInstance::Scrollbar`,
updating the bound `ScrollHandle` for the relevant axis.

## Consequences

- Assistive technologies can recognize scrollbars as dedicated controls (instead of generic containers).
- Automation/diagnostics can query scrollbars by role (`scroll_bar`) and assert structured scroll metadata without
  parsing strings.
- This remains mechanism-only: component ecosystems still decide scrollbar visibility policies (always/hover/scroll).

## Evidence anchors

- Contract: `crates/fret-core/src/semantics.rs`
- AccessKit role mapping: `crates/fret-a11y-accesskit/src/roles.rs`
- Declarative production: `crates/fret-ui/src/declarative/host_widget/semantics.rs`
- Scroll action plumbing: `crates/fret-ui/src/declarative/host_widget.rs`
- Gates:
  - `crates/fret-a11y-accesskit/src/tests.rs`
  - `crates/fret-ui/src/declarative/tests/semantics.rs`
