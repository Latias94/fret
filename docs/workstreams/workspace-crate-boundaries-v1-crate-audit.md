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
