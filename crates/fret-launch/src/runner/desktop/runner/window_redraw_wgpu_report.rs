use fret_app::App;
use fret_core::AppWindowId;

pub(super) fn maybe_record_window_redraw_wgpu_hub_report(
    app: &mut App,
    context: &fret_render::WgpuContext,
    app_window: AppWindowId,
    tick_id: u64,
    frame_id: u64,
) {
    let diag_wgpu_report = std::env::var_os("FRET_DIAG_WGPU_REPORT").is_some_and(|v| !v.is_empty());
    if !diag_wgpu_report {
        return;
    }

    let every_n = std::env::var("FRET_DIAG_WGPU_REPORT_EVERY_N_FRAMES")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or(60)
        .max(1);

    let should_sample = frame_id <= 2 || frame_id.is_multiple_of(every_n);
    if !should_sample {
        return;
    }

    let Some(report) = context.instance.generate_report() else {
        return;
    };

    let hub = report.hub_report();
    let counts = fret_render::WgpuHubReportCounts {
        adapters: (hub.adapters.num_allocated + hub.adapters.num_kept_from_user) as u64,
        devices: (hub.devices.num_allocated + hub.devices.num_kept_from_user) as u64,
        queues: (hub.queues.num_allocated + hub.queues.num_kept_from_user) as u64,
        command_encoders: (hub.command_encoders.num_allocated
            + hub.command_encoders.num_kept_from_user) as u64,
        buffers: (hub.buffers.num_allocated + hub.buffers.num_kept_from_user) as u64,
        textures: (hub.textures.num_allocated + hub.textures.num_kept_from_user) as u64,
        texture_views: (hub.texture_views.num_allocated + hub.texture_views.num_kept_from_user)
            as u64,
        samplers: (hub.samplers.num_allocated + hub.samplers.num_kept_from_user) as u64,
        shader_modules: (hub.shader_modules.num_allocated + hub.shader_modules.num_kept_from_user)
            as u64,
        render_pipelines: (hub.render_pipelines.num_allocated
            + hub.render_pipelines.num_kept_from_user) as u64,
        compute_pipelines: (hub.compute_pipelines.num_allocated
            + hub.compute_pipelines.num_kept_from_user) as u64,
    };

    app.with_global_mut_untracked(
        fret_render::WgpuHubReportFrameStore::default,
        |store, _app| {
            store.record(app_window, tick_id, frame_id, counts);
        },
    );
}
