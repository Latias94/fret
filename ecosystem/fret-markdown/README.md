# fret-markdown

Markdown renderer component(s) for Fret.

Current focus: fenced code blocks with optional tree-sitter highlighting via `fret-syntax`.

This crate is intentionally “layout-normal”: it renders into regular `AnyElement` trees and does not
create implicit scroll containers. Scrolling is decided by the host (wrap Markdown inside a scroll
panel if desired).

## Usage

Use `MarkdownComponents` to customize rendering and policies (links, images, code blocks, etc.).

```rust
let mut components = fret_markdown::MarkdownComponents::<App>::default().with_open_url();

// Default fenced code blocks are rendered via `fret-code-view`.
components.code_block_ui.wrap = fret_code_view::CodeBlockWrap::ScrollX;
components.code_block_ui.show_scrollbar_x = true;

// Optional: cap code block height and enable internal Y scrolling.
// Prefer the theme token `fret.markdown.code_block.max_height` / `markdown.code_block.max_height`.
components.code_block_ui.max_height = Some(fret_core::Px(360.0));

// Optional: add an “actions” area for code fences (copy, download, expand, …).
components.code_block_actions = Some(Arc::new(|cx, info| {
    // return AnyElement
    todo!()
}));

markdown::Markdown::new(source).into_element_with(cx, &components);
```

## Theme tokens

`fret-markdown` resolves tokens in this order:

1. `fret.markdown.*` (canonical, Fret-owned namespace)
2. `markdown.*` (compatibility keys for third-party theme reuse)
3. Semantic fallbacks (e.g. `foreground`, `primary`, `border`, …)

Code blocks:

- `fret.markdown.code_block.max_height` / `markdown.code_block.max_height` (optional): when set,
  caps code block height and enables internal vertical scrolling.

See `docs/adr/0103-text-decorations-and-markdown-theme-tokens.md` for the full token list and
compatibility rules.
