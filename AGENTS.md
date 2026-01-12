# Repository Guidelines

## Project Structure & Module Organization

- `crates/`: Core framework crates (Cargo workspace members).
  - `crates/fret-core`: platform-agnostic core contracts and IDs (keep minimal).
  - `crates/fret-runtime`: portable runtime services/value types.
  - `crates/fret-app`: app runtime (effects, commands, models).
  - `crates/fret-ui`: UI runtime and widgets (layout via `taffy`).
  - `crates/fret-render`: wgpu-based renderer building blocks.
  - `crates/fret-platform`: portable platform I/O contracts.
  - `crates/fret-platform-native`: native platform I/O implementation.
  - `crates/fret-platform-web`: wasm platform I/O implementation.
  - `crates/fret-runner-winit`: winit adapter (event/input mapping).
  - `crates/fret-runner-web`: compatibility shim re-exporting `fret-platform-web` (dedicated DOM adapter TBD).
  - `crates/fret-fonts`: bundled default font bytes for wasm/bootstrap.
  - `crates/fret-launch`: runner/launcher glue (desktop now; web/mobile later).
  - `crates/fret`: facade crate (re-exports).
- `ecosystem/`: Policy-heavy UI kits and reusable component surfaces (Cargo workspace members).
  - `ecosystem/fret-ui-kit`: shared interaction policies + headless primitives + styling helpers.
  - `ecosystem/fret-docking`: docking UI + interaction policy.
  - `ecosystem/fret-ui-shadcn`: shadcn/ui v4-aligned taxonomy + recipes built on `fret-ui-kit`.
  - `ecosystem/fret-bootstrap`: golden-path startup layer over `fret-launch` (optional).
  - `ecosystem/fret-ui-assets`: UI render asset caches facade (re-export surface over `fret-asset-cache`).
  - `ecosystem/fret-icons`: renderer-agnostic icon registry.
  - `ecosystem/fret-icons-lucide`: Lucide icon pack (data-only).
  - `ecosystem/fret-icons-radix`: Radix icon pack (data-only).
- `docs/`: documentation-driven design (start at `docs/README.md`); ADRs in `docs/adr/`.
- `apps/`: runnable apps / experiments (in the workspace, but excluded from `default-members`).
  - `apps/fret-demo`: native demo binaries (thin shells over `apps/fret-examples`).
  - `apps/fret-demo-web`: wasm demo shell (thin shell over `apps/fret-examples`).
  - `apps/fretboard`: dev CLI (run demos, hotpatch helpers, starter templates).
- `repo-ref/`: pinned upstream reference checkouts (not required to build).
- `.fret/`: generated local state when running the demo (e.g. layout/keymap JSON).

## Build, Test, and Development Commands

- `cargo build`: build the full workspace.
- `cargo run -p fret-demo --bin todo_demo`: run a specific native demo (writes to `.fret/`).
- `cargo run -p fretboard -- dev native --bin todo_demo`: preferred demo runner (consistent flags).
- `cargo run -p fretboard -- init todo --name my-todo [--ui-assets]`: generate a starter todo app template.
- `cargo test --workspace`: run all tests (may be sparse early on).
- Prefer `cargo nextest run` when available for faster test execution.
- `cargo fmt`: format code with rustfmt.
- `cargo clippy --workspace --all-targets -- -D warnings`: lint (treat warnings as errors).

Toolchain is pinned via `rust-toolchain.toml` (Rust 1.92) and the workspace uses Rust 2024 edition.

## Coding Style & Naming Conventions

- Follow standard Rust style (4-space indentation; rustfmt as the source of truth).
- Names: crates `fret-*`, modules/functions `snake_case`, types `UpperCamelCase`.
- Prefer workspace-managed deps in root `Cargo.toml` (`[workspace.dependencies]`).
- Respect layering: keep `fret-core` free of `wgpu`, `winit`, and layout engines (see `docs/dependency-policy.md`).

## Testing Guidelines

- Unit tests: place near code with `#[cfg(test)] mod tests { ... }`.
- Integration tests: `crates/<crate>/tests/*.rs`.
- Name tests with intent (e.g. `selection_extends_to_word_boundary`).

## Commit & Pull Request Guidelines

- Use Conventional Commits seen in history: `feat(scope): ...`, `fix(scope): ...`, `docs(adr): ...`.
- PRs should include: motivation, linked issue/ADR (if changing contracts), and UI screenshots/GIFs when behavior changes.
- Before opening: run `cargo fmt`, `cargo clippy ...`, and `cargo test --workspace`.

## Documentation & ADR Workflow

- Treat ADRs as the source of truth for cross-crate contracts.
- If you change a “hard-to-change” behavior (input, focus, docking, text, rendering boundaries), update or add an ADR in `docs/adr/`.
- Keep the ADR implementation audit up to date:
  - When you implement/refactor behavior covered by an ADR, update the corresponding row in `docs/adr/IMPLEMENTATION_ALIGNMENT.md`.
  - Mark one of: `Aligned`, `Aligned (with known gaps)`, `Partially aligned`, `Not implemented`, `Not audited`.
  - Add 1–3 evidence anchors (file paths / key functions / tests) or clearly describe the missing pieces.
