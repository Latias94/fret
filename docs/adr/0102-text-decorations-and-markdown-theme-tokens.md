# ADR 0102: Text Decorations and Markdown Theme Tokens

Status: Proposed

## Context

Fret already has a typography baseline centered around `TextStyle` (ADR 0058) and a token-driven theme
system (ADR 0032, ADR 0050, ADR 0101). `ecosystem/fret-markdown` (ADR 0099) renders Markdown into
regular element trees and intentionally relies on the existing text stack rather than introducing a
second styling system.

However, “real” Markdown and GFM content expects inline presentation features that Fret does not yet
model in `TextStyle`:

- emphasis (`*em*`) → italic / oblique,
- strikethrough (`~~del~~`) → line-through decoration,
- underlines (links or user-controlled) → underline decoration,
- task list markers (`- [x]`) → checkbox-like affordance (already a UI primitive, but needs a style
  contract),
- table rendering → structured cells with token-driven borders/backgrounds (future).

Today the markdown renderer can only approximate some of these via weight/color, which is not
sufficient for editor-grade documentation surfaces and AI chat transcripts.

We want to lock the design now, but defer implementation until the core text + renderer plumbing is
ready.

## Goals

1) Support Markdown/GFM fidelity without forking the theme system.
2) Keep Markdown theming purely token-driven and compatible with shadcn-aligned keys.
3) Avoid introducing implicit layout/scroll behavior (ADR 0099).
4) Keep renderer/backends consistent across platforms.

## Decision

### 1) Extend `TextStyle` (Typography v2 surface)

We will introduce explicit style dimensions needed by Markdown and general UI text:

- `FontStyle` (or `FontSlant`): `Normal | Italic | Oblique`.
- `TextDecoration`: supports at least:
  - underline,
  - line-through,
  - (optional) overline.
- Decoration configuration:
  - color override (default: text color),
  - thickness override (optional),
  - style (at least `Solid`, optionally `Dotted | Dashed | Wavy`).

Contract:

- Decorations are part of the renderer contract, not “fake” glyph runs.
- If a backend cannot produce a true italic face, it may synthesize oblique slant, but the API must
  still exist so Markdown can express intent.

### 2) Renderer contract for text decorations

Decorations are drawn as separate scene ops associated with text runs (not as independent widgets):

- Each text run may produce decoration quads/lines based on per-line metrics (baseline, ascent,
  descent) and font metrics (preferred underline position/thickness when available).
- Decorations must respect clipping/overflow and be ordered with text so they appear correctly under
  highlights/selection.
- Multi-line text:
  - underline and line-through apply per visual line,
  - link runs may span across line wraps; decoration should appear continuous per line.

### 3) Markdown theme token namespace (optional, with fallbacks)

`fret-markdown` will resolve styling through `Theme::color_by_key` / `Theme::metric_by_key` first,
then fall back to semantic theme keys (`Theme::color_required`) and baseline metric tokens
(`Theme::metric_required`).

We standardize the following lookup order:

1. `fret.markdown.*` (canonical, Fret-owned namespace)
2. `markdown.*` (compatibility keys for third-party theme reuse)
3. Semantic fallbacks (e.g. `foreground`, `muted-foreground`, `primary`, `border`, `card`, `muted`)
4. Baseline metric fallbacks (e.g. `metric.padding.sm`, `metric.padding.md`, `metric.font.line_height`)

We standardize the following optional tokens (names are canonical keys):

Colors:

- `markdown.muted` (fallback: `muted-foreground`)
- `markdown.link` (fallback: `primary`)
- `markdown.hr` (fallback: `border`)
- `markdown.inline_code.fg` (fallback: `foreground`)
- `markdown.inline_code.bg` (fallback: `accent`)
- `markdown.blockquote.border` (fallback: `border`)
- `markdown.table.border` (fallback: `border`)
- `markdown.table.header_bg` (fallback: `muted`)
- `markdown.task.checked` (fallback: `primary`)
- `markdown.task.unchecked` (fallback: `muted-foreground`)

Math (optional; only used when the `mathjax-svg` feature is enabled):

- `markdown.math.inline.fg` (fallback: `markdown.inline_code.fg`)
- `markdown.math.inline.bg` (fallback: `markdown.inline_code.bg`)
- `markdown.math.inline.height` (fallback: `metric.font.line_height`)
- `markdown.math.block.fg` (fallback: `foreground`)
- `markdown.math.block.bg` (fallback: `card`)
- `markdown.math.block.height` (fallback: `max(metric.font.line_height * 3.25, metric.font.size * 4.0)`)

Metrics:

- `markdown.inline_code.padding_x` (fallback: `Px(3.0)`)
- `markdown.inline_code.padding_y` (fallback: `Px(1.0)`)
- `markdown.blockquote.border_width` (fallback: `Px(3.0)`)
- `markdown.blockquote.padding` (fallback: `metric.padding.sm`)
- `markdown.table.cell.padding_x` (fallback: `metric.padding.sm`)
- `markdown.table.cell.padding_y` (fallback: `metric.padding.sm * 0.5`)
- `markdown.code_block.max_height` (default: `max(metric.font.mono_line_height * 16, metric.font.mono_size * 18)`)
- `markdown.math.inline.padding_x` (fallback: `markdown.inline_code.padding_x`)
- `markdown.math.inline.padding_y` (fallback: `markdown.inline_code.padding_y`)
- `markdown.math.block.padding` (fallback: `metric.padding.md`)

Notes:

- These tokens are optional; themes do not need to define them.
- Token keys must remain stable once shipped (ADR 0032, ADR 0050).
- We keep them distinct from `syntax.*` tokens used by `fret-code-view` so code highlighting can be
  themed independently.
- `markdown.code_block.max_height` (and its canonical `fret.markdown.*` variant) is consumed by the
  default fenced code block renderer to enable “internal scroll” for tall code blocks, without
  making Markdown itself a special scrolling surface (ADR 0099).

### 4) Markdown-to-style mapping (no implementation yet)

Once Typography v2 is implemented, `fret-markdown` will map parsed Markdown to text styles as:

- `strong` → increased `FontWeight`,
- `emphasis` → `FontStyle::Italic`,
- `strikethrough` → `TextDecoration::LineThrough`,
- links → `color=markdown.link` and optional underline via `markdown.link_underline`,
- inline code → monospace font + container background using `markdown.inline_code.*` tokens.

Task list markers will be rendered as a checkbox-like primitive with colors/spacing controlled by
the `markdown.task.*` tokens, but the selection/interaction policy remains host-defined (Markdown is
render-only by default).

## Alternatives Considered

1) **Approximate everything with weight/color**: rejected; does not meet fidelity requirements.
2) **Introduce a separate “rich text” styling system**: rejected; would fork theming and
   complicate composition.
3) **Render decorations as glyphs (text shaping hack)**: rejected; poor portability and incorrect
   metrics for selection/caret positioning.

## Consequences

- Requires cross-cutting changes in:
  - `crates/fret-core` (`TextStyle` surface),
  - `crates/fret-ui` text elements and layout/measurement,
  - renderer text submission / scene ops (decorations ordering and clipping).
- Unlocks:
  - true Markdown/GFM fidelity,
  - future editor features (spellcheck underline, diagnostics squiggles, link underlines) using the
    same decoration machinery.

## Follow-ups

- Update ADR 0058 (Typography v1) to reference this v2 extension once implemented.
- Add a focused “text decorations rendering contract” appendix to ADR 0029 (text pipeline) if needed.
- Extend `fret-markdown` to consume GFM events (`TaskListMarker`, `Strikethrough`, tables) once the
  typography primitives exist.
