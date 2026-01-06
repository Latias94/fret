#[cfg(feature = "alloc-profile")]
use std::alloc::{GlobalAlloc, Layout, System};
#[cfg(feature = "alloc-profile")]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(feature = "alloc-profile")]
#[derive(Default, Clone, Copy)]
pub struct AllocProfileSnapshot {
    pub alloc_calls: u64,
    pub alloc_bytes: u64,
    pub dealloc_calls: u64,
    pub realloc_calls: u64,
}

#[cfg(feature = "alloc-profile")]
struct CountingAllocator;

#[cfg(feature = "alloc-profile")]
static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "alloc-profile")]
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "alloc-profile")]
static DEALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
#[cfg(feature = "alloc-profile")]
static REALLOC_CALLS: AtomicU64 = AtomicU64::new(0);

#[cfg(feature = "alloc-profile")]
#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

#[cfg(feature = "alloc-profile")]
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let _ = layout;
        DEALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let _ = (layout, new_size);
        REALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[cfg(feature = "alloc-profile")]
pub fn snapshot() -> AllocProfileSnapshot {
    AllocProfileSnapshot {
        alloc_calls: ALLOC_CALLS.load(Ordering::Relaxed),
        alloc_bytes: ALLOC_BYTES.load(Ordering::Relaxed),
        dealloc_calls: DEALLOC_CALLS.load(Ordering::Relaxed),
        realloc_calls: REALLOC_CALLS.load(Ordering::Relaxed),
    }
}

#[cfg(feature = "alloc-profile")]
pub fn reset() {
    ALLOC_CALLS.store(0, Ordering::Relaxed);
    ALLOC_BYTES.store(0, Ordering::Relaxed);
    DEALLOC_CALLS.store(0, Ordering::Relaxed);
    REALLOC_CALLS.store(0, Ordering::Relaxed);
}

#[cfg(not(feature = "alloc-profile"))]
#[derive(Default, Clone, Copy)]
pub struct AllocProfileSnapshot {
    pub alloc_calls: u64,
    pub alloc_bytes: u64,
    pub dealloc_calls: u64,
    pub realloc_calls: u64,
}

#[cfg(not(feature = "alloc-profile"))]
pub fn snapshot() -> AllocProfileSnapshot {
    AllocProfileSnapshot::default()
}

#[cfg(not(feature = "alloc-profile"))]
pub fn reset() {}
