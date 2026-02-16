# Radix Primitives Audit — Avatar


## Upstream references (non-normative)

This document references optional local checkouts under `repo-ref/` for convenience.
Upstream sources:

- Radix UI Primitives: https://github.com/radix-ui/primitives

See `docs/repo-ref.md` for the optional local snapshot policy and pinned SHAs.
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
- **UI-assets integration**: prefer `ecosystem/fret-ui-assets` (re-export surface over
  `ecosystem/fret-asset-cache`: `ImageUploadService` / `ImageAssetCache`), mapping
  `(pending/ready/failed)` to `AvatarImageLoadingStatus`.

We intentionally do not force a particular image pipeline (network fetching, decoding, caching) at
the primitives layer.

