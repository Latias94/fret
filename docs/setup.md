# Setup (Native) — Toolchain, OS deps, and fast builds

This page is for **app developers** who want the fastest “edit → run” loop for Fret native demos and templates.

If you want the shortest onboarding path, also read `docs/first-hour.md`.

Reference (inspiration for structure and topics):
https://bevy.org/learn/quick-start/getting-started/setup/

## Rust toolchain

This repository pins Rust via `rust-toolchain.toml` (currently `1.92`).

- Install Rust with rustup: https://www.rust-lang.org/learn/get-started
- Ensure the pinned toolchain is available:

```bash
rustup toolchain install 1.92
```

## OS dependencies

Fret’s native stack is desktop-first and uses `winit` + `wgpu`. OS requirements are mostly the same as other
Rust desktop GPU apps.

### Windows (MSVC)

- Install **Visual Studio C++ Build Tools**
- Recommended workload: **Desktop development with C++**
- Minimum components:
  - MSVC (latest for your architecture)
  - Windows SDK (latest for your OS)
  - C++ CMake tools for Windows (helpful for native deps)

### macOS

- Install Xcode Command Line Tools: `xcode-select --install`

### Linux

Linux requirements vary by distro and Wayland/X11 setup.

If you hit build or runtime errors on Linux, start by verifying you can build and run a minimal `winit` + `wgpu`
app, then align your system packages accordingly.

## Editor / IDE

Use any editor, but install `rust-analyzer` (VS Code, IntelliJ Rust, Zed, etc.).

## Fast iteration (recommended)

### 1) Prefer `fretboard-dev` for running demos in this repo

For a quick “is my environment working?” run:

```bash
cargo run -p fretboard-dev -- dev native --bin todo_demo
```

Notes:

- On Windows, `fretboard-dev dev native ...` defaults to `--profile dev-fast` for faster builds.
- On other platforms (or if you want richer debug inspection), you can pass `--profile dev`.
- `fretboard-dev dev ...` now defaults to strict runtime diagnostics so framework bugs fail fast during local iteration.
- Use `--no-strict-runtime` only when you intentionally want to inspect fallback/recovery behavior.
- The published `fretboard` CLI remains the public `assets` / `config` / `new` surface; repo-runner commands stay on `fretboard-dev`.

### 2) Use the built-in aliases for the common path

This repo ships a few aliases in `.cargo/config.toml`:

```bash
cargo demo-todo
cargo build-todo
cargo timings-todo
```

`cargo timings-todo` is useful for identifying which crates dominate compile time.

### 3) Avoid building “everything” on Windows

Building many statically-linked demo binaries in parallel can exhaust virtual memory on Windows.

If you need to build all `fret-demo` bins, prefer the throttled helper:

```bash
python3 tools/windows/build-fret-demo-bins.py
```

## Optional: compilation caching (sccache)

If you frequently switch branches / worktrees or maintain multiple local builds, a compiler cache can help.

One common setup:

- Install `sccache` (https://github.com/mozilla/sccache)
- Set `RUSTC_WRAPPER=sccache` in your shell environment

## Optional: faster linking on Windows (rust-lld)

Link time can dominate large Rust binaries on Windows/MSVC.

If you want to experiment with faster linking, you can configure Cargo to use `rust-lld.exe` in a **local**
Cargo config (prefer user-level config so you can revert easily):

```toml
# %USERPROFILE%\.cargo\config.toml
[target.x86_64-pc-windows-msvc]
linker = "rust-lld.exe"
```

If you hit linker/toolchain issues, remove the override and fall back to the default MSVC linker.

## Common Windows pitfalls

- Stack overflow on launch: see `docs/known-issues.md` (“Windows: thread 'main' has overflowed its stack”).
- OOM / metadata mmap failures when building many bins: see `docs/known-issues.md` (“Windows: cargo build -p fret-demo --bins fails...”).
