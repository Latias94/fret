# ADR 0101: Semantic Theme Keys, Extensible Token Registry, and shadcn/new-york Alignment

Status: Accepted
Scope: `fret-ui` theme resolution + config keys; impacts component ecosystems (`fret-ui-kit`, `fret-ui-shadcn`)

## Context

Fret currently has:

- a small **typed baseline** theme surface (ADR 0050) (`ThemeColors`, `ThemeMetrics`),
- plus an **open-ended** `(String -> Color/Px)` store (`ThemeConfig.colors/metrics`) for extension tokens,
- plus **best-effort semantic aliases** intended to bridge gpui-component and shadcn-style vocabulary.

This hybrid approach worked to bootstrap the UI kit, but it has started to show structural issues:

1) **Two sources of truth drift**: some code reads typed fields (`theme.colors.*`), other code reads semantic keys
   (`theme.color_by_key("popover-foreground")`), and the same theme config can produce inconsistent UI depending
   on which read path a widget happens to use.

2) **shadcn/new-york alignment pressure**: shadcn/ui v4 (and its new-york-v4 style) is built around a stable set
   of semantic CSS variables (`--background`, `--popover-foreground`, ...). The Fret shadcn port should align with
   that vocabulary to minimize translation overhead and enable “copy the token sheet” workflows.

3) **Editor-scale extensibility**: some apps will be editor-like (code editor, node graph, timeline), where
   hundreds of tokens may exist (syntax scopes, gutter annotations, diagnostics, minimap, etc.). We need an
   extensible theming contract that:

   - allows user and plugin-defined tokens without changing `fret-ui`,
   - remains performant in hot paths,
   - avoids stringly-typed usage in Rust UI code (discoverability + correctness).

4) **Future layering**: theme files are expected to be layered (ADR 0014). We need a stable scheme for defaults,
   base style selection, and overrides that does not rely on ad-hoc per-widget fallbacks.

Reference baseline (local, pinned):

- shadcn/ui v4 site and registry: `repo-ref/ui/apps/v4/`
- new-york base variables (light/dark): `repo-ref/ui/apps/v4/styles/globals.css`
- Zed theme system (non-normative engineering reference):
  - theme schema + registry: `repo-ref/zed/crates/theme/src/schema.rs`, `repo-ref/zed/crates/theme/src/registry.rs`
  - syntax/theme style tables: `repo-ref/zed/crates/theme/src/styles`

## Decision

### 1) Adopt semantic-first theming: semantic token names are the canonical identity

The canonical token names used by the theme system SHOULD align with shadcn semantic variables:

- canonical names SHOULD match shadcn when applicable (e.g. `popover-foreground`)
- a token name MUST be stable once published
- theme files store keys by canonical name; internal IDs are derived at runtime

Canonical token names are **strings**, not Rust identifiers. We therefore allow two complementary
key styles:

- **shadcn-style** keys: kebab-case, no namespace (e.g. `muted-foreground`)
- **namespaced** keys: dot-separated namespaces (e.g. `editor.gutter.foreground`)

Both styles are canonical; shadcn-style keys are preferred for cross-ecosystem portability, while
namespaced keys are preferred for app- and component-specific extension.

This ADR does **not** mandate that Fret must look exactly like the web new-york palette; it mandates that the
**vocabulary** is aligned so that palettes can be ported and defaults can be shared across ecosystems.

### 2) Introduce typed core keys for semantic tokens (stable, discoverable)

Fret will define typed key sets that represent the stable **core** token set:

- `ThemeColorKey` (colors)
- `ThemeMetricKey` (metrics)

Widgets and component recipes should prefer typed keys over string lookups.

`ThemeColorKey` must cover the shadcn semantic palette used broadly across the UI kit:

- `background`, `foreground`
- `card`, `card-foreground`
- `popover`, `popover-foreground`
- `primary`, `primary-foreground`
- `secondary`, `secondary-foreground`
- `muted`, `muted-foreground`
- `accent`, `accent-foreground`
- `destructive`, `destructive-foreground`
- `border`, `input`, `ring`, `ring-offset-background`

