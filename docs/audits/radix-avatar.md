# Radix Primitives Audit — Avatar

This audit compares Fret's Radix-aligned avatar substrate against the upstream Radix
`@radix-ui/react-avatar` primitive implementation pinned in `repo-ref/primitives`.

## Upstream references (source of truth)

- Implementation: `repo-ref/primitives/packages/react/avatar/src/avatar.tsx`
- Public exports: `repo-ref/primitives/packages/react/avatar/src/index.ts`

Key upstream concepts:

- `Avatar` root tracks `imageLoadingStatus` (`idle`/`loading`/`loaded`/`error`) via context.
- `AvatarImage` renders only when the image is loaded and updates the shared status.
- `AvatarFallback` renders when the image is not loaded, optionally after `delayMs`.

## Fret mapping

- Radix-named facade: `ecosystem/fret-ui-kit/src/primitives/avatar.rs`.
- Status enum: `AvatarImageLoadingStatus`.
- Delay gate: `AvatarFallbackDelay` is frame-based and driven by the caller (`App::frame_id().0`).

## Loading status design notes (Fret-specific)

Fret's `ImageId` represents an already-registered renderer resource. This means "loading" is
typically expressed at the app/asset layer, not within the `Image` element itself.

Recommended integration patterns:

- **Model-driven availability**: store `Model<Option<ImageId>>` and render `AvatarImage` only when
  the model is `Some(id)`. Use `AvatarFallbackDelay` + `fallback_visible(...)` to gate fallback.
- **Asset-cache integration**: if you use `ecosystem/fret-asset-cache` (`ImageUploadService` /
  `ImageAssetCache`), map `(pending/ready/failed)` to `AvatarImageLoadingStatus` (see
  `ecosystem/fret-app-kit/src/avatar_asset_cache.rs` for a ready-to-use adapter).

We intentionally do not force a particular image pipeline (network fetching, decoding, caching) at
the primitives layer.

