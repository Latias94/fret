# Component Ecosystem State Integration v1

Status: Draft (workstream note; not an ADR)
Last updated: 2026-02-06

Related:

- `docs/workstreams/state-management-v1.md`
- `docs/workstreams/state-management-v1-extension-contract.md`
- `docs/adr/0066-fret-ui-runtime-contract-surface.md`
- `docs/adr/0051-model-observation-and-invalidation-contract.md`
- `docs/workstreams/imui-ecosystem-facade-v1.md`
- `docs/workstreams/imui-state-integration-v1.md`

This document answers a practical question for ecosystem authors:

- Should `primitives` or shadcn-style components depend on `fret-selector` / `fret-query`?
- How should dependency observation and invalidation responsibilities be split?

## 1) Decision summary (v1)

1. The observation contract stays explicit:
   - component authors / callers declare which `Model` or global they observe,
   - and choose the invalidation level (`Paint`, `Layout`, `HitTest`, `HitTestOnly`).
2. The framework remains responsible for tracking and propagation:
   - record observation edges,
   - deduplicate edges,
   - propagate invalidation after model/global changes.
3. `primitives` must remain state-stack agnostic:
   - no hard dependency on `fret-query` or `fret-selector`.
4. shadcn/recipe layer may provide optional integration:
   - adapter APIs and examples can use selector/query,
   - but core component contracts should not force these crates.

In short: explicit dependencies at call-sites, framework-managed propagation, and optional ecosystem adapters.

## 2) Why this split is correct for Fret

It matches current kernel boundaries:

- `fret-ui` is mechanism-only (`ADR 0066`), not policy-heavy app state orchestration.
- invalidation contracts are explicit (`ADR 0051`), so behavior remains debuggable and predictable.
- async/concurrency contracts are runner-boundary based (`ADR 0190`), making state helpers portable.

It also keeps ecosystem extension-friendly:

- official crates can expose a golden path,
- third-party crates can opt in gradually,
- immediate-mode wrappers (`imui`) can consume service APIs without being hook-coupled.

## 3) Layered support model

### 3.1 `primitives` layer

Rules:

- Do not require `QueryClient`, query handles, or selector types in primitive public APIs.
- Accept plain values + callbacks (or existing model-based props when they are already canonical).
- Keep semantics, focus, overlay, and interaction policy contracts independent from state-stack choices.

Allowed:

- local/internal element state (`with_state_for`, transient signals),
- explicit model/global observation using existing invalidation contracts.

Not allowed (for primitives):

- hidden query fetch in component internals,
- implicit selector recompute with undocumented dependencies,
- hard-coded async runtime assumptions.

### 3.2 `fret-ui-kit` / shadcn recipes layer

Rules:

- May provide optional adapters that integrate query/selector with component recipes.
- Keep adapters thin and data-oriented; domain mutations remain command-driven.
- Keep keymap/menu actions on stable `CommandId`; dynamic actions use typed routers.

Recommended shape:

- base recipe APIs stay state-stack agnostic,
- optional `state-*` features add selector/query sugar and examples,
- docs clearly distinguish "core recipe API" vs "state adapter API".

### 3.3 App layer (golden path)

Rules:

- own `Model<T>` and command handlers,
- use selector for derived read models,
- use query for async resource lifecycle,
- pass plain derived data into components where practical.

## 4) Invalidation responsibility model (explicit and auditable)

Author/caller responsibility:

- declare dependencies and invalidation strength intentionally,
- prefer the weakest invalidation that preserves correctness.

Framework responsibility:

- record observation edges,
- dedupe repeated edges,
- union/propagate invalidation masks to affected roots.

Guideline table:

- `Paint`: visual changes without geometry change (color, text content in fixed bounds).
- `Layout`: geometry or intrinsic size can change.
- `HitTest`: full hit-test behavior and geometry might change together.
- `HitTestOnly`: hit-test map/transform changes without layout recomputation.

## 5) Selector/query integration patterns for ecosystem crates

### Pattern A: service-first (default)

- hold `QueryClient` in app/global scope,
- resolve query and selector outputs in app/recipe driver code,
- hand plain values (and typed callbacks) to components.

Best for:

- reusable components,
- third-party ecosystem portability,
- immediate-mode wrappers.

### Pattern B: hook-like sugar (optional)

- in declarative recipes, expose helper constructors that call `use_selector` / `use_query*`.
- keep helper modules feature-gated and optional.

Best for:

- official demos/templates,
- rapid app authoring where convenience is more important than minimal dependencies.

### Pattern C: stream + reducer (not query polling)

- websocket/SSE/log streams remain inbox/reducer pipelines,
- optionally invalidate query snapshots when consistency points are reached.

Best for:

- high-frequency realtime UI.

## 6) Immediate-mode (`imui`) compatibility

`imui` should not be blocked by selector/query adoption.

Recommended approach:

- use service-first query APIs outside hook-only contexts,
- compute selector/query snapshots in host app state,
- pass plain data into immediate draw calls,
- keep typed command/event boundaries in wrapper adapters.

This keeps `imui` compatible with both "full state stack" apps and lightweight apps.

## 7) Fearless-refactor scope (pre-open-source friendly)

Allowed breaking cleanup in v1 window:

1. remove hard state-stack coupling from recipe APIs where it leaked,
2. move convenience-only APIs behind feature gates,
3. normalize docs/templates to one default state story,
4. add checks that prevent new stringly command parsing in official examples.

Not required in v1:

- rewriting every historical demo,
- forcing all third-party crates to adopt selector/query immediately.

## 8) Decision log and remaining open questions

Locked in this workstream:

- optional state feature naming for ecosystem-facing crates:
  - `state-selector`: enables selector adapters/sugar,
  - `state-query`: enables query adapters/sugar,
  - `state`: umbrella convenience feature (`state-selector` + `state-query`).
- primitives remain state-stack agnostic; recipe/app layers consume optional state adapters.

Remaining open questions:

- whether to publish shared adapter traits for third-party crates in `fret-ui-kit`,
- minimum required diagnostics/gates before v1 freeze (lint + nextest + diag scripts),
- how far to push "query-aware recipes" vs keeping recipe layer fully data-driven.

## 9) Execution tracker

Implementation and milestones are tracked in:

- `docs/workstreams/component-ecosystem-state-integration-v1-todo.md`
## 10) Initial audit and guardrail (2026-02-06)

Initial audit snapshot:

- No direct `fret-query` / `fret-selector` coupling was found in:
  - `ecosystem/fret-ui-kit/src`
  - `ecosystem/fret-ui-shadcn/src`
  - `ecosystem/fret-ui-material3/src`
  - `ecosystem/fret-imui/src`
- Existing selector/query integration remains concentrated in adapter-oriented surfaces
  (currently `ecosystem/fret-authoring`).

Guardrail added:

- `tools/check_component_state_coupling.ps1`
  - blocks direct selector/query imports and `use_query`/`use_selector` sugar in primitive-oriented
    source roots,
  - blocks direct `fret-query` / `fret-selector` dependencies in primitive-oriented Cargo manifests,
  - supports allowlisted adapter directories (`*/src/state/*`).

Recommended usage:

- run locally before PR: `powershell -ExecutionPolicy Bypass -File .\tools\check_component_state_coupling.ps1`
- optionally wire into consistency checks once the current workspace churn settles.
## 11) Ecosystem-by-ecosystem state recommendation matrix

Legend:

- `N`: no direct selector/query requirement (state-stack agnostic by default).
- `S`: selector-first (derived state is the main value).
- `Q`: query-first (async resource lifecycle is the main value).
- `S+Q`: both are usually valuable.
- `S+Q+Stream`: both, plus explicit stream/reducer path for realtime updates.

Notes:

- This is a **recommended default** map for v1 planning, not a hard compile-time contract.
- For UI component crates, treat selector/query support as optional adapter surfaces unless explicitly documented otherwise.