`ThemeMetricKey` must include the shadcn baseline metric(s) required by the component system:

- `radius` (and optionally derived radii such as `radius-sm`, `radius-md`, `radius-lg`)
- baseline typography metrics needed by `fret-ui` widgets (e.g. `font.size`, `font.line-height`, monospace variants)

Notes:

- The core key set is not intended to contain every possible token; it is the “common denominator” that ensures
  cross-component consistency.
- New keys added to the core key set must be treated as a compatibility commitment.

### 3) Make a token registry the single source of truth (typed fields become derived views)

The authoritative theme state is a **resolved token table**, not a struct of ad-hoc fields.

We will introduce a registry/resolution model:

- `ThemeRegistry`: defines which tokens exist, their kinds (color/metric), their default values, and aliases.
- `ThemeConfig`: provides user-specified overrides by name (strings).
- `ThemeSnapshot` / `ResolvedTheme`: is the resolved output used by layout/paint code.

Typed “baseline fields” (ADR 0050 `ThemeColors`/`ThemeMetrics`) become:

- either removed,
- or retained as a derived cache that is populated from the resolved token table for hot-path access and
  transitional compatibility.

Critically: there must be exactly one truth for “what is the border color”, and both old and new callsites must
observe the same resolved value.

### 4) Support extensible tokens via namespaced keys (without expanding the core key set)

In addition to the typed core keys, the system must support arbitrary user-defined keys.

Contract:

- Theme files MAY include unknown keys; they are preserved and resolved if registered, otherwise ignored.
- Component ecosystems and apps MAY register additional tokens at startup.
- Extensions MUST be namespaced to avoid collisions.

Recommended namespaces:

- `component.<name>.*` (component recipe-level tokens)
- `editor.*` (editor chrome beyond generic UI)
- `syntax.*` (syntax highlighting / scopes; see §8)
- `fret.viewport.*` (editor viewport overlays that are intentionally not shadcn-like)
- `chart-*`, `sidebar-*`, etc. (style-specific families used by new-york)

### 5) Canonical name normalization and aliasing (compat + ergonomics)

To keep migration and ecosystem ingestion practical:

- canonical identity is the shadcn-style kebab-case name (e.g. `popover-foreground`)
- the registry may define aliases that normalize common variants:
  - kebab-case vs dotted: `popover.foreground` -> `popover-foreground`
  - kebab-case vs snake_case: `ring_offset_background` -> `ring-offset-background`
  - legacy dotted keys from ADR 0050: `color.surface.background` -> `background`, etc.

Theme files may still use the legacy dotted keys during migration, but canonical documentation and new
themes should converge on semantic names.

### 6) Theme config schema remains flat (string keys), with optional versioning and base style selection

We keep the existing shape from ADR 0050:

- `colors: { "<key>": "<color token>" }`
- `metrics: { "<key>": <f32 px> }`

We add two optional fields for future-proofing:

- `schema_version` (integer; default 1)
- `extends` (string or array of strings; style/base theme identifiers)

Examples (non-normative):

```json
{
  "schema_version": 2,
  "name": "My Theme",
  "extends": ["shadcn/new-york-v4-dark"],
  "colors": {
    "background": "oklch(0.145 0 0)",
    "foreground": "oklch(0.985 0 0)",
    "editor.selection": "#3D8BFF66"
  },
  "metrics": {
    "radius": 10
  }
}
```

`extends` resolution and theme layering order is specified in §7.

### 7) Layering and precedence

Theme resolution composes layers in this precedence order (highest wins):

1. explicit per-component overrides (structured style props, not arbitrary strings)
2. window overrides (future; per-window DPI/density aware)
3. project theme
4. user theme
5. base style theme (e.g. `shadcn/new-york-v4`)
6. built-in framework fallback defaults

This order matches the intent of ADR 0032/0050 and keeps “style” (a cohesive baseline) distinct from user overrides.

### 8) Heavy editor theming: separate `SyntaxTheme` (do not overload the UI token table)

Syntax highlighting is a different problem than UI theming:

