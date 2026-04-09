use fret_core::AppWindowId;
use std::collections::HashMap;
use wgpu::AllocatorReport;

#[derive(Debug, Default, Clone, Copy)]
pub struct WgpuHubReportCounts {
    pub adapters: u64,
    pub devices: u64,
    pub queues: u64,
    pub command_encoders: u64,
    pub buffers: u64,
    pub textures: u64,
    pub texture_views: u64,
    pub samplers: u64,
    pub shader_modules: u64,
    pub render_pipelines: u64,
    pub compute_pipelines: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WgpuHubReportFrameSample {
    pub tick_id: u64,
    pub frame_id: u64,
    pub counts: WgpuHubReportCounts,
}

/// Best-effort store for the most recent wgpu hub report sample per window.
///
/// This is intended for diagnostics bundles (`fretboard-dev diag *`) and should not be used as a
/// correctness signal.
#[derive(Debug, Default)]
pub struct WgpuHubReportFrameStore {
    by_window: HashMap<AppWindowId, WgpuHubReportFrameSample>,
}

impl WgpuHubReportFrameStore {
    pub fn record(
        &mut self,
        window: AppWindowId,
        tick_id: u64,
        frame_id: u64,
        counts: WgpuHubReportCounts,
    ) {
        self.by_window.insert(
            window,
            WgpuHubReportFrameSample {
                tick_id,
                frame_id,
                counts,
            },
        );
    }

    pub fn latest_for_window(&self, window: AppWindowId) -> Option<WgpuHubReportFrameSample> {
        self.by_window.get(&window).copied()
    }
}

#[derive(Debug, Default, Clone)]
pub struct WgpuAllocatorReportTopAllocation {
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct WgpuAllocatorReportSummary {
    pub allocator_report_present: bool,
    pub total_allocated_bytes: u64,
    pub total_reserved_bytes: u64,
    pub blocks: u64,
    pub allocations: u64,
    pub metal_current_allocated_size_bytes: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct WgpuAllocatorReportFrameSample {
    pub tick_id: u64,
    pub frame_id: u64,
    pub summary: WgpuAllocatorReportSummary,
    pub top_allocations: Vec<WgpuAllocatorReportTopAllocation>,
}

/// Best-effort store for the most recent wgpu allocator report sample per window.
///
/// This is intended for diagnostics bundles (`fretboard-dev diag *`) and should not be used as a
/// correctness signal.
#[derive(Debug, Default)]
pub struct WgpuAllocatorReportFrameStore {
    by_window: HashMap<AppWindowId, WgpuAllocatorReportFrameSample>,
}

impl WgpuAllocatorReportFrameStore {
    pub fn record_sample(
        &mut self,
        window: AppWindowId,
        tick_id: u64,
        frame_id: u64,
        report: Option<AllocatorReport>,
        metal_current_allocated_size_bytes: Option<u64>,
        max_top_allocations: usize,
        max_name_bytes: usize,
    ) {
        let mut summary = WgpuAllocatorReportSummary {
            allocator_report_present: report.is_some(),
            metal_current_allocated_size_bytes,
            ..WgpuAllocatorReportSummary::default()
        };

        let mut top_allocations: Vec<WgpuAllocatorReportTopAllocation> = Vec::new();

        if let Some(report) = report {
            summary.total_allocated_bytes = report.total_allocated_bytes;
            summary.total_reserved_bytes = report.total_reserved_bytes;
            summary.blocks = report.blocks.len() as u64;

            let mut allocations = report.allocations;
            summary.allocations = allocations.len() as u64;
            allocations.sort_unstable_by_key(|alloc| std::cmp::Reverse(alloc.size));
            allocations.truncate(max_top_allocations);

            top_allocations = Vec::with_capacity(allocations.len());
            for allocation in allocations {
                let mut name = allocation.name;
                truncate_string_utf8_bytes(&mut name, max_name_bytes);
                top_allocations.push(WgpuAllocatorReportTopAllocation {
                    name,
                    size: allocation.size,
                });
            }
        }

        self.by_window.insert(
            window,
            WgpuAllocatorReportFrameSample {
                tick_id,
                frame_id,
                summary,
                top_allocations,
            },
        );
    }

    pub fn latest_for_window(&self, window: AppWindowId) -> Option<WgpuAllocatorReportFrameSample> {
        self.by_window.get(&window).cloned()
    }
}

fn truncate_string_utf8_bytes(s: &mut String, max_bytes: usize) {
    if s.len() <= max_bytes {
        return;
    }

    let mut cut = max_bytes;
    while cut > 0 && !s.is_char_boundary(cut) {
        cut -= 1;
    }
    s.truncate(cut);
}
