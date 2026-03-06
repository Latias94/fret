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

### 3. Children composition: acceptable with documented guidance

- Upstream accepts normal React `children`; in Fret the primitive stores one `AnyElement` root.
- This is not a parity bug. In a move-only element tree, the natural translation is to pass one composed wrapper element, for example image plus overlay chrome in a stack, as the child root.
- Recommendation: keep the single-child root model and document the wrapper composition pattern rather than introducing a special multi-child mechanism just for this primitive.

### 4. Gallery/docs alignment: improved, with intentional extras

- The gallery now keeps the official 16:9 demo first and adds a dedicated `Usage` section before the Fret-specific `Square`, `Portrait`, and `RTL` extras.
- This keeps the first-screen reading order closer to shadcn docs while still preserving valuable local stress cases for constrained widths and directionality.

## Evidence anchors

- Primitive implementation: `ecosystem/fret-ui-kit/src/primitives/aspect_ratio.rs`
- Builder/default-ratio tests: `ecosystem/fret-ui-kit/src/primitives/aspect_ratio.rs`
- shadcn export: `ecosystem/fret-ui-shadcn/src/aspect_ratio.rs`
- Gallery page: `apps/fret-ui-gallery/src/ui/pages/aspect_ratio.rs`
- Gallery snippets: `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/demo.rs`, `apps/fret-ui-gallery/src/ui/snippets/aspect_ratio/usage.rs`
- Existing layout gate: `ecosystem/fret-ui-shadcn/tests/web_vs_fret_layout/basic.rs`
- Existing diag smoke: `tools/diag-scripts/ui-gallery/aspect-ratio/ui-gallery-aspect-ratio-docs-smoke.json`

## Verdict

`AspectRatio` is not blocked by a mechanism bug. The main drift was API/readability/docs alignment, not layout-engine correctness. Current status: mechanism parity passes; API/docs parity is in review and materially improved.
