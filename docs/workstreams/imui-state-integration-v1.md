# imui + State Integration v1 (Service-First)

Status: Draft (guidance note)
Last updated: 2026-02-06

Related:

- `docs/workstreams/state-management-v1.md`
- `docs/workstreams/state-management-v1-extension-contract.md`
- `docs/workstreams/component-ecosystem-state-integration-v1.md`
- `docs/workstreams/imui-ecosystem-facade-v1.md`
- `docs/integrating-tokio-and-reqwest.md`

This note defines the recommended way to combine immediate-mode authoring (`imui`) with
`fret-selector` and `fret-query`.

## 1) Core stance

`imui` stays policy-light and state-stack agnostic.

- `fret-imui` should not require selector/query in its core API.
- Host apps own state and orchestration.
- Selector/query remain optional integration helpers, not mandatory framework coupling.

In practice: **service-first orchestration** in app code, then pass plain snapshots into immediate draws.

## 2) Ownership split

- Host app:
  - owns `Model<T>` and command handlers,
  - runs selector/query orchestration,
  - converts domain state to immediate draw snapshots.
- `imui` wrapper/component:
  - consumes plain values + typed callbacks,
  - emits typed intents (`MessageRouter`/`KeyedMessageRouter`),
  - avoids hidden async fetch or hidden selector dependencies.

## 3) Host-side orchestration pattern

```rust,ignore
struct TodoSnapshot {
    total: usize,
    active: usize,
    completed: usize,
    tip_label: String,
    loading: bool,
}

fn build_snapshot(cx: &mut ElementContext<'_, App>, st: &TodoState) -> TodoSnapshot {
    // 1) selector for derived counts
    let counts = cx.use_selector(
        |_cx| {
            // declare deps: revisions / model ids
            // keep explicit and auditable
            TodoDeps::from_models(st)
        },
        |_cx| derive_counts(st),
    );

    // 2) query for async tip/resource
    let tip_handle = cx.use_query_async(tip_key(), tip_policy(), move |_token| async move {
        // e.g. reqwest/json fetch here
        fetch_tip().await
    });

    let tip_state = cx
        .watch_model(tip_handle.model())
        .layout()
        .cloned()
        .unwrap_or_default();

    TodoSnapshot {
        total: counts.total,
        active: counts.active,
        completed: counts.completed,
        tip_label: format_tip(&tip_state),
        loading: tip_state.is_loading(),
    }
}

fn draw_immediate(ui: &mut impl UiWriter, snapshot: &TodoSnapshot, cmds: &TodoCommands) {
    // immediate draw stays data-oriented
    // no hidden fetch; no hidden dependency observation
    ui.label(format!(
        "{} total | {} active | {} completed",
        snapshot.total, snapshot.active, snapshot.completed
    ));
    ui.label(snapshot.tip_label.clone());
    if ui.button("Refresh tip").clicked() {
        ui.command(cmds.refresh_tip.clone());
    }
}
```

## 4) Common ecosystem integration scenarios

### 4.1 `reqwest` (REST/HTTP snapshot data)

Use `fret-query` for resource lifecycle and caching:

- run HTTP request in query fetch closure,
- map errors to retryable/permanent categories,
- invalidate by namespace after successful mutation.

Good fits:

- todo lists, issue tables, settings pages, metadata panes.

### 4.2 `sqlx` / local DB

- read paths: query-backed snapshots (`load_tasks`, `load_profile`),
- write paths: typed commands + transaction handling,
- post-write: invalidate related query keys.

### 4.3 websocket/SSE/realtime

Treat stream as reducer pipeline first:

- background producer -> inbox messages -> model reduce,
- optional query invalidation only at consistency boundaries,
- do not force high-frequency streams into query polling loops.

## 5) Third-party crate guidance

If an ecosystem crate wants optional selector/query support:

- keep core APIs state-agnostic,
- expose optional adapters behind features:
  - `state-selector`
  - `state-query`
  - `state` (umbrella)
- keep adapter modules explicit (`src/state.rs` or `src/state/*`).

## 6) Checklist for maintainers

1. Keep primitive/immediate core APIs state-agnostic.
2. Keep observation/invalidation explicit and reviewable.
3. Keep async fetch in query service layer, not in drawing helpers.
4. Keep command routing typed for dynamic actions.
5. Validate with gates (`check_component_state_coupling.ps1`, targeted checks/nextest).