# Query Key Conventions (`fret-query`)

Status: Draft (work-in-progress guidance; APIs may change)

This document defines **recommended conventions** for using `ecosystem/fret-query` keys in a way
that stays predictable across refactors and scales to large, editor-grade apps.

## What a query key is

In Fret, a query key is:

- **typed** by result type: `QueryKey<T>`
- a pair of:
  - a `'static` **namespace** (`&'static str`)
  - a **64-bit stable hash** of a structured key value (`impl Hash`)

Construction:

- `QueryKey::<T>::new(namespace, &key_value)`
- `QueryKey::<T>::new_named(namespace, &key_value, "debug label")` (optional)

The namespace is used for bulk invalidation (e.g. “all queries in this subsystem”).

## Namespace conventions

Recommended:

- **Dot-separated** and scoped: `"crate.subsystem.query_name.v1"`.
- Store namespaces as **constants** near the query definition:
  - `const NS: &str = "...";`
- Treat the final segment as a **version**:
  - bump it when you change key semantics or want to invalidate old caches naturally.

Examples:

- `"fret-examples.markdown.remote_image.v1"`
- `"my_app.workspace.settings.v2"`
- `"my_crate.git.status_for_repo.v1"`

## Structured key conventions

Your `key_value` must:

- be **deterministic** (same inputs → same hash),
- contain **all parameters** that affect the fetch result,
- avoid non-deterministic containers and unstable identity.

Recommended patterns:

1) Small tuples:

- `&(repo_id, path, include_ignored)`

2) A dedicated newtype that derives `Hash`:

```rust
#[derive(Hash)]
struct UserKey {
    user_id: u64,
    include_deleted: bool,
}
```

Avoid (common footguns):

- `HashMap` / `HashSet` as a key (iteration order is not stable).
- Floating point values (NaN and rounding edge cases); prefer quantized integers.
- Pointer identity (addresses, `*const T`) or transient IDs that change across runs.
- “Everything in one giant struct” keys when only a small subset affects the result.

If you need a map-like key, sort it into a `Vec<(K, V)>` first or use a deterministic map
representation.

## Invalidation strategy

Prefer:

- **Targeted invalidation** via exact keys (single resource).
- **Bulk invalidation** via namespaces for subsystem-level changes:
  - `QueryClient::invalidate_namespace("crate.subsystem.*")` is *not* supported today; namespaces
    must match exactly. Use one namespace per group you want to invalidate together.

If you need “invalidate all queries under a prefix”, model it explicitly with one shared namespace
constant per group (and call `invalidate_namespace` on each group).

## Collision note

The structured key is hashed to 64-bit. Collisions are possible in theory, but should be extremely
rare in practice when namespaces are scoped and keys are structured.

## Debugging keys

In debug builds, `fret-query` may emit **one-time warnings** for suspicious namespaces (missing a
scope separator, uppercase/whitespace, missing a `.vN` suffix, etc.).

If you need better diagnostics for collisions or query lifecycle tracing, use
`QueryKey::new_named(...)` to attach a human-readable label (it does **not** affect key equality).
