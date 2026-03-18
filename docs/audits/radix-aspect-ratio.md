# Radix Primitives Audit - Aspect Ratio

## Scope

This audit compares Fret's Radix-aligned `AspectRatio` surface against:

- Radix implementation: `repo-ref/primitives/packages/react/aspect-ratio/src/aspect-ratio.tsx`
- shadcn wrapper: `repo-ref/ui/apps/v4/registry/new-york-v4/ui/aspect-ratio.tsx`
- shadcn docs/demo: `repo-ref/ui/apps/v4/content/docs/components/aspect-ratio.mdx`, `repo-ref/ui/apps/v4/registry/new-york-v4/examples/aspect-ratio-demo.tsx`

See `docs/repo-ref.md` for the local `repo-ref/` snapshot policy.

## Upstream contract summary

- Radix keeps the primitive intentionally tiny and visual-agnostic.
- The DOM implementation uses a wrapper with `position: relative`, `width: 100%`, and a padding-bottom ratio trick.
- The actual content host is absolutely positioned to all four edges.
- shadcn re-exports the Radix primitive with effectively no additional behavior.
- Upstream `ratio` is optional and defaults to `1 / 1`.

## Fret mapping

- Layout contract: `LayoutStyle.aspect_ratio` is resolved by the layout engine instead of emulating the DOM padding-bottom trick.
- Primitive facade: `ecosystem/fret-ui-kit/src/primitives/aspect_ratio.rs`.
- shadcn export surface: `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs`.
- Gallery docs/demo surface: `apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs`.

## Findings

### 1. Mechanism parity: pass

- Fret stamps `layout.aspect_ratio = Some(ratio)` on the outer container and preserves `Overflow::Visible`, matching the upstream "no implicit clipping" behavior.
- Fret also creates a full-size absolute content host so child `w_full()/h_full()` sizing resolves against an explicit containing block, matching the outcome of the Radix DOM structure.
- This is the correct layer: the behavior belongs in `fret-ui-kit` as a headless primitive, not in `crates/fret-ui` policy or in `fret-ui-shadcn` recipe code.

### 2. API shape parity: improved

- Original Fret API required `AspectRatio::new(ratio, child)`, which was functionally correct but less source-aligned with shadcn/Radix usage.
- Fret now also supports `AspectRatio::with_child(child).ratio(...)`, which restores the more prop-like call shape while keeping the existing constructor for explicit use sites.
- Fret now exposes the upstream default ratio semantics through `with_child(...) -> 1 / 1`.

### 3. Children composition: improved

- Upstream accepts normal React `children`; Fret now supports both a single landed child and a
  direct multi-child surface via `new_children(...)` / `with_children(...)`.
- This keeps the move-only `AnyElement` contract intact while making image + overlay compositions
  read closer to upstream authoring.
- Wrapping children in one composed root is still valid when the caller wants a single local layout
  boundary, but it is no longer required for the common overlay case.

### 4. Gallery/docs alignment: improved

- The gallery now follows the shadcn docs path directly: `Demo`, `Usage`, `Square`, `Portrait`,
  `RTL`, then `API Reference`.
- `API Reference` now records the Fret-specific ownership notes and the multi-child composition
  mapping instead of ending the page with an internal-only `Notes` block.
- The `RTL` preview now mirrors the upstream teaching intent more closely by pairing the 16:9 media
  frame with an Arabic caption, so the section demonstrates both ratio stability and direction-aware
  text layout instead of only showing an image inside an RTL provider.

### 5. Demo image parity: improved

- The UI Gallery preview now prefers the bundled JPEG assets that match the intended docs-style
  presentation instead of synthesizing a gradient fallback image.
- While the asset is still decoding/uploading, the demo uses the shadcn `MediaImage` loading state
  so the user sees a neutral loading surface rather than a misleading pseudo-example.
- The screenshot diag now waits for the demo image `loaded` marker before capture, which keeps the
  regression artifact aligned with the real preview outcome instead of whichever placeholder won the
  first frame race.

## Evidence anchors

- Primitive implementation: `ecosystem/fret-ui-kit/src/primitives/aspect_ratio.rs`
- Builder/default-ratio tests: `ecosystem/fret-ui-kit/src/primitives/aspect_ratio.rs`
- shadcn export: `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs`
- Gallery snippets: `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/images.rs`, `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/usage.rs`
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/basic.rs`
- Existing diag smoke: `tools/diag-scripts/ui-gallery/aspect-ratio/ui-gallery-aspect-ratio-docs-smoke.json`
- Demo screenshot gate: `tools/diag-scripts/ui-gallery/aspect-ratio/ui-gallery-aspect-ratio-demo-screenshot.json`
- RTL screenshot gate: `tools/diag-scripts/ui-gallery/aspect-ratio/ui-gallery-aspect-ratio-rtl-screenshot.json`

## Verdict

`AspectRatio` is not blocked by a mechanism bug. The remaining drift was public-surface/docs
alignment plus demo preview fidelity, not layout-engine correctness. Current status: mechanism
parity passes; the primitive now supports direct multi-child composition, the UI Gallery page
matches the shadcn docs path more closely, and the demo preview waits for real bundled media rather
than teaching with a synthetic placeholder.
