# Text Style Cascade (Fearless Refactor v1)

Status: In progress.

This document is **non-normative**. It tracks the implementation plan for ADR 0314:

- `docs/adr/0314-inherited-text-style-cascade-and-refinement-v1.md`

## Why this workstream exists

Fret already has a foreground inheritance story, but not a typography inheritance story.

That mismatch shows up in exactly the places where a component ecosystem grows fastest:

- descriptive copy under alert/dialog/sheet/popover/card/field shells,
- headless component recipes that want subtree-local defaults,
- AI/content surfaces with direct compound children,
- component docs/examples that should read like upstream source instead of framework-specific
  workarounds.

In practice, the repo has accumulated two classes of workaround:

1. repeated metric lookups for “description-like” text, and
2. recursive descendant patching when a component wants inherited typography without a mechanism.

This is the right moment to fix the contract before more component families depend on ad hoc
patterns.

## Source of truth

### External references

- GPUI/Zed subtree text refinement model:
  - `repo-ref/zed/crates/gpui/src/style.rs`
  - `repo-ref/zed/crates/gpui/src/styled.rs`
  - `repo-ref/zed/crates/gpui/src/window.rs`
  - `repo-ref/zed/crates/gpui/src/elements/text.rs`

### Local anchors

- Foreground inheritance baseline:
  - `crates/fret-ui/src/element.rs`
  - `ecosystem/fret-ui-kit/src/declarative/current_color.rs`
- Default text style resolution today:
  - `crates/fret-ui/src/text/props.rs`
- Existing typography policy workstream:
  - `docs/workstreams/ui-typography-presets-v1/ui-typography-presets-v1.md`
- Existing foreground migration workstream:
  - `docs/workstreams/foreground-style-context-fearless-refactor-v1/DESIGN.md`
- Follow-on visual-text recipe gap (`Shimmer` / streaming plan text):
  - `docs/workstreams/shimmer-text-style-source-fearless-refactor-v1/DESIGN.md`

## Problem statement

### What is wrong today

The repository has no general-purpose inherited text-style mechanism.

As a result:

- typography intent is often encoded at leaf nodes only,
- subtree defaults are rebuilt per component,
- direct children composition requires patching instead of inheritance,
- “description/body copy” surfaces drift between shadcn, AI, and app-owned code.

### Why this matters

This is not only a styling nicety. Text-style inheritance affects:

- layout measurement,
- cache keys for text shaping/layout,
- authoring API shape,
- whether components can expose copy-pastable upstream-like examples.

That makes it a contract-level change.

## Scope / layering

### `crates/fret-core`

Owns the portable text refinement data model.

### `crates/fret-ui`

Owns:

- inherited text-style runtime propagation,
- resolution precedence,
- integration with passive text leaf measurement/paint.

It must **not** absorb component policy such as “description” or “field helper text”.

### `ecosystem/fret-ui-kit`

Owns:

- subtree typography helpers,
- preset vocabulary,
- migration shims that make the mechanism ergonomic.

### `ecosystem/fret-ui-shadcn` / `ecosystem/fret-ui-ai`

Own:

- recipe migration,
- removal of duplicate per-component metric lookup code,
- docs/demo updates and regression gates.

## Invariants (must hold)

1. **Inherited text style is measured, not paint-only**
   - Subtree typography must participate in text measurement and layout cache decisions.
2. **Explicit leaf style remains authoritative**
   - Inherited refinement is a default, not a forced override.
3. **v1 only touches passive text**
   - Inputs/editors keep their current text-style ownership until explicitly migrated.
4. **Policy stays out of `crates/fret-ui`**
   - The runtime provides the cascade; ecosystems decide the semantic preset vocabulary.
5. **Migration must shrink ad hoc component logic**
   - When the mechanism lands, local recursive patching and duplicate metric lookup helpers should
     disappear rather than coexist forever.

## Options considered

### Option A: Keep patching components locally

Rejected as the primary direction.

Why:

- it repeats metric and precedence logic across crates,
- it makes direct children composition brittle,
- and it leaves the root contract problem unsolved.

### Option B: Port GPUI's full style system immediately

Rejected for v1.

Why:

- it is much broader than the actual pain point,
- it would unnecessarily couple layout/chrome/style refactors,
- and it raises the landing risk without increasing certainty on the text contract.

### Option C: Add a narrow inherited text-style mechanism + staged migration

Chosen.

Why:

- it addresses the hard contract directly,
- it aligns with GPUI's proven text-style stack model,
- and it keeps component policy/preset work in the ecosystem layer.

## Design direction

### Runtime model

The runtime should behave as if it owns a `text_style_stack` analogous to GPUI's model, even if the
first Fret implementation uses a different internal carrier.

Normative outcome:

- container/subtree roots can install a text-style refinement,
- passive text leaves resolve `explicit > inherited > theme default`,
- the inherited refinement participates in measure/prepaint/paint.

### Authoring model

The repo should converge on one boring path:

- subtree-local typography defaults are installed by a real layout root or a dedicated text-style
  scope,
- components do not recursively rewrite descendant text nodes unless they are intentionally doing a
  one-off compatibility shim.

