# fret-syntax

Tree-sitter based syntax highlighting utilities for Fret component crates.

This crate is intentionally UI-agnostic: it returns byte ranges + highlight category names and
leaves theme token resolution to the component layer.

## Attribution

This crate bundles a set of Tree-sitter query files (`*.scm`) for highlighting and language
injections. Sources and license texts are recorded in:

- `ecosystem/fret-syntax/third_party/README.md`
- `ecosystem/fret-syntax/third_party/licenses/`
