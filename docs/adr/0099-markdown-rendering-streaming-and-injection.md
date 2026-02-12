# ADR 0099: Markdown Rendering, Streaming, and Component Injection

Status: Accepted

## Context

Fret aims to be a cross-platform, editor-grade UI framework. We need a reusable “rich content surface” that can power:

- LLM chat transcripts (streaming, token-by-token or chunk-by-chunk),
- docs/help panels and popovers,
- release notes / onboarding pages,
- markdown-driven descriptions inside component surfaces (shadcn-aligned UI).

Key constraints and requirements:

1) **Composable layout**: Markdown content must be embeddable inside arbitrary layouts (taffy/flex, shadcn primitives).
2) **AI-friendly streaming**: Incremental updates must avoid O(n²) “reparse + rerender the entire document” patterns.
3) **Stable UI subtrees**: Once a block is “committed”, its UI subtree should remain stable (cacheable + keyed).
4) **Injection points**: Apps must be able to render certain nodes with their own components (callouts, code block actions, custom links/images).
5) **Syntax highlighting**: Code fences should support tree-sitter highlighting with language injections.
6) **Security posture**: Markdown rendering must not execute content or perform I/O by default.
7) **Selection & copy** (expected): Rich content surfaces should support mouse selection and
   clipboard extraction without coupling selection semantics to the element tree structure (see ADR 0108).

We reference:

- `gpui-component`: a “TextView” style approach with a block/inline document model, virtualization, and a code-block actions injection hook.
- `streamdown`: a block-splitting + component-mapping approach, designed for streaming markdown in React.
- `mdstream`: a Rust streaming-first Markdown middleware with committed/pending blocks, transformers, and adapter patterns.
- Zed’s markdown and preview surfaces (non-normative engineering reference):
  - `repo-ref/zed/crates/markdown`
  - `repo-ref/zed/crates/markdown_preview`

## Decision

### 1) Crate boundaries (ecosystem-first, UI-agnostic core)

We standardize three layers:

1. `ecosystem/fret-syntax`
   - UI-agnostic tree-sitter highlighting: `highlight(source, language) -> Vec<HighlightSpan>`.
   - Bundles `*.scm` query files (highlight + injections) with tracked upstream sources and license texts under:
     - `ecosystem/fret-syntax/third_party/README.md`
     - `ecosystem/fret-syntax/third_party/licenses/`

2. `ecosystem/fret-code-view`
   - UI component for read-only code blocks (monospace, optional line numbers).
   - Delegates syntax highlighting to `fret-syntax`.
   - Must remain layout-friendly (width/height controlled by parent).

3. `ecosystem/fret-markdown`
   - A Markdown-to-elements renderer.
   - Must treat code fences as code blocks rendered via `fret-code-view`.
   - Must remain layout-friendly (embeddable under shadcn/flex).

### 2) Layout contract: Markdown is a “normal element”

`fret-markdown` must render into standard `AnyElement` trees with no “special layout context”.

- No implicit window-level portals.
- No implicit scrolling.
- No assumptions about parent size.

Scroll behavior is decided by the caller (e.g. wrap Markdown inside a scroll container / panel / sheet).

### 3) Theming & styling contract

Markdown styling must be driven by:

- global theme tokens (`Theme::color_by_key`, `Theme::metric_by_key`) and typed theme surfaces (see ADR 0032),
- a small, stable set of defaults for spacing and typography that align with existing component scales.

Syntax colors:

- `fret-code-view` resolves tree-sitter highlight tags via `color.syntax.<tag>`, with reasonable fallbacks when missing.

### 4) Injection points (component-level rendering hooks)

We adopt a “component mapping” model (streamdown-style) at the block level, with a minimal set of stable hooks.

`fret-markdown` should define a public options/config surface (exact names are not yet finalized) that enables:

- **Block render overrides**:
  - heading / paragraph / list / blockquote / table / thematic break / code fence / HTML block (if enabled).
  - Unknown blocks should have a safe fallback rendering.
