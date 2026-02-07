# Workspace Crate Boundaries Audit v1 — Crate Survey Notes

Status: Draft (workstream notes only; ADRs remain the source of truth)

This document is a lightweight companion to:

- `docs/workstreams/workspace-crate-boundaries-v1.md`
- `docs/workstreams/workspace-crate-boundaries-v1-todo.md`

It captures “what we learned” while auditing the current workspace crate graph, with a bias toward:

- clear, long-lived boundaries in `crates/`,
- extractable, policy-heavy iteration in `ecosystem/`,
- and a small number of facade crates that express the intended golden paths.

## 1) Boundary heuristics (merge vs split)

Use these rules of thumb when deciding whether to merge/split crates:

1. **Contracts vs implementations**
   - If a type is backend-agnostic and used across multiple backends, it belongs in a small contract
     crate (often `*-core`) with minimal dependencies.
   - If code is backend-specific (`wgpu`, `winit`, DOM), it belongs in a clearly named backend crate
     (`*-wgpu`, `*-winit`, `*-web`) and must not leak into kernel/UI contract layers.
2. **Portable by default**
   - Default features should stay portable (no “UI sugar” or backend implementation by default).
   - App-level ergonomics belong behind explicit features, or in higher-level facade crates.
3. **Ecosystem extractability**
   - Ecosystem crates can be small and numerous, but each should have an “extract story”:
     a coherent responsibility + an edge boundary that can become an external dependency later.
4. **Compile-time + cognition cost**
   - If a crate is always used together with exactly one parent and offers no meaningful boundary
     (no portability win, no extraction path, no compile-time isolation), it is a merge candidate.
5. **Naming must advertise the seam**
   - If a crate exists to make a seam explicit, its name should say so (`*-core`, `*-wgpu`, `*-web`).
   - Avoid generic names that hide whether something is a contract layer or an implementation.

## 2) Current shape (high-level)

### `crates/` (stable boundaries)

- Kernel contracts: `fret-core`, `fret-runtime`, `fret-app`, `fret-ui`
- Backends/adapters: `fret-platform-*`, `fret-runner-*`
- Renderer: `fret-render-core` (portable contracts), `fret-render-wgpu` (wgpu impl), `fret-render` (facade)
- Facade: `fret` (feature bundles expressing desktop vs web intent)
- Wiring: `fret-launch` (golden-path runner/app glue)

### `ecosystem/` (extractable policy + components)

- Policy engines: `fret-ui-headless` (TanStack-style table, typeahead, etc.)
- Component infrastructure: `fret-ui-kit` (tokens, layout recipes, helpers)
- Component surfaces: `fret-ui-shadcn`, `fret-ui-material3`, etc.
- App kits: `fret-kit`, `fret-bootstrap`, `fret-workspace`
- Specialized tools/components: code view/editor, docking, node graph, plots, etc.

## 3) Findings / adjustments made in this workstream

### 3.1 Render: explicit backend split

Completed (see workstream M1):

- `fret-render-core`: backend-agnostic render contract types
- `fret-render-wgpu`: wgpu implementation
- `fret-render`: compatibility facade

### 3.2 Router vs query: keep separate, tighten feature intent

Decision: keep `fret-router` (navigation/URL model) and `fret-query` (async resource cache/state) as
separate ecosystem crates.

Boundary tightening:

- Prefer **portable-by-default** features:
  - UI sugar (`ElementContext` extension traits) should be opt-in (e.g. `fret-query/ui`).
- Remove stale/unused feature flags when a crate does not actually implement them yet.

### 3.3 UI primitives: remove accidental micro-crate

Completed (see workstream M6):

- `ecosystem/fret-ui-primitives` was only consumed via `fret-ui-kit` compatibility shims, so the
  extra crate did not represent a real seam yet.
- We merged its code into `ecosystem/fret-ui-kit/src/{primitives,declarative}/*` to reduce workspace
  churn and dependency noise, while keeping the public import surface stable.

## 3.3 Reference patterns (notes only)

These snapshots are not prescriptive, but they help validate that our boundaries match common
successful shapes:

- **Zed / GPUI**: a GPU UI framework with strong platform feature gating (Wayland/X11/macOS/Windows),
  and a large “one crate, many optional platform deps” approach via features.
  - Takeaway for Fret: we still want platform separation, but we prefer explicit adapter crates
    (`*-winit`, `*-web`, `*-wgpu`) so contracts stay narrow and backends are swappable.
- **Dioxus**: a UI framework with a large multi-package workspace (`core`, `html`, `router`, `desktop`,
  `web`, etc.) that keeps portability via many explicit crates.
- Takeaway for Fret: this supports our “contract vs adapter vs facade” direction; the key is to
  avoid unnecessary micro-crates unless they represent a real seam.

## 3.4 Ecosystem crate graph snapshot (2026-02-07)

Method:

- `cargo metadata --no-deps` for workspace packages.
- Reverse-deps count = “how many workspace crates depend on this crate”.
- LOC = best-effort `.rs` line count under the crate directory (excluding `target/`).

This is *not* a stability signal by itself; it is a quick way to spot “single-consumer micro-crates”
that do not provide a seam.

| crate | rev deps | rs files | loc | path |
| --- | ---: | ---: | ---: | --- |
| `fret-ui-kit` | 16 | 166 | 66843 | `ecosystem/fret-ui-kit` |
| `fret-icons` | 10 | 1 | 420 | `ecosystem/fret-icons` |
| `fret-ui-shadcn` | 8 | 151 | 194310 | `ecosystem/fret-ui-shadcn` |
| `fret-canvas` | 7 | 35 | 7941 | `ecosystem/fret-canvas` |
| `fret-query` | 7 | 1 | 2790 | `ecosystem/fret-query` |
| `fret-authoring` | 7 | 3 | 286 | `ecosystem/fret-authoring` |
| `fret-bootstrap` | 4 | 3 | 14848 | `ecosystem/fret-bootstrap` |
| `delinea` | 3 | 57 | 38071 | `ecosystem/delinea` |
| `fret-ui-headless` | 3 | 86 | 31232 | `ecosystem/fret-ui-headless` |
| `fret-workspace` | 3 | 11 | 7298 | `ecosystem/fret-workspace` |
| `fret-markdown` | 3 | 8 | 4424 | `ecosystem/fret-markdown` |
| `fret-code-view` | 3 | 8 | 2814 | `ecosystem/fret-code-view` |
| `fret-selector` | 3 | 2 | 611 | `ecosystem/fret-selector` |
| `fret-undo` | 3 | 1 | 489 | `ecosystem/fret-undo` |
| `fret-node` | 2 | 342 | 77057 | `ecosystem/fret-node` |
| `fret-chart` | 2 | 19 | 15013 | `ecosystem/fret-chart` |
| `fret-kit` | 2 | 7 | 3828 | `ecosystem/fret-kit` |
| `fret-router` | 2 | 12 | 2400 | `ecosystem/fret-router` |
| `fret-syntax` | 2 | 2 | 1052 | `ecosystem/fret-syntax` |
| `fret-dnd` | 2 | 9 | 972 | `ecosystem/fret-dnd` |
| `fret-executor` | 2 | 1 | 847 | `ecosystem/fret-executor` |
| `fret-code-editor-view` | 2 | 1 | 808 | `ecosystem/fret-code-editor-view` |
| `fret-code-editor-buffer` | 2 | 1 | 753 | `ecosystem/fret-code-editor-buffer` |
| `fret-ui-assets` | 2 | 5 | 566 | `ecosystem/fret-ui-assets` |
| `fret-viewport-tooling` | 2 | 1 | 386 | `ecosystem/fret-viewport-tooling` |
| `fret-ui-material3` | 1 | 107 | 79204 | `ecosystem/fret-ui-material3` |
| `fret-plot` | 1 | 31 | 25559 | `ecosystem/fret-plot` |
| `fret-gizmo` | 1 | 23 | 16246 | `ecosystem/fret-gizmo` |
| `fret-docking` | 1 | 24 | 15287 | `ecosystem/fret-docking` |
| `fret-imui` | 1 | 3 | 4777 | `ecosystem/fret-imui` |
| `fret-ui-ai` | 1 | 20 | 4770 | `ecosystem/fret-ui-ai` |
| `fret-code-editor` | 1 | 7 | 4592 | `ecosystem/fret-code-editor` |
| `fret-icons-lucide` | 1 | 3 | 1825 | `ecosystem/fret-icons-lucide` |
| `fret-asset-cache` | 1 | 4 | 1202 | `ecosystem/fret-asset-cache` |
| `fret-icons-radix` | 1 | 3 | 448 | `ecosystem/fret-icons-radix` |
| `fret-i18n-fluent` | 1 | 1 | 225 | `ecosystem/fret-i18n-fluent` |
| `fret-plot3d` | 1 | 2 | 171 | `ecosystem/fret-plot3d` |

Immediate interpretation:

- **Data packs** (`fret-icons-*`) should remain separate even with 1 consumer: optional deps and
  compile-time isolation are the seam.
- Several crates have 1 consumer but **large code / domain boundaries** (`fret-docking`, `fret-gizmo`,
  `fret-plot`, `fret-ui-material3`): keeping them separate is still valuable for extraction and
  cognition.
- The only “small + single-consumer” crates at this snapshot (`fret-i18n-fluent`, `fret-plot3d`)
  are not clear merge wins without a stronger product direction (they might become shared soon).

### 3.5 Single-consumer review (rev deps = 1)

This section turns the `rev deps = 1` list into explicit recommendations. The goal is to avoid
accidental micro-crates while still preserving real seams (optional packs, domain extraction,
heavy dependency isolation).

