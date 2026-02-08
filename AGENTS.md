# Repository Guidelines

## Project Structure & Module Organization

- Canonical overview: `docs/README.md` and `docs/repo-structure.md`.
- `crates/`: Core framework crates (stable boundaries; framework + backends + runner glue).
  - Contracts / portable core: `crates/fret-core`, `crates/fret-runtime`.
  - Default integrated runtime: `crates/fret-app`, `crates/fret-ui`, `crates/fret-ui-app`.
  - Backends: `crates/fret-platform`, `crates/fret-platform-native`, `crates/fret-platform-web`, `crates/fret-runner-winit`, `crates/fret-runner-web`, `crates/fret-render`.
  - Integration / wiring: `crates/fret-launch`.
  - Public facade: `crates/fret`.
  - Other core glue: `crates/fret-a11y-accesskit`, `crates/fret-fonts`, `crates/fret-i18n`.
- `ecosystem/`: In-tree incubation crates (components, icon sets, app kits; may move out-of-tree later).
  - Component/policy layers: `ecosystem/fret-ui-kit`, `ecosystem/fret-docking`, `ecosystem/fret-ui-shadcn`.
  - App kit / defaults: `ecosystem/fret-kit`, `ecosystem/fret-bootstrap`, `ecosystem/fret-ui-assets`.
  - Icons: `ecosystem/fret-icons`, `ecosystem/fret-icons-lucide`, `ecosystem/fret-icons-radix`.
- `apps/`: Runnable apps / end-to-end harness shells (in the workspace, excluded from `default-members`).
  - Harness code: `apps/fret-examples`.
  - Native/web shells: `apps/fret-demo`, `apps/fret-demo-web`.
  - Tooling + diagnostics runner: `apps/fretboard`.
  - Other apps exist (e.g. UI gallery/editor); treat `apps/` as non-stable surfaces.
- `docs/`: Documentation-driven design; ADRs live in `docs/adr/`.
- `tools/`: Scripts and maintenance utilities (e.g. layering checks).
- `repo-ref/`: Pinned upstream references (not required to build; do not treat as dependencies).
- `assets/`, `themes/`, `screenshots/`: Non-code assets.
- `.fret/`: Project-scoped local state. Some files (e.g. `.fret/settings.json`, `.fret/keymap.json`) may be checked into VCS per ADRs.

## Build, Test, and Development Commands

- `cargo build`: build the full workspace.
- `cargo run -p fret-demo --bin todo_demo`: run a specific native demo (writes to `.fret/`).
- `cargo run -p fretboard -- dev native --bin todo_demo`: preferred demo runner (consistent flags).
- `cargo run -p fretboard -- dev web --demo ui_gallery`: run the web UI gallery demo.
- `cargo run -p fretboard -- new todo --name my-todo [--ui-assets]`: generate a starter todo app template.
- `cargo test --workspace`: run all tests (may be sparse early on).
- Prefer `cargo nextest run` when available for faster test execution.
- `cargo fmt`: format code with rustfmt.
- `cargo clippy --workspace --all-targets -- -D warnings`: lint (treat warnings as errors).
- `python3 tools/check_layering.py`: enforce workspace crate boundary rules (see `docs/dependency-policy.md`).

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

## Documentation Language

- Write repository documentation and code comments in English.
