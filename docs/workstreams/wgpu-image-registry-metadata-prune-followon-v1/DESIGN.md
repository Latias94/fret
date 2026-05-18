# WGPU Image Registry Metadata Prune Follow-on v1

Status: Closed
Last updated: 2026-05-18

## Why This Lane Exists

`ImageRegistry` kept `ImageColorSpace` inside each registered image entry after it had already used
that value to validate the texture format at registration/update time.

That made the registry interface shallower than it needed to be: callers still provide color-space
metadata through `ImageDescriptor`, but the retained registry state only needs the texture view,
size, format, and alpha mode for later render encoding.

## Assumptions First

- Confident: `ImageDescriptor.color_space` remains an input contract. Evidence:
  `ImageRegistry::register` and `ImageRegistry::update` still assert that the texture format matches
  the declared color space. If wrong, callers could register inconsistent image metadata silently.
- Confident: retained `ImageEntry.color_space` had no runtime reader. Evidence: `rg` found readers
  for `format` and `alpha_mode`, but no post-registration reader for the retained color-space field.
  If wrong, `cargo check -p fret-render-wgpu --locked --tests -j 1` would fail after field removal.
- Likely: no ADR update is needed because this does not change renderer contracts or public
  `ImageDescriptor` shape.

## Target State

- `ImageEntry` stores only metadata read by render encoding, diagnostics, or bind-group selection.
- `ImageDescriptor.color_space` remains part of the input contract and validation path.
- The `dead_code` suppressions on `ImageEntry` metadata fields are removed.

## Out Of Scope

- Changing image upload APIs or `ImageDescriptor` public fields.
- Reworking render-target metadata.
- Changing alpha-mode or image-format behavior in mask/image/custom-effect paths.

## Closure Policy

Close this lane once the metadata field is removed and the `fret-render-wgpu` test compile gate
passes.

## Closure

Closed on 2026-05-18 after pruning retained image color-space state from `ImageRegistry`.
