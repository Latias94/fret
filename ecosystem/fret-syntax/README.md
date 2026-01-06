# fret-syntax

Tree-sitter based syntax highlighting utilities for Fret component crates.

This crate is intentionally UI-agnostic: it returns byte ranges + highlight category names and
leaves theme token resolution to the component layer.

## Attribution

Some bundled highlight query files are derived from `gpui-component` (Apache-2.0) as a reference
source. See `docs/repo-ref.md` for the pinned checkout location under `repo-ref/`.

