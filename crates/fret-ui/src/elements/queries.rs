pub fn global_root(window: AppWindowId, name: &str) -> GlobalElementId {
    let mut h = Fnv1a64::default();
    window.hash(&mut h);
    h.write(name.as_bytes());
    GlobalElementId(h.finish())
}

pub fn with_element_cx<H: UiHost, R>(
    app: &mut H,
    window: AppWindowId,
    bounds: Rect,
    root_name: &str,
    f: impl FnOnce(&mut ElementCx<'_, H>) -> R,
) -> R {
    app.with_global_mut(ElementRuntime::new, |runtime, app| {
        let mut cx = ElementCx::new_for_root_name(app, runtime, window, bounds, root_name);
        f(&mut cx)
    })
}

pub fn root_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<Rect> {
    app.with_global_mut(ElementRuntime::new, |runtime, _app| {
        let state = runtime.for_window_mut(window);
        let root = state.node_entry(element).map(|e| e.root)?;
        state.root_bounds(root)
    })
}

pub fn node_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<NodeId> {
    with_window_state(app, window, |st| st.node_entry(element).map(|e| e.node))
}

/// Returns the last frame's bounds for a declarative element, if available.
///
/// This is a cross-frame geometry query intended for component-layer policies (e.g. anchored
/// overlays) that need a stable anchor rect. The value is updated during layout.
pub fn bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<Rect> {
    with_window_state(app, window, |st| st.last_bounds(element))
}

/// Returns the last frame's **visual** bounds (post-`render_transform` AABB) for a declarative
/// element, if available.
///
/// This is a cross-frame geometry query intended for component-layer anchored overlay policies
/// that must track render transforms (ADR 0083) while keeping layout authoritative.
pub fn visual_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
) -> Option<Rect> {
    with_window_state(app, window, |st| st.last_visual_bounds(element))
}

pub(crate) fn record_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    bounds: Rect,
) {
    with_window_state(app, window, |st| st.record_bounds(element, bounds));
}

pub(crate) fn record_visual_bounds_for_element<H: UiHost>(
    app: &mut H,
    window: AppWindowId,
    element: GlobalElementId,
    bounds: Rect,
) {
    with_window_state(app, window, |st| st.record_visual_bounds(element, bounds));
}

fn derive_child_id(parent: GlobalElementId, callsite: u64, child_salt: u64) -> GlobalElementId {
    let mut h = Fnv1a64::default();
    h.write_u64(parent.0);
    h.write_u64(callsite);
    h.write_u64(child_salt);
    GlobalElementId(h.finish())
}

fn stable_hash<T: Hash>(value: &T) -> u64 {
    let mut h = Fnv1a64::default();
    value.hash(&mut h);
    h.finish()
}

fn callsite_hash(loc: &Location<'_>) -> u64 {
    let mut h = Fnv1a64::default();
    h.write(loc.file().as_bytes());
    h.write_u32(loc.line());
    h.write_u32(loc.column());
    h.finish()
}

#[derive(Default)]
struct Fnv1a64(u64);

impl Hasher for Fnv1a64 {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325
        } else {
            self.0
        };
        for b in bytes {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }
}
