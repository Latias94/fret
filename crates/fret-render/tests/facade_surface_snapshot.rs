use fret_render::{
    AdapterCapabilities, BlurQualityCounters, BlurQualitySnapshot, ClearColor,
    EffectDegradationCounters, EffectDegradationSnapshot, ImageColorSpace, ImageDescriptor,
    IntermediatePerfSnapshot, RenderError, RenderPerfSnapshot, RenderSceneParams,
    RenderTargetAlphaMode, RenderTargetColorEncoding, RenderTargetColorPrimaries,
    RenderTargetColorRange, RenderTargetColorSpace, RenderTargetDescriptor,
    RenderTargetIngestStrategy, RenderTargetMatrixCoefficients, RenderTargetMetadata,
    RenderTargetOrientation, RenderTargetRotation, RenderTargetTransferFunction, Renderer,
    RendererCapabilities, RendererPerfFrameSample, RendererPerfFrameStore,
    StreamingImageCapabilities, SurfaceState, SvgAlphaMask, SvgPerfSnapshot, SvgRgbaImage,
    UploadedRgba8Image, WgpuAdapterSelectionSnapshot, WgpuAllocatorReportFrameSample,
    WgpuAllocatorReportFrameStore, WgpuAllocatorReportSummary, WgpuAllocatorReportTopAllocation,
    WgpuContext, WgpuHubReportCounts, WgpuHubReportFrameSample, WgpuHubReportFrameStore,
    WgpuInitAttemptSnapshot, WgpuInitDiagnosticsSnapshot, create_rgba8_image_storage,
    upload_alpha_mask, upload_rgba_image, upload_rgba8_image, viewport_overlay,
    write_rgba8_texture_region,
};

#[test]
fn facade_surface_snapshot_matches_v1_contract_buckets() {
    // Bucket A: core runtime/bootstrap entrypoints.
    let _ = std::mem::size_of::<Renderer>();
    let _ = std::mem::size_of::<RenderSceneParams<'static>>();
    let _ = std::mem::size_of::<SurfaceState<'static>>();
    let _ = std::mem::size_of::<WgpuContext>();
    let _ = std::mem::size_of::<ClearColor>();
    let _ = std::mem::size_of::<RenderError>();
    let _ = Renderer::new;
    let _ = Renderer::render_scene;
    let _ = SurfaceState::new;
    let _ = SurfaceState::new_with_usage;
    let _ = WgpuContext::new;
    let _ = WgpuContext::new_with_backends;

    // Bucket B: capability and adapter snapshots.
    let _ = std::mem::size_of::<AdapterCapabilities>();
    let _ = std::mem::size_of::<StreamingImageCapabilities>();
    let _ = std::mem::size_of::<RendererCapabilities>();
    let _ = std::mem::size_of::<WgpuAdapterSelectionSnapshot>();
    let _ = std::mem::size_of::<WgpuInitDiagnosticsSnapshot>();
    let _ = std::mem::size_of::<WgpuInitAttemptSnapshot>();
    let _ = RendererCapabilities::from_wgpu_context;
    let _ = RendererCapabilities::from_adapter_device;
    let _ = WgpuAdapterSelectionSnapshot::from_context;
    let _ = WgpuAdapterSelectionSnapshot::from_adapter;

    // Bucket C: render-target and ingest contracts.
    let _ = std::mem::size_of::<RenderTargetDescriptor>();
    let _ = std::mem::size_of::<RenderTargetMetadata>();
    let _ = std::mem::size_of::<RenderTargetColorSpace>();
    let _ = std::mem::size_of::<RenderTargetColorEncoding>();
    let _ = std::mem::size_of::<RenderTargetColorPrimaries>();
    let _ = std::mem::size_of::<RenderTargetColorRange>();
    let _ = std::mem::size_of::<RenderTargetMatrixCoefficients>();
    let _ = std::mem::size_of::<RenderTargetTransferFunction>();
    let _ = std::mem::size_of::<RenderTargetOrientation>();
    let _ = std::mem::size_of::<RenderTargetRotation>();
    let _ = std::mem::size_of::<RenderTargetAlphaMode>();
    let _ = std::mem::size_of::<RenderTargetIngestStrategy>();

    // Bucket B continued: diagnostics/report stores and snapshots.
    let _ = std::mem::size_of::<BlurQualityCounters>();
    let _ = std::mem::size_of::<BlurQualitySnapshot>();
    let _ = std::mem::size_of::<EffectDegradationCounters>();
    let _ = std::mem::size_of::<EffectDegradationSnapshot>();
    let _ = std::mem::size_of::<RenderPerfSnapshot>();
    let _ = std::mem::size_of::<IntermediatePerfSnapshot>();
    let _ = std::mem::size_of::<SvgPerfSnapshot>();
    let _ = std::mem::size_of::<RendererPerfFrameSample>();
    let _ = std::mem::size_of::<RendererPerfFrameStore>();
    let _ = std::mem::size_of::<WgpuHubReportCounts>();
    let _ = std::mem::size_of::<WgpuHubReportFrameSample>();
    let _ = std::mem::size_of::<WgpuHubReportFrameStore>();
    let _ = std::mem::size_of::<WgpuAllocatorReportFrameSample>();
    let _ = std::mem::size_of::<WgpuAllocatorReportFrameStore>();
    let _ = std::mem::size_of::<WgpuAllocatorReportSummary>();
    let _ = std::mem::size_of::<WgpuAllocatorReportTopAllocation>();

    // Buckets D/E: image upload helpers, SVG helpers, and viewport overlays.
    let _ = std::mem::size_of::<ImageDescriptor>();
    let _ = std::mem::size_of::<ImageColorSpace>();
    let _ = std::mem::size_of::<UploadedRgba8Image>();
    let _ = std::mem::size_of::<SvgAlphaMask>();
    let _ = std::mem::size_of::<SvgRgbaImage>();
    let _ = std::mem::size_of::<viewport_overlay::ViewportOverlay3dContext>();
    let _ = create_rgba8_image_storage;
    let _ = upload_rgba8_image;
    let _ = write_rgba8_texture_region;
    let _ = upload_alpha_mask;
    let _ = upload_rgba_image;
    let _ = viewport_overlay::run_overlays;
}
