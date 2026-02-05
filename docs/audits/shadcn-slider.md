# shadcn/ui v4 Audit — Slider

This audit compares Fret’s shadcn-aligned `Slider` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/slider.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/slider.tsx`
- Underlying primitive: Radix `@radix-ui/react-slider`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/slider.rs`
- Shared primitives:
  - Radix-aligned slider semantics/value updates: `ecosystem/fret-ui-kit/src/primitives/slider.rs`
  - Pointer-to-value mapping helpers: `ecosystem/fret-ui-kit/src/declarative/slider.rs`

## Audit checklist

### Layout & geometry (shadcn parity)

- Pass: Track height defaults to `h-1.5` (6px) via `component.slider.track_height`.
- Pass: Thumb defaults to `size-4` (16px) via `component.slider.thumb_size`.
- Pass: Layout height follows the track; the thumb is allowed to overflow without being clipped
  (overflow-visible semantics), matching the DOM implementation.
- Pass: Thumb stays visually in-bounds at the edges (Radix `getThumbInBoundsOffset` outcome), so the
  center-aligned thumb does not underflow/overflow the track at `t=0` / `t=1`.

### Semantics

- Pass: Exposes `SemanticsRole::Slider`, `value` text, and focusability via the root semantics node.

## Validation

- `cargo test -p fret-ui-shadcn --lib slider`
- Web layout gate: `cargo nextest run -p fret-ui-shadcn --test web_vs_fret_layout`
  (`web_vs_fret_layout_slider_demo_geometry`).
- Web layout gate (thumb insets): `cargo nextest run -p fret-ui-shadcn -E "test(web_vs_fret_layout_field_slider_thumb_insets_match_web)"`

## Follow-ups (recommended)

- Add a Radix-web gate for keyboard step behavior (e.g. ArrowRight) once we have a stable event
  harness for non-overlay primitives.
