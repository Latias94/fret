use fret_core::{AppWindowId, FrameId, Rect};
use std::{
    any::{Any, TypeId},
    collections::HashMap,
    hash::{Hash, Hasher},
    panic::Location,
};

use crate::UiHost;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalElementId(pub u64);

#[derive(Default)]
pub struct ElementRuntime {
    windows: HashMap<AppWindowId, WindowElementState>,
    gc_lag_frames: u64,
}

impl ElementRuntime {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            gc_lag_frames: 2,
        }
    }

    pub fn gc_lag_frames(&self) -> u64 {
        self.gc_lag_frames
    }

    pub fn set_gc_lag_frames(&mut self, frames: u64) {
        self.gc_lag_frames = frames;
    }

    pub fn for_window_mut(&mut self, window: AppWindowId) -> &mut WindowElementState {
        self.windows.entry(window).or_default()
    }

    pub fn prepare_window_for_frame(&mut self, window: AppWindowId, frame_id: FrameId) {
        let lag = self.gc_lag_frames;
        self.for_window_mut(window).prepare_for_frame(frame_id, lag);
    }
}

#[derive(Default)]
pub struct WindowElementState {
    state: HashMap<(GlobalElementId, TypeId), StateEntry>,
    prepared_frame: FrameId,
    prev_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
    cur_unkeyed_fingerprints: HashMap<u64, Vec<u64>>,
}

#[derive(Debug)]
struct StateEntry {
    value: Box<dyn Any>,
    last_seen_frame: FrameId,
}

impl WindowElementState {
    fn prepare_for_frame(&mut self, frame_id: FrameId, lag_frames: u64) {
        if self.prepared_frame == frame_id {
            return;
        }
        self.prepared_frame = frame_id;

        let cutoff = frame_id.0.saturating_sub(lag_frames);
        self.state.retain(|_, e| e.last_seen_frame.0 >= cutoff);

        std::mem::swap(
            &mut self.prev_unkeyed_fingerprints,
            &mut self.cur_unkeyed_fingerprints,
        );
        self.cur_unkeyed_fingerprints.clear();
    }
}

pub struct ElementCx<'a, H: UiHost = fret_app::App> {
    pub app: &'a mut H,
    pub window: AppWindowId,
    pub frame_id: FrameId,
    pub bounds: Rect,
    window_state: &'a mut WindowElementState,
    stack: Vec<GlobalElementId>,
    child_counters: Vec<u32>,
}

impl<'a, H: UiHost> ElementCx<'a, H> {
    pub fn new(
        app: &'a mut H,
        runtime: &'a mut ElementRuntime,
        window: AppWindowId,
        bounds: Rect,
        root: GlobalElementId,
    ) -> Self {
        let frame_id = app.frame_id();
        runtime.prepare_window_for_frame(window, frame_id);

        let window_state = runtime.for_window_mut(window);

        Self {
            app,
            window,
            frame_id,
            bounds,
            window_state,
            stack: vec![root],
            child_counters: vec![0],
        }
    }

    pub fn new_for_root_name(
        app: &'a mut H,
        runtime: &'a mut ElementRuntime,
        window: AppWindowId,
        bounds: Rect,
        root_name: &str,
    ) -> Self {
        Self::new(app, runtime, window, bounds, global_root(window, root_name))
    }

    pub fn root_id(&self) -> GlobalElementId {
        *self.stack.last().expect("root exists")
    }

    #[track_caller]
    pub fn scope<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let caller = callsite_hash(Location::caller());
        self.enter(caller, None, f)
    }

    #[track_caller]
    pub fn keyed<K: Hash, R>(&mut self, key: K, f: impl FnOnce(&mut Self) -> R) -> R {
        let caller = callsite_hash(Location::caller());
        let key_hash = stable_hash(&key);
        self.enter(caller, Some(key_hash), f)
    }

    pub fn with_state<S: Any, R>(
        &mut self,
        init: impl FnOnce() -> S,
        f: impl FnOnce(&mut S) -> R,
    ) -> R {
        let id = self.root_id();
        let key = (id, TypeId::of::<S>());

        let entry = self
            .window_state
            .state
            .entry(key)
            .or_insert_with(|| StateEntry {
                value: Box::new(init()),
                last_seen_frame: self.frame_id,
            });
        entry.last_seen_frame = self.frame_id;

        let state = entry
            .value
            .downcast_mut::<S>()
            .expect("element state type mismatch");
        f(state)
    }

    #[track_caller]
    pub fn for_each_keyed<T, K: Hash>(
        &mut self,
        items: &[T],
        mut key: impl FnMut(&T) -> K,
        mut f: impl FnMut(&mut Self, usize, &T),
    ) {
        self.scope(|cx| {
            for (index, item) in items.iter().enumerate() {
                let k = key(item);
                cx.keyed(k, |cx| f(cx, index, item));
            }
        });
    }

    #[track_caller]
    pub fn for_each_unkeyed<T: Hash>(
        &mut self,
        items: &[T],
        mut f: impl FnMut(&mut Self, usize, &T),
    ) {
        let list_id = callsite_hash(Location::caller());
        let fingerprints: Vec<u64> = items.iter().map(stable_hash).collect();
        self.window_state
            .cur_unkeyed_fingerprints
            .insert(list_id, fingerprints.clone());

        if let Some(prev) = self.window_state.prev_unkeyed_fingerprints.get(&list_id) {
            if prev != &fingerprints && items.len() > 1 {
                if cfg!(debug_assertions) {
                    tracing::warn!(
                        list_id = format_args!("{list_id:#x}"),
                        "unkeyed element list order changed; add explicit keys to preserve state"
                    );
                }
            }
        }

        self.scope(|cx| {
            for (index, item) in items.iter().enumerate() {
                let index_key = index as u64;
                cx.enter(list_id, Some(index_key), |cx| f(cx, index, item));
            }
        });
    }

    fn enter<R>(
        &mut self,
        callsite: u64,
        key_hash: Option<u64>,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let parent = self.root_id();
        let child_index = self.child_counters.last_mut().expect("counter exists");
        let slot = *child_index as u64;
        *child_index = child_index.saturating_add(1);

        let child_salt = key_hash.unwrap_or(slot);
        let id = derive_child_id(parent, callsite, child_salt);

        self.stack.push(id);
        self.child_counters.push(0);
        let out = f(self);
        self.child_counters.pop();
        self.stack.pop();
        out
    }
}

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
