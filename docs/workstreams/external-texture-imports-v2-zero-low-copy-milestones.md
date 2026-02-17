# External Texture Imports v2 (Zero/Low-Copy) — Milestones

## M0 — Design lock (time-boxed)

Deliverables:

- ADR 0282 is updated to “executable” detail (capabilities, fallback chain, metadata semantics).
- This workstream’s tracking docs exist and are linked:
  - `docs/workstreams/external-texture-imports-v2-zero-low-copy.md`
  - `docs/workstreams/external-texture-imports-v2-zero-low-copy-todo.md`
  - `docs/workstreams/external-texture-imports-v2-zero-low-copy-milestones.md`

Exit criteria:

- A contributor can answer “what happens on wasm/mobile when strategy X is requested” without
  reading code.

## M1 — Metadata closure (portable)

Deliverables:

- The imported-target metadata surface is sufficient for correctness parity for real media
  sources, and is bounded (no open-ended strings).
- Deterministic degradation rules exist for any metadata not representable on a target/backend.
- Diagnostics surface explicit counters/hints for metadata degradations.
  - Minimum expected set for “real media” correctness:
    - alpha mode + orientation (already required by v1),
    - bounded color encoding hints (primaries, transfer, matrix, range).

Exit criteria:

- Conformance test(s) exist for at least alpha + orientation semantics on imported targets.

## M2 — Native low/zero-copy uplift (capability-gated)

Deliverables:

- A capability-gated native ingestion path exists where feasible (platform decoder / GPU frame
  integration), with deterministic fallback and per-frame attribution in perf snapshots.
- The implementation is staged to reduce risk:
  - **M2A (real source closure)**: a real native frame source is wired end-to-end (e.g. Windows MF),
    even if it initially lands as `CpuUpload`/`GpuCopy`, with stable metadata semantics and gates.
  - **M2B (ceiling uplift)**: a true zero/low-copy path is added behind explicit capabilities
    (e.g. D3D12-only fast path on Windows), with deterministic fallback to the copy path.
    - Note: “true external-handle import” may be blocked by upstream APIs. In that case, M2B may be
      satisfied by a capability-gated **shared allocation** path (producer writes into a
      renderer-owned `wgpu::Texture`), which can still be “no-copy” while classifying as `Owned`.
    - Minimum proof for “shared allocation”:
      - a synthetic native writer can update a renderer-owned texture on a supported backend (e.g. DX12),
      - state transitions are deterministic and validated (no silent hazards),
      - a lightweight gate exists (diag correctness and/or steady perf baseline).
- A steady-state perf baseline exists for each landed path (non-regression + expected delta recorded).

Exit criteria:

- “requested vs effective” ingest strategy distributions are stable and observable in perf bundles.

## M3 — Mobile plan + gates (capability-gated)

Deliverables:

- iOS + Android strategy feasibility is documented with explicit prerequisites and fallback rules.
- At least one mobile steady-state perf baseline exists (even if it proves “copy-only for now”).

Exit criteria:

- Mobile readiness checklist exists in the ADR + workstream, and is enforced by gates where possible.

## M4 — Web zero-copy (blocked until backend support)

Deliverables:

- Once backend support exists, land WebGPU `ExternalTexture` sampling behind capabilities with
  deterministic fallback to the existing GPU-copy path.
- Add web steady-state perf baselines for zero-copy vs copy (explicit headroom).
