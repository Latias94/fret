use fret_core::RenderTargetId;
use fret_render::{
    RenderTargetColorSpace, RenderTargetDescriptor, RenderTargetIngestStrategy,
    RenderTargetMetadata, Renderer,
};

use crate::common::{EngineFrameKeepalive, EngineFrameUpdate};
use crate::native_external_import::{NativeExternalImportError, NativeExternalTextureFrame};

#[derive(Debug)]
pub enum NativeExternalImportOutcome {
    Imported {
        effective: RenderTargetIngestStrategy,
    },
    FellBack {
        effective: RenderTargetIngestStrategy,
        err: NativeExternalImportError,
    },
}

pub struct ImportedViewportFallbackUpdate {
    pub view: wgpu::TextureView,
    pub size: (u32, u32),
    pub metadata: RenderTargetMetadata,
    pub keepalive: Option<EngineFrameKeepalive>,
}

impl ImportedViewportFallbackUpdate {
    pub fn new(
        view: wgpu::TextureView,
        size: (u32, u32),
        metadata: RenderTargetMetadata,
        keepalive: Option<EngineFrameKeepalive>,
    ) -> Self {
        Self {
            view,
            size,
            metadata,
            keepalive,
        }
    }

    fn into_parts(
        self,
    ) -> (
        wgpu::TextureView,
        (u32, u32),
        RenderTargetMetadata,
        Option<EngineFrameKeepalive>,
    ) {
        (self.view, self.size, self.metadata, self.keepalive)
    }
}

#[derive(Default)]
pub struct ImportedViewportFallbacks {
    pub owned: Option<ImportedViewportFallbackUpdate>,
    pub gpu_copy: Option<ImportedViewportFallbackUpdate>,
    pub cpu_upload: Option<ImportedViewportFallbackUpdate>,
}

impl ImportedViewportFallbacks {
    pub fn single(
        strategy: RenderTargetIngestStrategy,
        update: ImportedViewportFallbackUpdate,
    ) -> Self {
        let mut fallbacks = Self::default();
        match strategy {
            RenderTargetIngestStrategy::Owned => fallbacks.owned = Some(update),
            RenderTargetIngestStrategy::GpuCopy => fallbacks.gpu_copy = Some(update),
            RenderTargetIngestStrategy::CpuUpload => fallbacks.cpu_upload = Some(update),
            RenderTargetIngestStrategy::Unknown | RenderTargetIngestStrategy::ExternalZeroCopy => {
                panic!("unexpected fallback strategy: {strategy:?}")
            }
        }
        fallbacks
    }

    pub fn single_view(
        strategy: RenderTargetIngestStrategy,
        view: wgpu::TextureView,
        size: (u32, u32),
        metadata: RenderTargetMetadata,
        keepalive: Option<EngineFrameKeepalive>,
    ) -> Self {
        Self::single(
            strategy,
            ImportedViewportFallbackUpdate::new(view, size, metadata, keepalive),
        )
    }
}

/// Per-frame imported render target intended to be embedded into the UI via `SceneOp::ViewportSurface`.
///
/// Unlike [`super::ViewportRenderTarget`], this helper does **not** call `renderer.update_render_target(...)`
/// directly after initial registration. Instead, it records registry updates as explicit runner
/// deltas (`EngineFrameUpdate.target_updates`) as locked by ADR 0234.
///
/// This is the intended authoring shape for:
/// - platform-provided per-frame views (e.g. "external texture" on web),
/// - engine subsystems that want to keep registry mutation staged through the runner.
///
/// Notes:
/// - A stable `RenderTargetId` is still renderer-owned. The first registration needs `&mut Renderer`.
/// - Subsequent frames should push `RenderTargetUpdate::Update` through `EngineFrameUpdate`.
#[derive(Debug)]
pub struct ImportedViewportRenderTarget {
    id: RenderTargetId,
    format: wgpu::TextureFormat,
    color_space: RenderTargetColorSpace,
}

impl ImportedViewportRenderTarget {
    fn with_ingest_strategies(
        mut metadata: RenderTargetMetadata,
        requested: RenderTargetIngestStrategy,
        effective: RenderTargetIngestStrategy,
    ) -> RenderTargetMetadata {
        metadata.requested_ingest_strategy = requested;
        metadata.ingest_strategy = effective;
        metadata
    }

