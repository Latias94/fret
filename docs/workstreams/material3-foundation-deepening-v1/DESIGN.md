# Material3 Foundation Deepening v1

Status: Closed
Last updated: 2026-05-31

## Problem

Material3 has useful foundation modules, but three important interfaces are still too shallow:

- Material context is not the only Material-facing seam for direction and tree-local defaults.
- The field family still exposes too much label, slot, active-indicator, supporting-text, and popup
  anchor implementation detail through recipe modules.
- The token visual matrix is large and difficult to extend because import, registry, audit, and
  fixture outcomes are coupled through broad modules.

This keeps locality weak: an App Author-visible drift in RTL, field geometry, or token outcomes can
require editing several Material3 recipe modules instead of one deep Material foundation module.

## Target State

Material3 keeps policy in `ecosystem/fret-ui-material3` while continuing to reuse Runtime Substrate
and `fret-ui-kit` mechanisms:

- `foundation::context` is the Material-facing interface for resolved layout direction, content
  defaults, motion scheme, ripple configuration, and logical inline helpers.
- `foundation::field_family` or equivalent private modules own shared Material field chrome and
  expose a small recipe-facing interface for TextField, Select, Autocomplete, and ExposedDropdown.
- `tokens` exposes a typed registry/outcome interface that keeps generated Material Web data and
  fixture runners as adapters.

## Scope

In scope:

- `ecosystem/fret-ui-material3/src/foundation/*`
- `ecosystem/fret-ui-material3/src/{text_field,select,autocomplete,exposed_dropdown}.rs`
- Material popup/menu consumers when they still bypass Material context direction.
- `ecosystem/fret-ui-material3/src/tokens/*`
- Material3 tests and headless/token gates.
- Workstream catalog and evidence docs.

Out of scope:

- Moving Material policy into `crates/fret-ui`.
- Replacing `fret-ui-kit` popper, roving focus, active-descendant, or direction primitives.
- Reopening closed per-component packet lanes unless this work produces a narrow follow-on.
- Changing public recipe names for compatibility alone. Breaking changes are allowed only when they
  deepen the Material3 interface and delete real accidental complexity.

## Architecture Direction

### Material Context

The Material context module becomes the one seam Material recipes cross for direction and tree-local
defaults. It adapts to core `LayoutDirection` and `fret-ui-kit` primitives behind the seam.

First migration target:

- Replace residual `direction_prim::use_direction_in_scope(cx, None)` calls in Material recipes
  where Material theme direction should be authoritative.

### Field Family

The field family module should own:

- variant chrome for filled/outlined fields,
- floating-label progress and geometry,
- leading/trailing slot logical placement,
- supporting/error text placement,
- active indicator layer,
- combobox/listbox relation plumbing that is common to field-triggered popups.

Recipe modules should provide value state, semantics details, option models, and token namespace
adapters. They should not reimplement field chrome math.

### Token Matrix

The token matrix should separate:

- source adapter: Material Web generated data,
- registry: typed Material token lookup and fallback semantics,
- outcome matrix: expected role/variant/state/scheme records,
- runner: thin Rust tests that assert matrix outcomes.

The deletion test should pass: deleting a generated source adapter should not delete the registry
interface, and deleting one fixture runner should not delete the outcome vocabulary.

## Assumptions

- Confident: `fret-ui-material3` is an Incubating Component Surface in the Policy Layer; it must not
  move Material policy into the Runtime Substrate.
- Confident: Existing closed RTL lanes prove the direction provider bridge and logical edge helpers,
  but residual consumers still bypass Material context.
- Likely: Field family deepening should happen before a large token matrix rewrite for field tokens,
  because field recipe modules currently expose much of the state/layout complexity.
- Likely: The token matrix split can be done incrementally by adding a typed registry/outcome seam
  before deleting large generated or fixture modules.

## Source References

- `CONTEXT.md`
- `docs/architecture.md`
- `docs/adr/0032-style-tokens-and-theme-resolution.md`
- `docs/adr/0057-declarative-layout-style-and-flex-semantics.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/workstreams/material3/material3-refactor-plan.md`
- `docs/workstreams/material3-visual-behavior-layout-parity-v2/WORKSTREAM.json`
- `docs/workstreams/material3-layout-direction-provider-bridge-v1/CLOSEOUT_AUDIT_2026-05-30.md`
- `docs/workstreams/material3-field-logical-insets-v1/CLOSEOUT_AUDIT_2026-05-30.md`
- `docs/workstreams/material3-token-visual-matrix-v1/WORKSTREAM.json`
