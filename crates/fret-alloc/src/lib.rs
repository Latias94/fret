#[cfg(all(feature = "mimalloc", feature = "jemalloc"))]
compile_error!("fret-alloc: features `mimalloc` and `jemalloc` are mutually exclusive");

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

pub const ALLOCATOR_NAME: &str = if cfg!(feature = "mimalloc") {
    "mimalloc"
} else if cfg!(feature = "jemalloc") {
    "jemalloc"
} else {
    "system"
};

pub fn allocator_name() -> &'static str {
    ALLOCATOR_NAME
}