| crate | used by | rs files | loc | path | recommendation |
| --- | --- | ---: | ---: | --- | --- |
| `fret-plot3d` | `fret-examples` | 2 | 171 | `ecosystem/fret-plot3d` | **Candidate**: merge into `fret-plot` or move into `apps/fret-examples` until it has a second consumer. |
| `fret-i18n-fluent` | `fret-bootstrap` | 1 | 225 | `ecosystem/fret-i18n-fluent` | Keep separate: third-party bridge + dependency isolation; revisit if `crates/fret-i18n` wants an opt-in Fluent backend feature. |
| `fret-icons-radix` | `fret-bootstrap` | 3 | 448 | `ecosystem/fret-icons-radix` | Keep separate: data pack / optional dependency seam. |
| `fret-asset-cache` | `fret-ui-assets` | 4 | 1202 | `ecosystem/fret-asset-cache` | Keep separate: reusable subsystem boundary; likely to gain more consumers. |
| `fret-icons-lucide` | `fret-bootstrap` | 3 | 1825 | `ecosystem/fret-icons-lucide` | Keep separate: data pack / optional dependency seam. |
| `fret-code-editor` | `fret-ui-gallery` | 7 | 4592 | `ecosystem/fret-code-editor` | Keep separate: domain boundary; expected future reuse by editor shells. |
| `fret-ui-ai` | `fret-ui-gallery` | 20 | 4770 | `ecosystem/fret-ui-ai` | Keep separate: experimental domain; avoid infecting baseline UI deps. |
| `fret-imui` | `fret-examples` | 3 | 4777 | `ecosystem/fret-imui` | Keep separate: facade boundary for immediate-mode authoring; expect reuse. |
| `fret-docking` | `fret-examples` | 24 | 15287 | `ecosystem/fret-docking` | Keep separate: large domain seam (strategy-heavy, editor-grade). |
| `fret-gizmo` | `fret-examples` | 23 | 16246 | `ecosystem/fret-gizmo` | Keep separate: domain seam (tooling + math + interaction). |
| `fret-plot` | `fret-examples` | 31 | 25559 | `ecosystem/fret-plot` | Keep separate: domain seam; already large enough to justify isolation. |
| `fret-ui-material3` | `fret-ui-gallery` | 107 | 79204 | `ecosystem/fret-ui-material3` | Keep separate: design-system surface; isolation enables optional adoption. |

Follow-up suggestion:

- If `fret-plot3d` remains tiny and single-consumer after it stabilizes, prefer merging it into
  `fret-plot` (as a module) to reduce crate tax. If it grows into a real 3D subsystem, keep it
  separate and ensure it has its own demos + tests.

## 4) Open questions worth auditing next

1. **Facade policy (`crates/fret`)**
   - Re-audit default features and bundles to ensure the crate stays “portable by default” while
     still offering clear golden-path bundles (`desktop`/`wasm` aliases).
2. **UI layer clarity in ecosystem**
   - Verify that `fret-ui-headless` remains UI-agnostic and that UI-specific affordances live in
     `fret-ui-kit` / component crates.
3. **Micro-crate justification**
   - Some ecosystem crates currently have a single in-tree consumer; confirm whether they exist for
     extraction/portability reasons (keep) vs accidental fragmentation (merge).

## 4.1 Candidate list (merge / split / rename)

These are not commitments; they are “review prompts” to keep boundaries intentional.

Potential merge candidates (only if they do not represent a real extraction seam):

- `ecosystem/fret-ui-primitives` → `ecosystem/fret-ui-kit` (done)
  - It was only consumed via `fret-ui-kit` compatibility shims, so the extra crate did not provide
    a real seam yet.
  - We kept the public import surface stable (`fret-ui-kit::primitives::*`) while removing a
    transitive path dependency.

Potential split candidates (only if ownership/portability becomes unclear):

- `ecosystem/fret-ui-headless`
  - If it grows into multiple independent policy engines (table, typeahead, selection models, etc),
    consider splitting by subsystem (e.g. `fret-table-headless`, `fret-typeahead-headless`) to keep
    build times and dependency graphs shallow for downstream users.

Potential “keep separate” candidates (single consumer is OK if the seam is real):

- `ecosystem/fret-icons-*` (data packs): keep as separate crates to make optional icon packs cheap.
- `ecosystem/fret-code-editor-buffer` / `ecosystem/fret-code-editor-view`: keep if buffer/view remain
  portable and reusable across editor shells and demos.

## 5) Proposed next deliverables (documentation-first)

- Add a short “crate boundary map” section to `docs/workstreams/workspace-crate-boundaries-v1.md`
  summarizing which crates are:
  - contracts (`*-core`),
  - implementations (`*-wgpu` / `*-winit` / `*-web`),
  - facades (`fret`, `fret-render`),
  - ecosystem policy/components (`ecosystem/*`).
- Add concrete merge/split candidates with rationale (one paragraph each) and a “why now / why
  later” decision.