| Ecosystem crate | Recommended mode | Notes |
| --- | --- | --- |
| `delinea` | `S` (optional `Q`) | Headless chart math is primarily derived-state heavy; query can feed remote datasets. |
| `fret-asset-cache` | `Q` | Async resource loading/caching lifecycle is core. |
| `fret-authoring` | `S` / `Q` (feature-gated) | Adapter facade surface; keep optional features. |
| `fret-bootstrap` | `S+Q` | App bootstrap commonly needs derived UI flags and async startup resources. |
| `fret-canvas` | `N` | Rendering/authoring surface; keep data acquisition outside. |
| `fret-chart` | `S+Q` | Derived chart projections + async dataset refresh are common together. |
| `fret-code-editor` | `S` (optional `Q`) | Cursor/selection/folding are derived-heavy; remote indexes/LSP can be query-fed. |
| `fret-code-editor-buffer` | `N` | Buffer core should stay runtime/data-structure focused. |
| `fret-code-editor-view` | `S` | View projections (wrap, highlights, gutters) are derived-state centric. |
| `fret-code-view` | `S` | Read-only code projection/filtering favors selector-first. |
| `fret-dnd` | `N` | Interaction mechanism crate; avoid state-stack coupling. |
| `fret-docking` | `S` | Layout/view derivations dominate; keep async fetching outside docking core. |
| `fret-executor` | `N` | Execution substrate, not UI state policy. |
| `fret-gizmo` | `S` | Manipulator/read-model derivations are primary. |
| `fret-icons` | `N` | Data registry crate; no selector/query requirement. |
| `fret-icons-lucide` | `N` | Data-only icon pack. |
| `fret-icons-radix` | `N` | Data-only icon pack. |
| `fret-imui` | `N` (host-side `S+Q`) | Core immediate facade remains agnostic; host app computes selector/query snapshots. |
| `fret-kit` | `S+Q` | Golden-path app layer benefits from the full state stack. |
| `fret-markdown` | `S+Q+Stream` | Derived document projections + async asset/query + stream-like updates. |
| `fret-node` | `S` (optional `Q`) | Graph derivations are primary; remote metadata can be query-fed. |
| `fret-plot` | `S+Q` | Plot transforms/aggregation + async data loading are common. |
| `fret-plot3d` | `S+Q` | Same as `fret-plot`, with heavier derived projection needs. |
| `fret-query` | `Q` | Query core implementation crate. |
| `fret-router` | `S` (optional `Q`) | Route-state derivation first; optional route-level prefetch/query integration. |
| `fret-selector` | `S` | Selector core implementation crate. |
| `fret-syntax` | `N` (optional `S`) | Parsing/highlighting core should stay independent; selectors may wrap outputs at view layer. |
| `fret-ui-ai` | `S+Q+Stream` | AI UX often combines async calls, derived UI state, and stream updates. |
| `fret-ui-assets` | `Q` | Asset/resource lifecycle management is query-like by nature. |
| `fret-ui-headless` | `S` | Headless state machines and derived projections are core value. |
| `fret-ui-kit` | `N` core, optional `S+Q` adapters | Keep primitives/contracts agnostic; add optional `state-*` adapters. |
| `fret-ui-material3` | `N` core, optional `S+Q` adapters | Same policy as shadcn/ui-kit ecosystems. |
| `fret-ui-shadcn` | `N` core, optional `S+Q` adapters | Recipe layer can offer optional convenience adapters. |
| `fret-undo` | `S` | Derived undo/redo availability and projection logic are selector-first. |
| `fret-viewport-tooling` | `S` (optional `Q`) | Viewport/editor state derivation first; optional async asset/resource queries. |
| `fret-workspace` | `S+Q` | Workspace shells frequently mix derived UI state and async project resources. |

## 12) Component-family decision matrix (when to use selector/query)

| Component family | Recommended mode | Why |
| --- | --- | --- |
| Buttons, toggles, chips, basic inputs | `N` / `S` | Mostly local/model state + enable/disable derivation; avoid embedded fetch. |
| Menus, popovers, tooltips, dialogs | `N` / `S` | Interaction policy + focus/overlay state; query only when content is truly remote. |
| Data tables / trees / outliners | `S+Q` | Remote snapshots + heavy derived projections (filters/sorts/groups/selection). |
| Search/command palette | `S+Q` or `S+Q+Stream` | Query for remote results, selector for ranking/grouping, stream for realtime suggestions. |
| Forms (local validation) | `S` | Derived validity/dirty/submit-state; no forced query dependency. |
| Forms (remote validation/options) | `S+Q` | Async options/validation with derived field-level projections. |
| Markdown/code preview with remote assets | `S+Q` | Query for assets; selector for projection/counters/outline. |
| Charts/plots/node views | `S` (optional `Q`) | Derived transforms dominate; query adds value when data is remote. |
| Realtime logs/telemetry panels | `S+Q+Stream` | Stream ingestion + snapshot queries + derived summarization. |

Practical rule:

- If data is local and deterministic: start with `S`.
- If data is async snapshot-like: add `Q`.
- If updates are continuous/realtime: keep stream/reducer as the primary path and treat query as snapshot cache.
## 13) Initial code adoption (2026-02-06)

Feature naming adoption (v1 convention):

- `ecosystem/fret-ui-shadcn/Cargo.toml`
  - `state-selector`, `state-query`, `state`
- `ecosystem/fret-ui-material3/Cargo.toml`
  - `state-selector`, `state-query`, `state`
- `ecosystem/fret-ui-kit/Cargo.toml`
  - `state-selector`, `state-query`, `state` (reserved for adapter modules)
- `ecosystem/fret-imui/Cargo.toml`
  - `state-selector`, `state-query`, `state`
  - compatibility aliases: `selector`, `query`

Recipe-layer adapter sample (optional, feature-gated):

- `ecosystem/fret-ui-shadcn/src/state.rs`
  - selector helper: `use_selector_badge(...)`
  - query helpers: `query_status_badge(...)`, `query_error_alert(...)`
- `ecosystem/fret-ui-shadcn/src/lib.rs`
  - gated exports through `prelude` under `state-selector` / `state-query`

This keeps primitive contracts state-stack agnostic while giving official recipes a low-friction,
opt-in state integration path.

## 14) imui compatibility landing (service-first)

The immediate-mode compatibility guidance is now captured in:

- `docs/workstreams/imui-state-integration-v1.md`

Scope of that landing note:

- keep `imui` core APIs state-stack agnostic,
- orchestrate selector/query on the host side,
- pass plain snapshots into immediate draws,
- include practical integration scenarios (`reqwest`, `sqlx`, stream pipelines).