    pub fn new(format: wgpu::TextureFormat, color_space: RenderTargetColorSpace) -> Self {
        Self {
            id: RenderTargetId::default(),
            format,
            color_space,
        }
    }

    pub fn id(&self) -> RenderTargetId {
        self.id
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn color_space(&self) -> RenderTargetColorSpace {
        self.color_space
    }

    pub fn is_registered(&self) -> bool {
        self.id != RenderTargetId::default()
    }

    /// Ensure this imported target has a stable id.
    ///
    /// This performs the initial `register_render_target(...)` call. Callers typically do this
    /// once during a `gpu_ready(...)` / `gpu_frame_prepare(...)`-style hook.
    pub fn ensure_registered(
        &mut self,
        renderer: &mut Renderer,
        view: wgpu::TextureView,
        size: (u32, u32),
    ) -> RenderTargetId {
        if self.is_registered() {
            return self.id;
        }

        self.ensure_registered_with_metadata(renderer, view, size, RenderTargetMetadata::default())
    }

    pub fn ensure_registered_with_metadata(
        &mut self,
        renderer: &mut Renderer,
        view: wgpu::TextureView,
        size: (u32, u32),
        metadata: RenderTargetMetadata,
    ) -> RenderTargetId {
        if self.is_registered() {
            return self.id;
        }

        let desc = RenderTargetDescriptor {
            view,
            size,
            format: self.format,
            color_space: self.color_space,
            metadata,
        };
        self.id = renderer.register_render_target(desc);
        self.id
    }

    /// Record an imported view update as a runner delta for the current frame.
    ///
    /// Panics if the target has not been registered yet.
    pub fn push_update(
        &self,
        update: &mut EngineFrameUpdate,
        view: wgpu::TextureView,
        size: (u32, u32),
    ) {
        assert!(
            self.is_registered(),
            "ImportedViewportRenderTarget::push_update requires a registered RenderTargetId"
        );
        self.push_update_with_metadata(update, view, size, RenderTargetMetadata::default())
    }

    pub fn push_update_with_metadata(
        &self,
        update: &mut EngineFrameUpdate,
        view: wgpu::TextureView,
        size: (u32, u32),
        metadata: RenderTargetMetadata,
    ) {
        assert!(
            self.is_registered(),
            "ImportedViewportRenderTarget::push_update_with_metadata requires a registered RenderTargetId"
        );
        let desc = RenderTargetDescriptor {
            view,
            size,
            format: self.format,
            color_space: self.color_space,
            metadata,
        };
        update.update_render_target(self.id, desc);
    }

    /// Record an imported view update with explicit requested vs effective ingestion strategies.
    ///
    /// This exists to keep v2 capability-gated fallback behavior observable while still staging
    /// registry mutation through runner deltas (ADR 0234).
    pub fn push_update_with_ingest_strategies(
        &self,
        update: &mut EngineFrameUpdate,
        view: wgpu::TextureView,
        size: (u32, u32),
        metadata: RenderTargetMetadata,
        requested: RenderTargetIngestStrategy,
        effective: RenderTargetIngestStrategy,
    ) {
        self.push_update_with_metadata(
            update,
            view,
            size,
            Self::with_ingest_strategies(metadata, requested, effective),
        );
    }

    /// Record an imported view update while selecting an effective ingestion strategy via the v2
    /// deterministic fallback chain (ADR 0282).
    ///
    /// This helper exists so demos and call sites only need to declare which strategies they can
    /// supply, and do not duplicate the deterministic ordering logic.
    pub fn push_update_with_deterministic_fallback(
        &self,
        update: &mut EngineFrameUpdate,
        view: wgpu::TextureView,
        size: (u32, u32),
        metadata: RenderTargetMetadata,
        requested: RenderTargetIngestStrategy,
        available: &[RenderTargetIngestStrategy],
    ) -> RenderTargetIngestStrategy {
        let effective = Self::select_deterministic_fallback_effective(requested, available);
        self.push_update_with_ingest_strategies(update, view, size, metadata, requested, effective);
        effective
    }

    /// Record an imported view update while selecting an effective ingestion strategy via the v2
    /// deterministic fallback chain (ADR 0282), using explicit fallback payloads.
    ///
    /// Compared to `push_update_with_deterministic_fallback`, this helper removes boilerplate by:
    /// - deriving `available` from the provided payload set,
    /// - selecting the effective strategy deterministically,
    /// - and pushing any per-frame keepalive token when present.
    ///
    /// Callers MUST provide at least one fallback payload.
    pub fn push_update_with_fallbacks(
        &self,
        update: &mut EngineFrameUpdate,
        requested: RenderTargetIngestStrategy,
        mut fallbacks: ImportedViewportFallbacks,
    ) -> RenderTargetIngestStrategy {
        let mut available = Vec::with_capacity(3);
        if fallbacks.owned.is_some() {
            available.push(RenderTargetIngestStrategy::Owned);
        }
        if fallbacks.gpu_copy.is_some() {
            available.push(RenderTargetIngestStrategy::GpuCopy);
        }
        if fallbacks.cpu_upload.is_some() {
            available.push(RenderTargetIngestStrategy::CpuUpload);
        }

        assert!(
            !available.is_empty(),
            "ImportedViewportFallbacks must provide at least one strategy"
        );

        let effective = Self::select_deterministic_fallback_effective(requested, &available);
        let (view, size, metadata, keepalive) = match effective {
            RenderTargetIngestStrategy::Owned => fallbacks
                .owned
                .take()
                .expect("Owned fallback must be present when marked available")
                .into_parts(),
            RenderTargetIngestStrategy::GpuCopy => fallbacks
                .gpu_copy
                .take()
                .expect("GpuCopy fallback must be present when marked available")
                .into_parts(),
            RenderTargetIngestStrategy::CpuUpload => fallbacks
                .cpu_upload
                .take()
                .expect("CpuUpload fallback must be present when marked available")
                .into_parts(),
            RenderTargetIngestStrategy::Unknown | RenderTargetIngestStrategy::ExternalZeroCopy => {
                panic!("unexpected fallback strategy: {effective:?}")
            }
        };

        let metadata = Self::with_ingest_strategies(metadata, requested, effective);
        if let Some(keepalive) = keepalive {
            self.push_update_with_metadata_and_keepalive(update, view, size, metadata, keepalive);
        } else {
            self.push_update_with_metadata(update, view, size, metadata);
        }

        effective
    }

    /// Record an imported view update and a per-frame keepalive token.
    ///
    /// Use this when the imported view depends on an ephemeral external handle (e.g. a WebCodecs
    /// `VideoFrame`) whose lifetime must be extended until submission.
    pub fn push_update_with_keepalive<T: 'static>(
        &self,
        update: &mut EngineFrameUpdate,
        view: wgpu::TextureView,
        size: (u32, u32),
        keepalive: T,
    ) {
        self.push_update(update, view, size);
        update.push_keepalive(keepalive);
    }