- token count is high (scopes, modifiers, diagnostics)
- values are often `TextStyle`-like (foreground + weight + italics + underline), not just colors
- it is frequently imported from existing formats (VS Code, TextMate, Zed themes)

Decision:

- introduce a separate `SyntaxTheme` (and potentially `EditorTheme`) subsystem that is **app/ecosystem-owned**
  and may reference core UI colors (e.g. `ThemeColorKey`) as fallbacks, but does not dump thousands of keys into
  the base UI table.

This keeps the core UI theme ergonomic and the syntax theme optimized for editor workloads.

### 9) Deterministic invalidation: a single theme revision is the cache key

Theme changes must increment a monotonic `theme_revision` that participates in:

- layout caches,
- paint caches,
- prepared text/blob caches where styling affects glyph output.

Any registry changes (new tokens registered) that affect resolution must also bump the revision.

## Implementation Status

- Implemented: canonical key aliasing via `crates/fret-ui/src/theme_registry.rs`.
- Implemented: required token accessors via `crates/fret-ui/src/theme.rs` (`Theme::color_required`, `Theme::metric_required`).
- Implemented: `ecosystem/fret-ui-shadcn` no longer reads typed theme fields (`theme.colors.*` / `theme.metrics.*`).
- Implemented: `ecosystem/fret-ui-kit` is also token-based (no direct `theme.colors.*` / `theme.metrics.*` reads in the UI-kit surface).
- TODO: add a CI guard that prevents reintroducing typed reads in shadcn-aligned crates (at least `fret-ui-shadcn` + `fret-ui-kit`).

## Design Outline (Non-Normative)

This section sketches the intended shape; exact names may change during implementation.

### Core data model

- `ThemeRegistry`
  - registers tokens by `(kind, canonical_name)`
  - stores:
    - default value source (built-in, style crate, app)
    - alias names (strings)
    - documentation metadata (optional): “used by component X”
- `ThemeConfig`
  - stores raw overrides by name, in file-friendly formats (hex, `hsl(...)`, `oklch(...)`)
  - may include `extends` / style selection hints
- `ResolvedTheme`
  - a resolved table for colors and metrics
  - provides:
    - `color(ThemeColorKey) -> Color`
    - `metric(ThemeMetricKey) -> Px`
    - `color_by_name(&str) -> Option<Color>` (for extension tokens)

### Key types

- `ThemeColorKey` and `ThemeMetricKey` (core enum/const sets)
- `ThemeTokenId` (for registered extension tokens; not serialized)
- `ThemeTokenName` (string; serialized identity)

### Performance considerations

- Hot-path UI code must not allocate or hash strings during paint/layout.
- Core tokens (`ThemeColorKey`/`ThemeMetricKey`) should be resolved to direct indices.
- Extension tokens can be resolved via an interned ID or cached lookup, with resolution happening outside the
  tightest loops (e.g. in component render functions, not per-glyph).

## Compatibility and Migration Plan (Phased)

This ADR is intentionally migration-friendly. We will phase it so we can ship incremental improvements without
breaking existing themes or component code.

### Phase 0 (already in flight): semantic aliases and backfill

Keep the current bridge that maps shadcn semantic names onto existing baseline tokens, and ensure typed reads
and semantic reads do not drift.

### Phase 1: introduce core typed keys and resolve through one canonical table

- Add `ThemeColorKey`/`ThemeMetricKey` and `ThemeRegistry`.
- Make `Theme::apply_config` resolve into a single table.
- Keep `ThemeColors/ThemeMetrics` as derived caches populated from the table.

### Phase 2: migrate component ecosystems to typed keys

- `fret-ui-shadcn` and `fret-ui-kit` should prefer core typed keys or registered token IDs over string literals.
- Add linting guidance and tests to prevent new stringly-typed keys in hot paths.

### Phase 3: deprecate legacy dotted keys in docs and examples

- Document canonical shadcn key names.
- Keep dotted keys as aliases for a transition window.
- Provide a small conversion script/tooling (optional) to rewrite theme JSON keys.

### Phase 4: introduce `SyntaxTheme` and editor-specific theming surfaces

- Define a dedicated schema and runtime representation for syntax styles.
- Keep UI theme clean; allow references to UI palette tokens.

