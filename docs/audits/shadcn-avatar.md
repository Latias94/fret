# shadcn/ui v4 Audit — Avatar

This audit compares Fret’s shadcn-aligned `Avatar` against the upstream shadcn/ui v4 docs and the
`new-york-v4` registry implementation in `repo-ref/ui`.

## Upstream references (source of truth)

- Docs page: `repo-ref/ui/apps/v4/content/docs/components/avatar.mdx`
- Registry implementation (new-york): `repo-ref/ui/apps/v4/registry/new-york-v4/ui/avatar.tsx`
- Underlying primitive: `repo-ref/primitives/packages/react/avatar/src/avatar.tsx`

## Fret implementation

- Component code: `ecosystem/fret-ui-shadcn/src/avatar.rs`
- Radix helpers: `ecosystem/fret-ui-kit/src/primitives/avatar.rs`

## Audit checklist

### Composition surface

- Pass: Exposes `Avatar`, `AvatarImage`, `AvatarFallback`.
- Pass: Root is overflow-clipped and fully rounded (shadcn `rounded-full` outcome).
- Pass: Image uses `aspect-square` and fills the root.

### Image loading + fallback

- Pass (Fret-specific): Supports model-driven image availability via `AvatarImage::model(Model<Option<ImageId>>)`
  and optional image via `AvatarImage::maybe(Option<ImageId>)`.
- Pass: `AvatarFallback` can be gated to the “image not loaded” outcome via
  `AvatarFallback::when_image_missing_model(...)` / `when_image_missing(...)`.
- Pass: `AvatarFallback` supports an optional delay via `delay_ms(...)` / `delay_frames(...)`,
  approximating Radix `delayMs`.

## Validation

- `cargo test -p fret-ui-shadcn --lib avatar`