### Policy model

`fret-ui-kit` now owns the subtree authoring surface in `ecosystem/fret-ui-kit/src/typography.rs`:

- `scope_text_style(...)`
- `scope_text_style_with_color(...)`
- `scope_description_text(...)`
- `TypographyPreset -> TextStyleRefinement` via `preset_text_refinement(...)`

This keeps semantic naming (`description`, `control_ui_sm`, `muted_body`) out of `crates/fret-ui`.

### Description-family `children` API policy

Decision:

- Do **not** add a new runtime/text-mechanism surface for composable description content.
- Treat composable description bodies as a **component/recipe-layer API** problem.
- Use subtree-local description scoping (`scope_description_text*`) as the single shared implementation path.

Rationale:

1. The mechanism gap is already closed by `TextStyleRefinement` + inherited foreground/text-style scope.
2. A description component that accepts nested children is still just “an element subtree with description semantics/tokens”, not a new text primitive.
3. Different families want different wrapper layout policy (`w_full`, gaps, alignment, modal description registration), so a one-size-fits-all runtime API would overfit and leak policy downward.

Policy rule:

- Add `new_children(...)` / `into_element_with_children(...)` only when one of these is true:
  - upstream shadcn/Radix examples commonly use nested content,
  - in-tree product usage needs inline code/links/multiple passive text leaves,
  - the component already has a provider/context pattern that naturally constructs descendants lazily.
- Otherwise keep the public surface text-first and rely on `scope_description_text*` internally.

First selected adopters:

- `AlertDescription` (already landed)
- `CardDescription`
- `DialogDescription`

Deferred surfaces:

- `SheetDescription`, `PopoverDescription`, `AlertDialogDescription`, `EmptyDescription`
- `FieldDescription` stays text-first until there is concrete demand, because it also carries form-description registration concerns.

### Title-family audit policy

Decision:

- Do **not** add a generic runtime/title-mechanism helper just to mirror the description-family API.
- Treat title surfaces as **semantic owners** whose typography and container policy may legitimately differ per family.
- Expose composable `children` APIs selectively at the component layer when upstream/title docs are children-first.

Rationale:

1. The mechanism gap is already closed by inherited text-style / foreground scope; the remaining drift is API shape and composition, not missing runtime text machinery.
2. Title surfaces do **not** share one stable semantic contract the way description/helper-copy surfaces do: `ArtifactTitle`, `EnvironmentVariablesTitle`, and `TerminalTitle` differ in weight, foreground, and wrapper semantics, while `CodeBlockTitle` is primarily a layout container.
3. A single `scope_title_text(...)` convenience would either overfit medium/semibold heading-like titles or underfit muted/control/container titles such as `TerminalTitle` and `CodeBlockTitle`.

Policy rule:

- Add `new_children(...)` only when upstream docs/examples already treat the title as composable content or when default fallback text must coexist with custom descendants.
- Keep the surface text-first when the title is a local recipe with only plain-copy demand.
- When a title becomes composable, prefer subtree-local inherited refinement/color on the component-owned root so nested passive text stays unpatched.

Current audit outcome:

- Add selective component-layer `children` APIs:
  - `ArtifactTitle`
  - `EnvironmentVariablesTitle`
  - `TerminalTitle`
- Already aligned / already composable:
  - `CodeBlockTitle`
  - `AnnouncementTitle`
- Keep intentionally text-first for now:
  - `BannerTitle`

### Landed authoring surface

The repo should now prefer one boring authoring path:

1. If a passive text leaf should fully inherit subtree-local defaults, start from `ui::raw_text(...)`
   instead of `ui::text(...)` so no preset style is baked into the leaf.
2. Install subtree defaults on a real root using `fret_ui_kit::typography::scope_text_style(...)` or
   `scope_text_style_with_color(...)`.
3. For description/helper-copy surfaces, prefer `scope_description_text(...)` (or the fallback-key
   variant) instead of rebuilding metric lookup logic per component.
4. If a component already has a `TypographyPreset`, bridge it through
   `fret_ui_kit::typography::preset_text_refinement(...)` for full semantic overrides, or
   `fret_ui_kit::typography::composable_preset_text_refinement(...)` when the subtree should keep
   composing with parent defaults.
5. Foundation passive-text helpers should prefer composable refinements (for example
   `fret_ui_kit::typography::composable_refinement_from_style(...)`) so they keep semantic
   size/line-height intent while leaving default-equivalent family/emphasis fields unset for parent
   scopes to contribute.

This is the preferred ecosystem path until/unless a richer subtree text authoring DSL is needed.

## What this workstream is not

This is **not**:

- a full prose/layout typography system,
- a rich-text span styling redesign,
- an editor text pipeline rewrite,
- a replacement for `ForegroundScope`.

## Exit criteria

This workstream is complete when:

- ADR 0314 is implemented for passive text leaves,
- `fret-ui-kit` has an ergonomic subtree text-style authoring surface,
- high-value description/body component families have migrated,
- at least one AI/direct-children surface no longer needs recursive descendant patching,
- and duplicate per-component description typography logic has a clear cleanup path.
