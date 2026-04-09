# Baseline Decision 2026-04-09

Status: Accepted for this lane
Date: 2026-04-09

## Decision

For the `icon-system-extension-v1` lane, the correct v1 boundary is:

This lane no longer optimizes for the smallest contract delta.
The accepted rule is:

- prefer the durable public boundary now,
- even if that means a broader pre-release refactor,
- because a second icon-contract rewrite later would be worse.

### 1. `fret-icons` should grow a first-class icon-definition contract

Do not stop at "raw `IconSource` plus a little adjacent metadata".

The durable target is a first-class icon definition/resolution contract whose conceptual shape is:

- primary source data,
- optional fallback data,
- explicit presentation/render-intent metadata,
- alias resolution that preserves all of the above.

That means the lane should think in terms of an `IconDefinition` / `ResolvedIcon` family even if
the exact type names land in slices.

### 2. Render intent should not live inside raw `IconSource`

Keep `IconSource` as source data.
Render intent belongs in the icon definition/presentation layer rather than inside the raw source
enum.

The lane should treat the durable contract shape as conceptually equivalent to:

- source bytes,
- optional fallback data,
- plus a renderer-agnostic presentation block such as:
  - monochrome mask,
  - original SVG colors.

The exact type names can still be chosen during implementation, but the boundary is fixed:

- `IconSource` stays data-only,
- presentation/render intent is explicit,
- and helper code must be able to read the fully resolved definition after alias resolution.

### 3. Multicolor SVGs should land as a distinct declarative surface in `crates/fret-ui`

Do not overload `SvgIconProps` with a render-mode toggle.

Instead, add a distinct `SvgImage`-style declarative surface in `crates/fret-ui` that maps to the
already-existing `SceneOp::SvgImage` path.

The intended public shape is conceptually:

- `SvgIconProps` / `cx.svg_icon(...)` for tinted monochrome icons,
- `SvgImageProps` / `cx.svg_image(...)` for original-color SVG rendering.

This keeps the current `currentColor` icon path explicit and stable while giving multicolor SVGs a
first-class runtime surface.

### 4. Third-party import-pack crates should follow an explicit build-time pack protocol

The approved v1 protocol for custom packs should be:

1. vendor-prefixed ids stay explicit,
2. semantic aliases remain opt-in and use first-writer-wins registration,
3. pack crates expose explicit install seams,
4. Iconify/local-SVG importing happens at build/codegen time, not through a runtime network client
   in core,
5. crates that ship icons plus other package-owned assets publish one composed installer/bundle
   surface instead of asking apps to replay raw registration steps,
6. pack metadata/provenance has an explicit home rather than living only in README prose.

The intended minimum public shape is:

- `register_vendor_icons(&mut IconRegistry)`
- optional `register_ui_semantic_aliases(&mut IconRegistry)`
- `app::install(app: &mut fret_app::App)`
- optional `advanced::install_with_ui_services(app, services)` when the crate wants to fit the
  bootstrap installer shape

This keeps the extension seam explicit, grep-friendly, and compatible with both `fret` and
`fret-bootstrap`.

## Why this is the correct v1 boundary

### 1. ADR 0065 already says source is data, not rendering

Putting render mode directly inside `IconSource` would blur the ADR's own separation.

What the lane actually needs is not "rendering logic in the registry".
It needs a durable icon-definition contract that survives resolution so authoring helpers can
choose the correct already-existing runtime path without inventing helper-only shadow contracts.

### 2. `SvgIconProps` already means "tinted icon", not "any SVG"

`SvgIconProps` currently carries:

- explicit color,
- inherited foreground behavior,
- and an icon-oriented fit/default path.

If we add a generic render-mode field there, we immediately get awkward semantics:

- what does `color` mean when the SVG keeps original colors?
- should `inherit_color` be ignored, rejected, or half-applied?

A distinct `SvgImage` surface keeps those meanings clean and reviewable.

### 3. The renderer work is already done lower down

This lane does not need a new low-level raster path.
`SceneOp::SvgImage` and `Canvas::svg_image(...)` already exist.

That means the cleanest move is to expose the missing declarative/runtime surface directly instead
of inventing another policy layer in `fret-ui-kit`.

### 4. `dioxus-iconify` is a good generator reference, not a direct runtime model

`repo-ref/dioxus-iconify` is useful for two reasons:

- it proves that a build-time import/vendoring workflow is practical,
- and it shows that generated alias modules can hide vendor collection names from app code.

But Fret should not copy its public model directly because:

- Dioxus generates app-local Rust constants/components as the primary surface,
- while Fret needs a reusable framework contract centered on semantic `IconId` plus pack
  registration/install seams.

So the right reuse is:

- borrow the generator/vendoring idea,
- do not borrow the raw vendor-name-first public API as Fret's core contract.

## Alternatives considered and rejected

### A. Put render mode directly in `IconSource`

Rejected because:

- it weakens the "source is data" separation,
- it makes future source expansion (`GlyphFallback`, other metadata) harder to reason about,
- and it couples raw bytes and authoring intent too tightly.

### B. Keep render mode out of the registry entirely and solve it in helpers only

Rejected because:

- third-party packs would have no explicit way to declare their default rendering intent,
- different apps would have to rediscover the same pack-specific rule,
- and reviewability would depend on helper conventions instead of contract data.

### C. Stop at "small adjacent metadata" and avoid a real icon-definition reset

Rejected because:

- it still leaves `fret-icons` centered on byte-only resolution,
- it pushes fallback/presentation growth into follow-on rewrites,
- and it increases the chance that multicolor support becomes another compatibility layer instead of
  the primary contract.

### D. Add one generic SVG declarative surface with an enum mode

Rejected because:

- it would overload today's `SvgIcon` semantics,
- it would create invalid or confusing prop combinations,
- and it would force the monochrome and multicolor stories into one API before the repo has proof
  that such unification is actually better.

## Consequences for the next slice

1. `ecosystem/fret-icons` should move toward a first-class icon-definition / resolved-icon
   contract.
2. `crates/fret-ui` should add a distinct declarative `SvgImage` element surface.
3. `ecosystem/fret-ui-kit` should keep `icon(...)` as the monochrome golden path and add an
   explicit path that can honor registry-authored icon presentation.
4. the lane should add a target-interface document so the refactor is measured against a durable
   end state rather than a smallest diff.
5. `docs/crate-usage-guide.md` should eventually document the pack protocol explicitly instead of
   requiring users to infer it from first-party examples.

## Evidence

- `docs/adr/0065-icon-system-and-asset-packaging.md`
- `ecosystem/fret-icons/src/lib.rs`
- `ecosystem/fret-ui-kit/src/declarative/icon.rs`
- `crates/fret-core/src/scene/mod.rs`
- `crates/fret-ui/src/element.rs`
- `crates/fret-ui/src/elements/cx.rs`
- `crates/fret-ui/src/declarative/host_widget/paint.rs`
- `crates/fret-ui/src/canvas.rs`
- `docs/crate-usage-guide.md`
- `docs/examples/todo-app-golden-path.md`
- `repo-ref/dioxus-iconify/README.md`
- `repo-ref/dioxus-iconify/src/main.rs`
- `repo-ref/dioxus-iconify/src/generator.rs`
- `repo-ref/dioxus-iconify/src/svg.rs`
