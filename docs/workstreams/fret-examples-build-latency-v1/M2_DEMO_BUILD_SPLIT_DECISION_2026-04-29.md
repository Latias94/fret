# Fret Examples Build Latency v1 - M2 Demo Build Split Decision - 2026-04-29

Status: complete

## Decision

Use separate example-family crates for heavy demo families. The first proof is
`apps/fret-examples-imui`, which owns the `imui_shadcn_adapter_demo` source and exposes a direct
`imui_shadcn_adapter_demo` bin.

`apps/fret-examples` may temporarily re-export the moved module while older demo selectors migrate,
but the fast iteration target must not depend on `fret-examples`.

## Rationale

- `apps/fret-demo/src/bin/*` targets live in one Cargo package, so package dependencies are not a
  clean per-bin isolation boundary.
- Feature-family gating would require users and tools to remember the right feature/no-default
  feature combination for each demo target.
- A separate family crate gives each heavy demo family its own dependency closure, runnable bins, and
  source-policy gates without changing framework contracts.

## Proof Slice

- Moved `imui_shadcn_adapter_demo` source ownership from
  `apps/fret-examples/src/imui_shadcn_adapter_demo.rs` to
  `apps/fret-examples-imui/src/imui_shadcn_adapter_demo.rs`.
- Added `apps/fret-examples-imui/src/bin/imui_shadcn_adapter_demo.rs`.
- Kept the legacy `fret_examples::imui_shadcn_adapter_demo` path as a re-export from
  `apps/fret-examples/src/lib.rs`.
- Updated source-policy gates to read the new owner path.
- Checked `cargo tree -p fret-examples-imui -e normal`; it has no `fret-examples v...`
  dependency entry.

## Gates

```text
python tools/gate_fret_examples_imui_split_source.py
cargo check -p fret-examples-imui --bin imui_shadcn_adapter_demo --jobs 1
cargo check -p fret-demo --bin imui_shadcn_adapter_demo --jobs 1
cargo check -p fret-examples --lib --jobs 1
```

## Next Step

Migrate the remaining IMUI proof/demo sources into `apps/fret-examples-imui` in small packages,
starting with the demos that do not depend on `crate::` helpers in `fret-examples`.
