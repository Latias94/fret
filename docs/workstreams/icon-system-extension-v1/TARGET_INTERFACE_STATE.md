# Icon System Extension v1 — Target Interface State

Status: target state for the contract-first icon reset
Last updated: 2026-04-09

This document records the intended public interface state for the icon-system refactor.

It is intentionally not a "smallest diff" plan.
It exists to prevent a second public-contract rewrite once multicolor icons and third-party import
packs become normal usage rather than edge cases.

## Non-negotiable target

1. Reusable component code continues to speak only in semantic `IconId` values.
2. Alias resolution preserves the whole icon definition:
   - source,
   - fallback,
   - presentation/render intent.
3. Renderer choice is explicit in the resolved icon contract rather than being rediscovered ad hoc
   in helper code.
4. `crates/fret-ui` owns SVG runtime surfaces, not icon-pack policy or vendor semantics.
5. Build-time import/vendoring is the approved Iconify-style workflow.
   A runtime network client is not part of the core icon contract.

## 1. `ecosystem/fret-icons` target contract

### 1.1 Stable nouns that should exist

The durable icon contract should revolve around a first-class icon definition, not only raw SVG
bytes.

Target concept inventory:

| Concept | Target role | Notes |
| --- | --- | --- |
| `IconId` | semantic identity | remains the reusable component-facing id type |
| `IconDefinition` | registry value | groups source + fallback + presentation metadata |
| `IconSource` | primary source data | data only; no render policy |
| `IconFallback` | optional fallback data | where glyph fallback belongs if retained by the ADR |
| `IconPresentation` | presentation metadata | renderer-agnostic description of how authored SVG should be presented |
| `IconRenderMode` | render intent | at least `Mask` vs `OriginalColors` |
| `ResolvedIcon` / `ResolvedIconOwned` | resolved registry output | returns resolved source together with fallback/presentation metadata |

The important architectural rule is:

- `IconSource` should not become a "bag of everything".
- The registry should resolve an icon definition, not only bytes.

### 1.2 Stable behaviors that should exist

Target registry behavior:

- `register_icon(id, definition)` or an equivalent first-class registration path exists.
- Alias helpers remain available, but aliases resolve to the full target definition.
- Resolution APIs return a resolved icon definition, not only resolved SVG bytes.
- Byte-only convenience helpers may exist temporarily, but they are compatibility helpers rather
  than the real long-term contract.

### 1.3 What should not be the target state

Reject these as the final interface state:

- render-mode metadata bolted onto helper code only,
- `IconSource` carrying rendering semantics directly,
- alias resolution that preserves bytes but loses presentation intent,
- a contract where multicolor support exists only in `fret-ui-kit` conventions and not in the
  registry result.

## 2. `crates/fret-ui` target runtime surface

### 2.1 Stable element split

The runtime should expose two stable SVG leaf surfaces:

| Surface | Meaning | Lowering |
| --- | --- | --- |
| `SvgIconProps` / `cx.svg_icon(...)` | tinted monochrome icon path | `SceneOp::SvgMaskIcon` |
| `SvgImageProps` / `cx.svg_image(...)` | original-color SVG image path | `SceneOp::SvgImage` |

This split should remain explicit even if a future generic wrapper is added later.

### 2.2 Why this split is the right durable surface

`SvgIconProps` already encodes themed/icon semantics:

- explicit color,
- inherited foreground support,
- icon-like defaults.

Multicolor SVG rendering has different semantics:

- no tint color contract,
- no `currentColor` inheritance,
- original authored colors should be preserved.

Those are different runtime surfaces, not one prop bag with a mode toggle.

### 2.3 Rejected runtime state

Do not treat the following as the target surface:

- `SvgIconProps { render_mode: ... }`,
- "just use `ImageProps`/`ImageId`" for multicolor SVG authoring,
- a helper-only path that reaches `Canvas::svg_image(...)` while `ElementContext` remains
  monochrome-only.

## 3. `ecosystem/fret-ui-kit` target authoring surface

`fret-ui-kit` should expose two distinct authoring postures above the registry/runtime contracts:

1. themed UI icon posture
   - default for reusable controls and recipe chrome,
   - keeps today's monochrome/currentColor golden path,
   - should remain what `icon(...)` means.
2. registry-authored posture
   - explicit path that honors resolved icon presentation metadata,
   - intended for brand marks, multicolor assets, and imported Iconify-style art.

The exact helper names can still be chosen during implementation, but the distinction is
non-negotiable.
The durable target is not "one helper that sometimes does surprising things".

Target rule:

- recipe/component chrome should continue to opt into themed icon behavior explicitly,
- apps should also have an explicit helper path that says "render this icon as authored".

## 4. Third-party icon-pack / import-pack target protocol

### 4.1 Required public shape

The approved durable pack shape should stay explicit and grep-friendly:

- `register_vendor_icons(&mut IconRegistry)`
- optional `register_ui_semantic_aliases(&mut IconRegistry)`
- `app::install(app: &mut fret_app::App)`
- optional `advanced::install_with_ui_services(app, services)` for bootstrap-fit installers

For reusable bundles that ship icons plus additional package-owned assets:

- prefer one composed installer/bundle surface,
- or an `InstallIntoApp`-compatible bundle type,
- instead of asking apps to replay icon registration and asset registration manually.

### 4.2 Pack metadata posture

This lane should treat pack metadata as part of the durable target, even if the exact type lands a
slice later.

At minimum, the target protocol should have a stable place for:

- vendor namespace / pack id,
- import model (`generated`, `vendored`, `manual`, or equivalent),
- and any information needed to review pack provenance and intended rendering posture.

The exact metadata type can still be refined, but the contract should not assume that pack crates
are "just a few free functions forever".

### 4.3 Import workflow rule

The approved Iconify-style workflow is:

- fetch or read source icons at build/codegen time,
- generate or vendor stable Rust/SVG artifacts into the pack crate,
- register those artifacts into the `IconRegistry`,
- expose explicit install seams for apps/bootstrap.

The following is intentionally out of scope for the framework core:

- runtime HTTP fetching from Iconify,
- dynamic vendor resolution inside `crates/fret-ui`,
- app-facing APIs that expose remote icon source identity as the core component contract.

## 5. Transitional surfaces that may exist, but should not define the end state

Allowed as transition:

- existing byte-only `resolve_svg*` helpers,
- `fret-ui-kit` helpers that still assume monochrome by default,
- pack crates without explicit metadata while the contract is being landed.

Not allowed as the long-term excuse for avoiding the refactor:

- "the lower renderer already supports it",
- "third-party packs can infer the rules from examples",
- "we can add metadata later if needed".

## 6. Rejected overall interface state

Do not converge on any of the following:

- semantic ids that leak vendor names into reusable component APIs,
- a single overloaded SVG element surface that mixes icon and image semantics,
- a registry contract that only understands bytes while policy crates guess the rest,
- a core Iconify client/runtime fetch layer,
- design-system-specific icon rules inside `crates/fret-ui`,
- app-facing docs that require users to discover pack structure by reading first-party source.
