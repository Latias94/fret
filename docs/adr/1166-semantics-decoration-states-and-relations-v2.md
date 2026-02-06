# ADR 1166: Semantics Decorations (v2) — states and relations

Status: Proposed

## Context

ADR 1161 introduced `AnyElement::attach_semantics(SemanticsDecoration)` as a **layout-transparent**
mechanism for stamping `test_id` and limited a11y fields onto an existing declarative element.

This solved a recurring layout footgun where authors wrapped a flex item in a `Semantics` element
just to add `test_id` / label / role, unintentionally inserting an extra layout node (e.g. breaking
`flex-1` + `min-w-0` width propagation and causing unexpected ellipsis).

However, real-world authoring still needs a small set of additional semantics fields **without**
adding a layout wrapper:

- state flags: `disabled`, `selected`, `expanded`, and the tri-state `checked`
- relationships: `labelled_by`, `described_by`, and `controls`
- `active_descendant` for composite widgets using `aria-activedescendant`-style patterns

Today, authors must either:

- introduce a `Semantics` wrapper (risking layout changes), or
- duplicate semantics state into a typed widget's own a11y props (not always available), or
- give up on structured relationships in automation and diagnostics.

We want to keep the decorator surface typed and small (not a generic prop bag), while covering the
most common "stamp semantics onto an existing node" needs.

## Decision

Extend `SemanticsDecoration` to support:

1. **State flags**
   - `disabled: Option<bool>`
   - `selected: Option<bool>`
   - `expanded: Option<bool>`
   - `checked: Option<Option<bool>>` (outer `Option` = "override present"; inner `Option` = tri-state)

2. **Relations (declarative element IDs)**
   - `labelled_by_element: Option<u64>`
   - `described_by_element: Option<u64>`
   - `controls_element: Option<u64>`
   - `active_descendant_element: Option<u64>`

### Precedence and behavior

- Decorations are applied **after** element-kind semantics are produced.
- For scalar fields (`role`, `label`, `test_id`, `value`, state flags), when the decoration field is
  present it **overrides** the element-produced value.
- For relationships, decorations are **additive**:
  - they push additional `labelled_by` / `described_by` / `controls` entries when resolvable
  - they do not clear relationships contributed by the element kind
- `active_descendant_element` resolves via the per-frame declarative element ID map; if the target
  node cannot be resolved, the decorator does not mutate `active_descendant`.

### Non-goals

- This is not a general Radix `Slot/asChild` prop-merging system (ADR 0117).
- No runtime retargeting/merging of interaction handlers or layout props.
- No dynamic attribute maps; fields remain explicit and typed.

## Consequences

- `attach_semantics` becomes viable for more a11y + diagnostics use cases without layout wrappers.
- UI automation scripts (`fretboard diag`) can rely on stable `test_id` plus state/relations where
  needed, without authors accidentally changing sizing behavior.
- The semantics surface remains intentionally small, and stays aligned with Fret's typed layering
  rules.

## References

- `docs/adr/1161-semantics-decorators-and-attach-semantics-v1.md`
- `docs/adr/0033-semantics-tree-and-accessibility-bridge.md`
- `docs/adr/0117-trigger-composition-and-no-slot-aschild.md`
- `docs/adr/0174-ui-diagnostics-snapshot-and-scripted-interaction-tests.md`
