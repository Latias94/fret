# Foreground inheritance (late binding) (fearless refactor v2)

Status: In progress (workstream)

Last updated: 2026-02-24

Milestones: `docs/workstreams/foreground-inheritance-late-binding-v2-milestones.md`

TODO: `docs/workstreams/foreground-inheritance-late-binding-v2-todo.md`

Related (v1 bridge / background): `docs/workstreams/current-color-inheritance-fearless-refactor-v1.md`

## Motivation

We want shadcn/Radix-style authoring ergonomics where descendants (icons, spinners, text) inherit
their semantic foreground from the nearest “host” without callsites manually threading tokens.

v1 added a build-time provider using `ElementContext::inherited_state_*`. This improved ergonomics,
but it has a hard limitation in Fret’s current authoring model:

- Fret builds an ephemeral `AnyElement` tree in one pass, and many leaves resolve their final
  paint-time values (e.g. `SvgIconProps.color`) during `into_element(cx)`.
- If an element is constructed *outside* a provider scope and then passed into a slot as an
  already-built `AnyElement`, it cannot retroactively inherit the provider’s state.

This is a correctness and refactor hazard for component ecosystems: it makes authoring outcomes
depend on whether a subtree was “built inline” vs “prebuilt and passed in”.

v2 moves foreground inheritance from a build-time stack into a paint-time inherited style state,
encoded in the element tree.

## Design (v2)

### 1) Encode inheritance in the element tree

Add a transparent wrapper element kind:

- `ForegroundScope` (`crates/fret-ui`): installs an optional `foreground: Color` for the subtree.

This makes inheritance depend on structural ancestry (like CSS/Flutter), not on how the
`AnyElement` values were constructed.

### 2) Plumb paint-time inherited style

During paint traversal, carry a small `PaintStyleState`:

- v2 carries `foreground: Option<Color>` only.
- The state is passed into `UiTree::paint_node` and down to children.
- The state is included in `PaintCacheKey` so cached paint output is not replayed under a
  different inherited foreground.

### 3) Leaf adoption (consume inherited foreground)

Leaf painting resolves color in this order:

1. Explicit per-leaf color (if provided by the element props).
2. Inherited `PaintStyleState.foreground` (if set by an ancestor `ForegroundScope`).
3. Existing theme fallback.

Special case (icons):

- `SvgIconProps` gains `inherit_color: bool`.
- When `inherit_color=true`, paint prefers inherited foreground and falls back to `SvgIconProps.color`.

This preserves existing “explicit color wins” behavior while enabling `currentColor`-style default
inheritance without changing the `SvgIconProps.color: Color` ABI everywhere.

## Migration strategy

1. Land mechanism: `ForegroundScope` + `PaintStyleState` + paint/cache plumbing.
2. Update ecosystem leaves (icon/spinner/text builders) to opt into inherited foreground by default.
3. Migrate shadcn-aligned hosts that compute stateful foregrounds to install `ForegroundScope`
   rather than relying on build-time inherited state.
4. Keep the v1 provider APIs temporarily for compatibility, but migrate high-ROI components first.
5. Add/expand diag screenshot gates to lock “dark background + icon” cases.

## Tracking table

| Item | Layer | Status | Evidence anchors |
| --- | --- | --- | --- |
| Add `ForegroundScope` element kind | `crates/fret-ui` | Landed | `crates/fret-ui/src/element.rs` |
| Plumb `PaintStyleState` through paint traversal | `crates/fret-ui` | Landed | `crates/fret-ui/src/widget.rs` |
| Include inherited foreground in `PaintCacheKey` | `crates/fret-ui` | Landed | `crates/fret-ui/src/tree/paint_cache.rs` |
| `SvgIconProps` supports inherited foreground via `inherit_color` | `crates/fret-ui` | Landed | `crates/fret-ui/src/element.rs` + `crates/fret-ui/src/declarative/host_widget/paint.rs` |
| Text/StyledText/SelectableText prefer inherited foreground when `color=None` | `crates/fret-ui` | Landed | `crates/fret-ui/src/declarative/host_widget/paint.rs` |
| Spinner prefers inherited foreground when `color=None` | `crates/fret-ui` | Landed | `crates/fret-ui/src/declarative/host_widget/paint.rs` |
| Paint-time regression test (ForegroundScope -> text + icon) | `crates/fret-ui` | Landed | `crates/fret-ui/src/declarative/tests/foreground_inheritance.rs` |
| Ecosystem `icon_with(...)` opts into inherited foreground by default | `ecosystem/fret-ui-kit` | Landed | `ecosystem/fret-ui-kit/src/declarative/icon.rs` |
| shadcn hosts install foreground scope for content | `ecosystem/fret-ui-shadcn` | In progress | (see TODO) |

## Open questions (v2+)

- Do we want a generalized `StyleScope` that also carries a full `TextStyle` stack, or keep
  foreground-only in `crates/fret-ui` and add text-style inheritance at a higher layer?
- Should `SvgIconProps` evolve from `(color, inherit_color)` to an explicit `ColorPolicy` enum to
  avoid sentinel/fallback ambiguity?
- How should inherited style interact with view-cache “contained layout” semantics and future
  retained/declarative bridging?
