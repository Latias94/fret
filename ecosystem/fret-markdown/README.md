# fret-markdown

Markdown renderer component(s) for Fret.

Current focus: fenced code blocks with optional tree-sitter highlighting via `fret-syntax`.

This crate is intentionally “layout-normal”: it renders into regular `AnyElement` trees and does not
create implicit scroll containers. Scrolling is decided by the host (wrap Markdown inside a scroll
panel if desired).

## Mermaid

By default, fenced code blocks with language `mermaid` are rendered as SVG via `merman` (headless).
Rendering runs in the background and is cached via `fret-query`.
Note: Mermaid SVG often uses `<foreignObject>` for labels; `fret-markdown` uses a best-effort
text overlay fallback so diagrams remain readable in headless renderers.

Note: `merman` is currently pulled in as a workspace path dependency (`../merman`). Clone the
`merman` repo next to `fret` (same parent directory) to build Mermaid support.

- Disable Mermaid rendering:

```toml
fret-markdown = { path = "...", default-features = false }
```

- Re-enable Mermaid explicitly:

```toml
fret-markdown = { path = "...", default-features = false, features = ["mermaid"] }
```

## Usage

Use `MarkdownComponents` to customize rendering and policies (links, images, code blocks, etc.).

```rust
let components = fret_markdown::MarkdownComponents::<App>::default()
    .with_open_url()
    // Default fenced code blocks are rendered via `fret-code-view`.
    .with_code_block_wrap(fret_code_view::CodeBlockWrap::ScrollX)
    .with_code_block_scrollbar_x(true)
    // Optional: cap code block height via theme (default: ~16 lines) and enable internal Y scrolling.
    // Theme keys:
    // - canonical: `fret.markdown.code_block.max_height`
    // - compat: `markdown.code_block.max_height`
    .with_code_block_max_height_from_theme(true)
    .with_code_block_scrollbar_y(true);

// Optional: add an “actions” area for code fences (copy, download, expand, …).
let components = fret_markdown::MarkdownComponents {
    code_block_actions: Some(Arc::new(|cx, info| {
        // `info.id` is a stable block identifier (useful for per-block state like expand/collapse).
        // return AnyElement
        todo!()
    })),
    // Optional: tweak the default code block renderer per fence (expand/collapse, wrap overrides, …).
    code_block_ui_resolver: Some(Arc::new(|_cx, _info, _options| {
        // mutate options
    })),
    ..components
};

markdown::Markdown::new(source).into_element_with(cx, &components);
```

## Theme tokens

`fret-markdown` resolves tokens in this order:

1. `fret.markdown.*` (canonical, Fret-owned namespace)
2. `markdown.*` (compatibility keys for third-party theme reuse)
3. Semantic fallbacks (e.g. `foreground`, `primary`, `border`, …)

Code blocks:

- `fret.markdown.code_block.max_height` / `markdown.code_block.max_height` (default: derived from
  mono font metrics, roughly 16 lines): caps code block height and enables internal vertical scrolling.
- `MarkdownComponents.code_block_ui.show_scrollbar_y` (optional): when enabled and `max_height` is
  set, show a vertical scrollbar for tall code blocks (hover-visible by default, matching shadcn / Radix).
- `MarkdownComponents.code_block_max_height_from_theme` (default: true): disable this to opt out of
  theme-driven `max_height` resolution.

See `docs/adr/0103-text-decorations-and-markdown-theme-tokens.md` for the full token list and
compatibility rules.