    pub fn push_update_with_metadata_and_keepalive(
        &self,
        update: &mut EngineFrameUpdate,
        view: wgpu::TextureView,
        size: (u32, u32),
        metadata: RenderTargetMetadata,
        keepalive: EngineFrameKeepalive,
    ) {
        self.push_update_with_metadata(update, view, size, metadata);
        update.keepalive.push(keepalive);
    }

    fn deterministic_fallback_chain_for_requested(
        requested: RenderTargetIngestStrategy,
    ) -> &'static [RenderTargetIngestStrategy] {
        match requested {
            RenderTargetIngestStrategy::Unknown => &[
                RenderTargetIngestStrategy::Owned,
                RenderTargetIngestStrategy::GpuCopy,
                RenderTargetIngestStrategy::CpuUpload,
            ],
            RenderTargetIngestStrategy::ExternalZeroCopy => &[
                RenderTargetIngestStrategy::ExternalZeroCopy,
                RenderTargetIngestStrategy::Owned,
                RenderTargetIngestStrategy::GpuCopy,
                RenderTargetIngestStrategy::CpuUpload,
            ],
            RenderTargetIngestStrategy::GpuCopy => &[
                RenderTargetIngestStrategy::GpuCopy,
                RenderTargetIngestStrategy::CpuUpload,
            ],
            RenderTargetIngestStrategy::CpuUpload => &[RenderTargetIngestStrategy::CpuUpload],
            RenderTargetIngestStrategy::Owned => &[RenderTargetIngestStrategy::Owned],
        }
    }

    fn select_deterministic_fallback_effective(
        requested: RenderTargetIngestStrategy,
        available: &[RenderTargetIngestStrategy],
    ) -> RenderTargetIngestStrategy {
        let chain = Self::deterministic_fallback_chain_for_requested(requested);
        for candidate in chain {
            if available.contains(candidate) {
                return *candidate;
            }
        }

        // Fallback: if the caller provided an "available" set that doesn't overlap with the
        // deterministic chain, pick the first available value deterministically.
        available
            .first()
            .copied()
            .unwrap_or(RenderTargetIngestStrategy::CpuUpload)
    }

    /// Attempt to import a platform-produced external frame and record a runner delta update.
    ///
    /// This helper implements ADR 0234's staging shape:
    /// - import (capability-gated) in driver code,
    /// - update the renderer registry via explicit deltas,
    /// - carry per-frame keepalive tokens through submission.
    ///
    /// Callers MUST provide deterministic fallback to copy paths when `Err` is returned.
    pub fn push_native_external_import_update(
        &mut self,
        renderer: &mut Renderer,
        update: &mut EngineFrameUpdate,
        ctx: &fret_render::WgpuContext,
        caps: &fret_render::RendererCapabilities,
        frame: Box<dyn NativeExternalTextureFrame>,
    ) -> Result<(), NativeExternalImportError> {
        let imported = frame.import(ctx, caps)?;

        if !self.is_registered() {
            let _ = self.ensure_registered_with_metadata(
                renderer,
                imported.view.clone(),
                imported.size,
                imported.metadata,
            );
        }

        self.push_update_with_metadata_and_keepalive(
            update,
            imported.view,
            imported.size,
            imported.metadata,
            imported.keepalive,
        );

        Ok(())
    }

    /// Attempt a native external import, but deterministically fall back to a caller-provided
    /// update when the import cannot be performed.
    ///
    /// This helper centralizes v2's "bounded strategy set + deterministic fallback chain"
    /// selection (ADR 0282) so demos and call sites don't re-implement the same ordering.
    ///
    /// `fallback_available` declares which effective strategies the caller can provide a fallback
    /// update for, and `fallback_for` must be able to produce an update for the selected strategy.
    pub fn push_native_external_import_update_with_deterministic_fallback(
        &mut self,
        renderer: &mut Renderer,
        update: &mut EngineFrameUpdate,
        ctx: &fret_render::WgpuContext,
        caps: &fret_render::RendererCapabilities,
        requested: RenderTargetIngestStrategy,
        frame: Box<dyn NativeExternalTextureFrame>,
        fallback_available: &[RenderTargetIngestStrategy],
        mut fallback_for: impl FnMut(
            RenderTargetIngestStrategy,
        ) -> (
            wgpu::TextureView,
            (u32, u32),
            RenderTargetMetadata,
            Option<EngineFrameKeepalive>,
        ),
    ) -> NativeExternalImportOutcome {
        let imported = match frame.import(ctx, caps) {
            Ok(imported) => imported,
            Err(err) => {
                let fallback_effective =
                    Self::select_deterministic_fallback_effective(requested, fallback_available);
                let (fallback_view, fallback_size, fallback_metadata, fallback_keepalive) =
                    fallback_for(fallback_effective);
                let metadata =
                    Self::with_ingest_strategies(fallback_metadata, requested, fallback_effective);

                if !self.is_registered() {
                    let _ = self.ensure_registered_with_metadata(
                        renderer,
                        fallback_view.clone(),
                        fallback_size,
                        metadata,
                    );
                }

                if let Some(keepalive) = fallback_keepalive {
                    self.push_update_with_metadata_and_keepalive(
                        update,
                        fallback_view,
                        fallback_size,
                        metadata,
                        keepalive,
                    );
                } else {
                    self.push_update_with_metadata(update, fallback_view, fallback_size, metadata);
                }

                return NativeExternalImportOutcome::FellBack {
                    effective: fallback_effective,
                    err,
                };
            }
        };

        let effective = imported.metadata.ingest_strategy;
        let metadata = Self::with_ingest_strategies(imported.metadata, requested, effective);

        if !self.is_registered() {
            let _ = self.ensure_registered_with_metadata(
                renderer,
                imported.view.clone(),
                imported.size,
                metadata,
            );
        }

        self.push_update_with_metadata_and_keepalive(
            update,
            imported.view,
            imported.size,
            metadata,
            imported.keepalive,
        );

        NativeExternalImportOutcome::Imported { effective }
    }

    /// Attempt a native external import, but deterministically fall back to a caller-provided
    /// update when the import cannot be performed.
    ///
    /// Compared to `push_native_external_import_update_with_deterministic_fallback`, this helper
    /// removes boilerplate at call sites by accepting an explicit set of fallback payloads rather
    /// than requiring a separate `fallback_available` slice and a mapping closure.
    ///
    /// Callers MUST provide at least one fallback payload.
    pub fn push_native_external_import_update_with_fallbacks(
        &mut self,
        renderer: &mut Renderer,
        update: &mut EngineFrameUpdate,
        ctx: &fret_render::WgpuContext,
        caps: &fret_render::RendererCapabilities,
        requested: RenderTargetIngestStrategy,
        frame: Box<dyn NativeExternalTextureFrame>,
        mut fallbacks: ImportedViewportFallbacks,
    ) -> NativeExternalImportOutcome {
        let mut fallback_available = Vec::with_capacity(3);
        if fallbacks.owned.is_some() {
            fallback_available.push(RenderTargetIngestStrategy::Owned);
        }
        if fallbacks.gpu_copy.is_some() {
            fallback_available.push(RenderTargetIngestStrategy::GpuCopy);
        }
        if fallbacks.cpu_upload.is_some() {
            fallback_available.push(RenderTargetIngestStrategy::CpuUpload);
        }

        assert!(
            !fallback_available.is_empty(),
            "ImportedViewportFallbacks must provide at least one strategy"
        );

        self.push_native_external_import_update_with_deterministic_fallback(
            renderer,
            update,
            ctx,
            caps,
            requested,
            frame,
            &fallback_available,
            move |fallback_effective| match fallback_effective {
                RenderTargetIngestStrategy::Owned => fallbacks
                    .owned
                    .take()
                    .expect("Owned fallback must be present when marked available")
                    .into_parts(),
                RenderTargetIngestStrategy::GpuCopy => fallbacks
                    .gpu_copy
                    .take()
                    .expect("GpuCopy fallback must be present when marked available")
                    .into_parts(),
                RenderTargetIngestStrategy::CpuUpload => fallbacks
                    .cpu_upload
                    .take()
                    .expect("CpuUpload fallback must be present when marked available")
                    .into_parts(),
                RenderTargetIngestStrategy::Unknown
                | RenderTargetIngestStrategy::ExternalZeroCopy => {
                    panic!("unexpected fallback strategy: {fallback_effective:?}")
                }
            },
        )
    }

    /// Record an unregister request as a runner delta and clear the local id.
    pub fn push_unregister(&mut self, update: &mut EngineFrameUpdate) {
        if !self.is_registered() {
            return;
        }
        update.unregister_render_target(self.id);
        self.id = RenderTargetId::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_fallback_prefers_chain_order() {
        let available = &[
            RenderTargetIngestStrategy::CpuUpload,
            RenderTargetIngestStrategy::GpuCopy,
        ];
        let effective = ImportedViewportRenderTarget::select_deterministic_fallback_effective(
            RenderTargetIngestStrategy::Unknown,
            available,
        );
        assert_eq!(effective, RenderTargetIngestStrategy::GpuCopy);
    }

    #[test]
    fn deterministic_fallback_ignores_non_chain_strategies() {
        let available = &[
            RenderTargetIngestStrategy::Owned,
            RenderTargetIngestStrategy::CpuUpload,
        ];
        let effective = ImportedViewportRenderTarget::select_deterministic_fallback_effective(
            RenderTargetIngestStrategy::GpuCopy,
            available,
        );
        assert_eq!(effective, RenderTargetIngestStrategy::CpuUpload);
    }

    #[test]
    fn deterministic_fallback_handles_empty_available() {
        let effective = ImportedViewportRenderTarget::select_deterministic_fallback_effective(
            RenderTargetIngestStrategy::ExternalZeroCopy,
            &[],
        );
        assert_eq!(effective, RenderTargetIngestStrategy::CpuUpload);
    }

    #[test]
    fn deterministic_fallback_considers_owned_for_external_zero_copy() {
        let available = &[
            RenderTargetIngestStrategy::CpuUpload,
            RenderTargetIngestStrategy::Owned,
        ];
        let effective = ImportedViewportRenderTarget::select_deterministic_fallback_effective(
            RenderTargetIngestStrategy::ExternalZeroCopy,
            available,
        );
        assert_eq!(effective, RenderTargetIngestStrategy::Owned);
    }
}
