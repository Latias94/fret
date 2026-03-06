# Foreground Style Context (Fearless Refactor v1)

Status: Draft (workstream note)

This workstream exists because Fret's current foreground inheritance surface has a semantic mismatch:

- it looks like style inheritance,
- but it is implemented as a real wrapper element,
- which means it can still affect tree shape, layout ownership, and authoring expectations.

That mismatch is easy to misunderstand in a large codebase. The goal of this workstream is to make
foreground inheritance **boring, explicit, and safe** by moving it toward a true style-context
model owned by real elements instead of synthetic author-facing wrapper nodes.

## Context

Today we have two overlapping mechanisms:

1. A subtree-scoped inherited-state pattern in `ElementContext`:
   - `docs/service-injection-and-overrides.md`
   - `ecosystem/fret-ui-kit/src/declarative/current_color.rs`
2. A concrete declarative element node that pushes foreground during paint:
   - `crates/fret-ui/src/element.rs`
   - `crates/fret-ui/src/declarative/host_widget/measure.rs`
   - `crates/fret-ui/src/declarative/host_widget/paint.rs`

This means authoring helpers such as `scope_children(...)` can look like a transparent fragment
even though the runtime sees a wrapper node.

That is the root problem. The problem is not only documentation quality or a few bad call sites.
It is that the author-facing abstraction and the runtime shape do not match.

## Source of Truth

### External references

- GPUI / Zed style cascade model:
  - `repo-ref/zed/crates/gpui/src/styled.rs`
  - `repo-ref/zed/crates/gpui/src/elements/div.rs`
- Fret guidance on subtree-scoped providers:
  - `docs/service-injection-and-overrides.md`

### Local anchors

- Current inherited foreground helper surface:
  - `ecosystem/fret-ui-kit/src/declarative/current_color.rs`
- Current wrapper-element contract:
  - `crates/fret-ui/src/element.rs`
  - `crates/fret-ui/src/elements/cx.rs`
- Current runtime behavior for the wrapper:
  - `crates/fret-ui/src/declarative/host_widget/measure.rs`
  - `crates/fret-ui/src/declarative/host_widget/paint.rs`

## Scope / Layering

This workstream crosses both mechanism and ecosystem layers, so ownership must stay explicit:

- `crates/fret-ui`
  - owns the runtime contract for inherited foreground/style context,
  - owns wrapper compatibility behavior while migration is in flight,
  - must not absorb shadcn-specific policy.
- `ecosystem/fret-ui-kit`
  - owns authoring helpers and migration shims,
  - should stop teaching author-facing wrapper semantics as the preferred model.
- `ecosystem/fret-ui-shadcn`
  - owns recipe migration and parity gates,
  - should migrate high-risk call sites away from wrapper-shaped inheritance.

This is a mechanism-level correction with ecosystem migration work layered on top.

## Problem Statement

### What is wrong today

The current `ForegroundScope` model is easy to misuse because it has mixed signals:

- it behaves like inherited styling during paint,
- but it is still a real declarative node in mount/measure/layout,
- and helpers such as `scope_children(...)` can collapse multiple siblings under one wrapper in a
  way that the parent flow did not obviously ask for.

This creates failure modes that feel unrelated at the call site:

- wrapped text unexpectedly overflows in one line,
- fill/shrink behavior changes after adding color inheritance,
- sibling ordering or layout ownership becomes less obvious,
- overlay content roots gain hidden wrapper semantics.

### Why this matters

Fret is explicitly contract-driven and layering-sensitive. A style inheritance surface that behaves
like a layout wrapper is exactly the kind of subtle contract drift that keeps reappearing as
recipe-level bugs.

## Invariants (must hold)

1. **Style inheritance must not require an author-visible synthetic layout node**
   - Installing inherited foreground on a subtree must not silently rewrite the parent's flow
     ownership model.
2. **Real layout roots own style scope boundaries**
   - Containers, flex roots, rows, columns, pressables, and similar concrete elements should be the
     place where inherited style context is introduced.
3. **Consumers resolve explicit color first, inherited color second, theme fallback last**
   - Text, styled text, selectable text, icons, and spinners must preserve the same priority model.
4. **Compatibility must be staged**
   - Existing `ForegroundScope` call sites cannot be hard-broken in one step.
5. **Behavior changes must leave gates behind**
   - At least one unit/integration gate must prove that adding inherited foreground no longer changes
     sibling flow ownership.

## Current Risk Surface

Known risk anchors that should be reviewed during migration:

- `ecosystem/fret-ui-shadcn/src/dropdown_menu.rs`
- `ecosystem/fret-ui-shadcn/src/select.rs`
- `ecosystem/fret-ui-shadcn/src/tabs.rs`
- `ecosystem/fret-ui-shadcn/src/input_group.rs`
- `ecosystem/fret-ui-ai/src/elements/message.rs`

These are not necessarily all wrong today, but they represent the class of places where inherited
foreground can accidentally behave like a wrapper fragment.

## Options Considered

### Option A: Keep `ForegroundScope`, improve docs only

Rejected as the main strategy.

Why:

- It keeps the abstraction mismatch intact.
- It relies on call-site discipline in a large mono-repo.
- It does not remove the core wrapper-shaped footgun.

### Option B: Keep wrappers, but restrict authoring to `scope_element(...)`

Useful as a temporary stopgap, but not sufficient as the final design.

Why:

- It reduces the worst misuse pattern (`scope_children(...)` returning multiple siblings).
- It still models inheritance as a wrapper node instead of true style context.
- It still makes post-hoc styling depend on tree-shape rewrites.

### Option C: Move inherited foreground to a real style-context model

Recommended.

Why:

- It aligns authoring intent with runtime behavior.
- It matches the direction used by GPUI/Zed: inherited style is context, not a synthetic layout
  element.
- It makes layout semantics easier to reason about and easier to gate.

## Proposed Design

### Stage 1: True inherited foreground context

Introduce a mechanism-level inherited foreground carrier that is pushed by real elements and read by
paint consumers when no explicit foreground is provided.

Expected v1 consumers:

- `Text`
- `StyledText`
- `SelectableText`
- `SvgIcon`
- spinner/loading glyph surfaces that currently inherit foreground-like color

This stage should focus on foreground only, not the full text-style cascade.

### Stage 2: Real elements own scope boundaries

Add a small opt-in field or equivalent runtime hook so real layout elements can install inherited
foreground for their descendants without requiring a separate wrapper node.

Candidate owners:

- `Container`
- `Pressable`
- flex/row/column-like layout elements
- other real subtree roots already used as recipe composition boundaries

Important: this should not be modeled as generic layout state. It is better treated as paint/text
context carried by traversal, not by `LayoutStyle` itself.

### Stage 3: Compatibility bridge for `ForegroundScope`

Keep `ForegroundScope` as a compatibility surface during migration, but change its role:

- no longer the preferred authoring primitive,
- eventually an internal bridge or deprecated public surface,
- guarded by migration tests so behavior remains explicit while call sites move.

### Stage 4: Optional follow-up for text-style cascade

Once foreground inheritance is stable, decide whether a broader inherited text-style context should
also exist (font size, weight, line height, letter spacing, etc.).

This should be a separate decision gate because it is a broader contract than foreground alone.

## Migration Strategy

1. Lock this workstream's design and audit the existing surface.
2. Implement a minimal inherited foreground context in `crates/fret-ui`.
3. Migrate the highest-risk call sites in `fret-ui-shadcn` and nearby ecosystem crates.
4. Mark `scope_children(...)` as transitional and stop using it in new code.
5. Decide whether `ForegroundScope` remains public, becomes deprecated, or becomes internal-only.
6. Decide separately whether full text-style cascade needs an ADR-backed v2.

## ADR Trigger (when an ADR becomes necessary)

This workstream document is enough for exploration and migration planning.

An ADR should be added or updated before we do any of the following:

- change the public contract of `ForegroundScope`,
- introduce a generic inherited foreground/text-style contract in `crates/fret-ui`,
- change behavior across overlay/provider/root boundaries in a way that downstream authors must rely
  on,
- or remove the public wrapper-based surface entirely.

If only ecosystem helpers change while the mechanism contract stays the same, an ADR is likely not
required.

## Risks / Open Questions

1. **Overlay/root inheritance boundaries**
   - Should inherited foreground automatically cross overlay roots, or should recipes thread it
     explicitly?
2. **Compatibility window**
   - How long do we keep `ForegroundScope` public once the real style-context path exists?
3. **Non-text consumers**
   - Which visual primitives should consume inherited foreground in v1 beyond text and icons?
4. **Text-style scope creep**
   - Do not let foreground inheritance silently become a full typography cascade without an explicit
     decision.

## Definition of Done (v1)

This workstream is considered ready to land when:

- the mechanism path for inherited foreground no longer requires author-visible wrapper nodes,
- high-risk recipe surfaces stop depending on `scope_children(...)`,
- regression gates prove that adding inherited foreground does not rewrite sibling flow ownership,
- and the public migration story is documented clearly enough that future refactors are boring.
