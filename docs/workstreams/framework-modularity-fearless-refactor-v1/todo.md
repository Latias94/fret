# Framework Modularity (Fearless Refactor v1) — TODO

This is a TODO list for the workstream described in `design.md`.

## M0 — Baseline and alignment

- [ ] Write a short “Which crate do I depend on?” table (profiles A–D) and link it from:
  - [ ] `README.md` (brief)
  - [ ] `docs/README.md` (canonical pointer)
- [ ] Add a “public entry crates” list and define stability expectations:
  - [ ] `fret-core`
  - [ ] `fret-runtime`
  - [ ] `fret-ui`
  - [ ] `fret-framework`
  - [ ] `ecosystem/fret`
- [ ] Confirm the intended role split is consistent with:
  - [ ] ADR 0037 (`fret` vs `fret-components`)
  - [ ] ADR 0092 (core/backends/apps split)

## M1 — Consumption profiles as buildable targets

- [x] Add a portable consumption profiles gate (`python3 tools/check_consumption_profiles.py`) and run it in CI (`.github/workflows/consistency-checks.yml`).
- [ ] Add a minimal “contracts-only” build gate:
  - [ ] `cargo check -p fret-core`
- [ ] Add a minimal “UI substrate” build gate:
  - [ ] `cargo check -p fret-ui`
- [ ] Add a minimal “manual assembly” build gate:
  - [ ] `cargo check -p fret-framework --no-default-features --features core,runtime,ui`
- [ ] Add a minimal “batteries” build gate:
  - [ ] `cargo check -p fret` (ecosystem meta crate, default features)
- [ ] Ensure `python3 tools/check_layering.py` remains a required gate.

## M2 — Reduce glue bloat (launcher refactor)

- [x] Replace `winit::dpi` types in the public launcher config/spec with Fret-owned window geometry types (`WindowLogicalSize`, `WindowPosition`) to keep the public surface backend-agnostic.
- [ ] Audit `crates/fret-launch` for platform-specific “weight”:
  - [ ] winit event loop / window state / IME routing
  - [ ] web RAF + browser event mapping
  - [ ] effect draining glue (clipboard/dialog/open-url/etc)
- [ ] Decide and document the split strategy:
  - [ ] keep `fret-launch` as a small facade
  - [ ] move heavy implementation to `fret-launch-desktop` / `fret-launch-web` (or equivalent)
- [ ] Add a compile-time feature matrix so a user can opt into only one platform backend.

## M3 — Public surface hygiene (maintenance)

- [ ] Introduce “promotion rules” for new public crates/APIs:
  - [ ] must have an owning layer (kernel vs backend vs ecosystem)
  - [ ] must have a layering check and at least one targeted test/demo anchor
  - [ ] must have a doc entry in the profile table
- [ ] Establish a deprecation process for old entry points / feature aliases:
  - [ ] warn in docs first
  - [ ] keep compatibility for at least one minor cycle (or per ADR)

## Evidence / Exit criteria (for calling v1 “done enough”)

- [ ] A developer can choose a profile and copy/paste a dependency snippet that compiles.
- [ ] CI (or local gates) prevents drift for all profiles.
- [ ] Launcher glue is not a growth hotspot (bounded ownership + platform splits).
- [ ] Boundary rules stay green (`tools/check_layering.py`).
