# Repository Guidelines

## Project Structure & Module Organization

- `crates/`: Cargo workspace members (primary source code).
  - `crates/fret-core`: platform-agnostic core contracts and IDs (keep minimal).
  - `crates/fret-app`: app runtime (effects, commands, models).
  - `crates/fret-ui`: UI runtime and widgets (layout via `taffy`).
  - `crates/fret-render`: wgpu-based renderer building blocks.
  - `crates/fret-runner-winit-wgpu`: desktop runner wiring winit + wgpu.
  - `crates/fret-demo`: runnable demo entry point (`src/main.rs`).
  - `crates/fret`: facade crate (re-exports).
- `docs/`: documentation-driven design (start at `docs/README.md`); ADRs in `docs/adr/`.
- `repo-ref/`: pinned upstream reference checkouts (not required to build).
- `.fret/`: generated local state when running the demo (e.g. layout/keymap JSON).

## Build, Test, and Development Commands

- `cargo build`: build the full workspace.
- `cargo run -p fret-demo`: run the demo app (writes to `.fret/`).
- `cargo test --workspace`: run all tests (may be sparse early on).
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