## Alternatives Considered

1) **Keep typed baseline tokens as the only truth**
   - Pros: fastest, strongly typed.
   - Cons: constant pressure to expand the typed surface; shadcn/new-york alignment becomes a perpetual mapping
     layer; extension tokens remain second-class and drift continues.
   - Decision: rejected as a long-term design.

2) **String keys only (no typed core keys)**
   - Pros: fully extensible, easiest to mirror shadcn naming.
   - Cons: poor discoverability, easy to typo, hot-path string hashing risk, harder to refactor.
   - Decision: rejected; we keep typed core keys.

3) **Adopt shadcn names but keep the old dotted schema as canonical**
   - Pros: minimal file churn.
   - Cons: the ecosystem vocabulary remains mismatched; component alignment continues to require translation.
   - Decision: rejected; canonical identity becomes shadcn-like names, with dotted keys treated as aliases.

## Consequences

- UI kit components and themes can align directly with shadcn/new-york vocabulary.
- Theme files become more portable across ecosystems and easier for users to author.
- Extension tokens become first-class via registration and namespacing, without expanding `fret-ui`.
- The theme system becomes a stronger contract for editor-scale apps (many components, many themes).

## Appendix A: shadcn/new-york base variables

The new-york-v4 baseline variables (light/dark) are defined in:

- `repo-ref/ui/apps/v4/styles/globals.css`

The core semantic palette listed in §2 is directly derived from that vocabulary.

## Appendix B: Compatibility mapping from ADR 0050 dotted keys (illustrative)

This mapping is provided to keep older themes functional while components migrate toward the shadcn vocabulary.
Exact mappings may evolve as we stabilize the new registry, but the goal is to minimize surprises and avoid
per-component ad-hoc fallbacks.

**Core surfaces**

- `color.surface.background` -> `background`
- `color.text.primary` -> `foreground`
- `color.panel.background` -> `card`
- `color.panel.border` -> `border` (and `input` by default)

**Menus / popovers**

- `color.menu.background` -> `popover`
- `color.menu.border` -> `border`

**Lists / hover / selection**

- `color.hover.background` -> `accent`
- `color.selection.background` -> `primary` (best-effort; some apps may prefer `accent`)
- `color.list.background` -> `background` (best-effort)
- `color.list.row.hover` -> `accent`
- `color.list.row.selected` -> `primary` (best-effort)

**Focus**

- `color.focus.ring` -> `ring`

**Viewport overlays (Fret/editor-specific)**

The viewport overlay keys from ADR 0050 remain valid and should move under a stable namespace such as:

- `fret.viewport.selection.fill`
- `fret.viewport.selection.stroke`
- `fret.viewport.marker`
- `fret.viewport.drag_line.pan`
- `fret.viewport.drag_line.orbit`
- `fret.viewport.gizmo.x`
- `fret.viewport.gizmo.y`
- `fret.viewport.gizmo.handle.background`
- `fret.viewport.gizmo.handle.border`
- `fret.viewport.rotate_gizmo`

## Appendix C: new-york “extended” token families (non-core)

shadcn/new-york-v4 defines additional variables beyond the core semantic palette. They are widely used by the
site and some components, but they are not required for a minimal portable UI kit.

We treat these as **style-provided extensions**:

- they should be registered (with defaults) by the shadcn/new-york style bundle,
- they are addressable by name (and optionally by strongly-typed constants in the style crate),
- they are not part of the `ThemeColorKey`/`ThemeMetricKey` core set unless they prove universally necessary.

Examples observed in `repo-ref/ui/apps/v4/styles/globals.css`:

- `chart-1` .. `chart-5` (chart palettes)
- `sidebar`, `sidebar-foreground`, `sidebar-primary`, `sidebar-primary-foreground`, `sidebar-accent`,
  `sidebar-accent-foreground`, `sidebar-border`, `sidebar-ring`
- `surface`, `surface-foreground` (non-shadcn core “surface” semantics used by the site)
- `code`, `code-foreground`, `code-highlight`, `code-number`
- `selection`, `selection-foreground`
