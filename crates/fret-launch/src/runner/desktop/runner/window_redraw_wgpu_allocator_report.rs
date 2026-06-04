use fret_app::App;
use fret_core::AppWindowId;

pub(super) fn maybe_record_window_redraw_wgpu_allocator_report(
    app: &mut App,
    context: &fret_render::WgpuContext,
    app_window: AppWindowId,
    tick_id: u64,
    frame_id: u64,
) {
    let diag_wgpu_allocator_report =
        std::env::var_os("FRET_DIAG_WGPU_ALLOCATOR_REPORT").is_some_and(|v| !v.is_empty());
    if !diag_wgpu_allocator_report {
        return;
    }

    let every_n = std::env::var("FRET_DIAG_WGPU_ALLOCATOR_REPORT_EVERY_N_FRAMES")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(300)
        .max(1);
    let top_n = std::env::var("FRET_DIAG_WGPU_ALLOCATOR_REPORT_TOP_N")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(16)
        .max(1);
    let max_name_bytes = std::env::var("FRET_DIAG_WGPU_ALLOCATOR_REPORT_MAX_NAME_BYTES")
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .unwrap_or(160)
        .max(16);

    let should_sample = frame_id <= 2 || frame_id.is_multiple_of(every_n);
    if !should_sample {
        return;
    }

    let report = context.device.generate_allocator_report();
    #[cfg(target_os = "macos")]
    let metal_current_allocated_size_bytes = unsafe {
        context
            .device
            .as_hal::<wgpu::hal::api::Metal>()
            .map(|dev| dev.raw_device().currentAllocatedSize() as u64)
    };
    #[cfg(not(target_os = "macos"))]
    let metal_current_allocated_size_bytes: Option<u64> = None;

    app.with_global_mut_untracked(
        fret_render::WgpuAllocatorReportFrameStore::default,
        |store, _app| {
            store.record_sample(
                app_window,
                tick_id,
                frame_id,
                report,
                metal_current_allocated_size_bytes,
                top_n,
                max_name_bytes,
            );
        },
    );
}
