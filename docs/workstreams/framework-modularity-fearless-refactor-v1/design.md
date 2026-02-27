# Framework Modularity (Fearless Refactor v1)

This workstream is about making Fret easier to *consume modularly* (Bevy-like “depend on only what you need”)
while making the repository easier to *maintain without boundary drift*.

These notes are **not** an ADR. If we change hard-to-change contracts, we must write/adjust ADRs.

## Problem Statement (What we are actually trying to fix)

### From a framework user’s perspective

Today, a new user can reasonably ask:

1) “Which crate do I depend on?”
2) “How do I get a minimal build that compiles fast?”
3) “Can I embed Fret into my engine (engine-owned GPU context) without pulling in platform glue?”
4) “Can I use only `fret-core` (docking ops/scene contracts) without UI/runtime?”

We have many crates (kernel/backends/ecosystem/apps) and *the layering is correct*, but the **consumption
profiles** are not made explicit and enforced as a stable product surface.

### From a maintainer’s perspective

The main risks are not “too many crates” but:

- Entry points becoming unclear (multiple “golden paths” with different feature defaults).
- Glue crates becoming “everything crates” (especially cross-platform launchers).
- Feature sets drifting such that “minimal kernel” is no longer minimal.
- Ecosystem experiments pulling in backend dependencies via exceptions and convenience wiring.

This workstream defines **consumption profiles**, clarifies **what is public/stable**, and introduces **gates**
that keep modular consumption viable during rapid iteration.

## Guiding Constraints (Non-negotiable)

- Keep kernel portable: no backend leakage into `fret-core` / `fret-runtime` / `fret-app` / `fret-ui`.
- Keep ecosystem portable: ecosystem crates must not depend on backend crates (exceptions must remain rare and explicit).
- Keep backends swappable: backend crates must not depend on `fret-ui` or ecosystem crates.
- Prefer “outcome ports”, not implementation ports (ADR-driven contracts).

References:

- `docs/adr/0092-crate-structure-core-backends-apps.md`
- `docs/adr/0037-workspace-boundaries-and-components-repository.md`
- `docs/dependency-policy.md`
- `tools/check_layering.py`

## Personas (Who we are designing for)

1) **App author (desktop-first)**
   - Wants a batteries-included path with good defaults.
   - Wants predictable feature flags (turn off diagnostics/assets/icons easily).

2) **Engine embedder**
   - Wants to own the GPU context and possibly the window/event loop.
   - Wants to use only the contracts needed: `fret-core` scene ops + renderer backend, or UI substrate only.

3) **Component library author**
   - Needs stable `fret-ui` mechanisms and `fret-runtime` host boundary.
   - Must stay portable and avoid platform/render dependencies.

4) **Framework maintainer**
   - Needs strict layering enforcement and “profile build” gates.
   - Needs a small set of clearly-owned public entry crates and stable prelude patterns.

## Target Consumption Profiles (Bevy-like modularity)

We define profiles as **documented dependency recipes** + **feature gates** + **CI checks**.

### Profile A — Contracts-only (portable)

Goal: depend on “just the contracts”.

- Core: `fret-core`
- Optional adjunct contracts (still portable): `fret-runtime`, `fret-platform` (contracts only), `fret-render-core`

Success criteria:

- No platform/render dependencies pulled in transitively.
- Build times are small and deterministic.

### Profile B — UI substrate (portable kernel)

Goal: build UI behavior and emit a backend-agnostic scene without choosing a platform backend.

- `fret-ui` + `fret-runtime` (+ optional `fret-app` if using default app runtime)

Success criteria:

- A custom host can be implemented without pulling in winit/wgpu/web-sys.
- Public surface is stable enough for ecosystem component crates.

### Profile C — Backend assembly (advanced/manual)

Goal: choose platform + renderer explicitly (winit vs web, wgpu now, future backends later).

- `fret-framework` (feature-selected bundles), or direct deps on `fret-launch` + backend crates

Success criteria:

- There is one “manual assembly” surface with clearly documented features.
- Each backend path has a minimal compile profile and a small reference app.

### Profile D — Batteries-included (golden path)

Goal: “it runs out of the box” with recommended ecosystem policies.

- `ecosystem/fret` (batteries-included meta crate)

Success criteria:

- Defaults are coherent and well documented (what features do, how to disable).
- “Golden path” stays in ecosystem; kernel remains clean.

## Refactor Targets (What to change, at a high level)

### 1) Make entry points explicit and stable

Define a short, stable table in docs:

- “If you want X, depend on Y, enable features Z.”
- Keep “manual assembly” separate from “batteries-included”.

Concrete TODO direction:

- Document `fret-framework` features as the canonical “assembly surface” (it is already feature-gated).
- Document `ecosystem/fret` as the canonical “batteries-included” path.
- Ensure both are aligned and do not create contradictory defaults.

### 2) Split glue where it hurts modularity (`fret-launch`)

`fret-launch` tends to grow as it owns real platform integration. The risk is it becomes a “everything crate”.

Concrete TODO direction:

- Keep `fret-launch` as a small facade and move heavy implementations behind platform-specific crates/modules:
  - `fret-launch-desktop` (winit, native platform I/O, desktop specifics)
  - `fret-launch-web` (wasm/web-sys specifics)
- Ensure the assembly surface can opt out of unneeded platforms via features.

### 3) Add “profile gates” (prevent regressions)

Layering is already enforced. We also need “profile build” gates so modular consumption does not rot.

Concrete TODO direction:

- Add CI checks (or local scripts) that build:
  - Profile A: `cargo check -p fret-core`
  - Profile B: `cargo check -p fret-ui`
  - Profile C: `cargo check -p fret-framework --no-default-features --features core,ui,runtime,render`
  - Profile D: `cargo check -p fret` (ecosystem batteries) with default features
- Add at least one “minimal example” app per profile (or ensure existing demos cover them).

### 4) Public surface classification (maintenance scalability)

Not all crates should be treated as equally stable.

Concrete TODO direction:

- Maintain a small list of “publicly supported entry crates”:
  - `fret-core`, `fret-runtime`, `fret-ui`, `fret-framework`, `ecosystem/fret`
- Everything else is either backend implementation, ecosystem policy, or internal glue.
- When a crate is promoted to “public”, require an ADR note and gates/tests.

## Non-goals (Explicitly out of scope for v1)

- Rewriting renderer architecture (handled by renderer workstreams).
- Collapsing crates into fewer crates “just because”.
- Forcing a single app runtime or a single async runtime.
- Moving ecosystem out-of-tree (we can prepare for it, but not required here).

## Open Questions (Need answers before large code refactors)

1) What is the long-term “stable crate name” for the recommended top-level dependency?
   - Today: `ecosystem/fret` is the batteries meta crate; `fret-framework` is the advanced assembly facade.
2) Do we want a new *kernel-only* facade crate (no backends), or is `fret-framework`’s feature model sufficient?
3) Should we rename “runner” crates to “backend” crates for clarity (ADR 0092 future work)?
4) What is the acceptable exception policy for ecosystem -> backend deps (keep allowlist tiny)?