- **Code block actions** (gpui-component-inspired):
  - Provide an optional “actions area” element for code fences (copy, download, run, expand, etc.).
- **Link handling**:
  - Rendering may style links, but link activation must be delegated to the host (no navigation side-effects by default).
- **Images and external resources**:
  - The markdown layer does not fetch network resources.
  - Provide a resolver hook so hosts can decide how to load and cache images (or render placeholders).

Injection hooks must:

- be pure rendering hooks (no implicit I/O),
- keep the element tree stable when keyed (see next section),
- avoid forcing any single “design system” (shadcn can be used by the host via hooks).

### 5) Streaming contract: committed blocks + pending block (mdstream model)

We standardize a streaming-friendly document model:

- **Committed blocks**: append-only and immutable once committed.
- **Pending block**: may be updated on each new chunk; only this block is expected to change frequently.

Footnotes and reference definitions are inherently cross-block features. Until mdstream’s invalidation
support is fully realized, its default configuration may choose stability over block splitting.
For UI rendering in Fret, `ecosystem/fret-markdown` intentionally configures mdstream to keep blocks
(`FootnotesMode::Invalidate`) so headings, lists, code fences, and math blocks remain independently
layoutable and key-stable.

To avoid O(n²) and preserve stable UI subtrees:

- Each committed block must have a stable `BlockId` used as the keyed identity in the UI tree.
- Rendering must use keyed iteration for committed blocks (Fret: `ElementContext::for_each_keyed`) so block subtrees do not remount.
- Pending block may be keyed by its current `BlockId` and replaced when the stream commits it.

Recommended integration:

- Use `mdstream` as the streaming middleware:
  - `MdStream` for chunk splitting + pending termination/repair.
  - `DocumentState` for UI-friendly application of updates.
  - Optional: `PulldownAdapter` for cached parsing per committed block.

`fret-markdown` should support:

- static mode (single parse),
- streaming mode (render from a `DocumentState` / `(committed, pending)` view).

For static mode, finalize the stream once so the trailing pending block becomes committed (so block
classification like `MathBlock` is reliable).

### 6) Security posture and HTML handling

Default behavior:

- treat raw HTML as plain text or drop it (exact choice must be explicit and documented),
- do not execute scripts or load external resources,
- do not implement CSS support.

If HTML is supported in the future, it must be:

- opt-in,
- sanitized to an allowlist,
- rendered as a constrained subset.

## Consequences

- Markdown becomes a first-class, composable content surface that integrates naturally with shadcn/flex layouts.
- LLM chat UIs can stream content efficiently without full-document reparse and without subtree thrash.
- Apps can inject custom components and actions without forking the renderer.
- We defer “full rich text” (inline runs across multiple styles) as an optimization/ergonomics layer, not as a prerequisite for block-level composition.

Trade-offs:

- Initial MVP may render inline formatting conservatively until a richer inline model (runs) is introduced.
- HTML support remains intentionally minimal/opt-in to preserve security and portability.

## Alternatives Considered

1) **Full HTML renderer with CSS**: too large a surface area; portability and security risks.
2) **Single monolithic “TextView” renderer**: harder to inject arbitrary components and to reuse shadcn primitives.
3) **Re-parse entire markdown on each chunk**: acceptable for static docs but not for LLM streaming (latency + flicker).
4) **Reuse Zed’s query corpus directly**: not compatible with our licensing posture for bundled sources.

## Follow-ups

- Define the concrete `MarkdownOptions` / `MarkdownComponents` API in `fret-markdown` (block overrides + code actions + link/image hooks).
- Add a streaming renderer entry point that consumes `mdstream::DocumentState` and renders keyed committed blocks.
- Add a demo: a chat-like “streaming markdown” panel in `apps/fret-examples`.
- Add basic tests for fence language parsing and block key stability (where feasible).
