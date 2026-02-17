use anyhow::Context as _;
use fret_app::App;
use fret_core::{AppWindowId, Event, KeyCode};
use fret_launch::{EngineFrameUpdate, ImportedViewportRenderTarget};
use fret_render::{
    RenderTargetColorSpace, RenderTargetIngestStrategy, RenderTargetMetadata, Renderer, WgpuContext,
};
use fret_runtime::PlatformCapabilities;
use fret_ui::element::{
    ContainerProps, CrossAlign, Elements, FlexProps, LayoutStyle, Length, MainAlign,
    ViewportSurfaceProps,
};
use fret_ui::{ElementContext, Invalidation, Theme};

fn env_flag_default_false(name: &str) -> bool {
    let Ok(raw) = std::env::var(name) else {
        return false;
    };
    matches!(
        raw.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

#[cfg(target_os = "windows")]
mod wmf {
    use anyhow::Context as _;
    use fret_render::{
        RenderTargetColorEncoding, RenderTargetColorPrimaries, RenderTargetColorRange,
        RenderTargetMatrixCoefficients, RenderTargetTransferFunction,
    };
    use std::ffi::OsStr;
    use std::path::{Path, PathBuf};
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_TEX2D_VPIV, D3D11_TEX2D_VPOV,
        D3D11_TEXTURE2D_DESC, D3D11_USAGE_DEFAULT, D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
        D3D11_VIDEO_PROCESSOR_CONTENT_DESC, D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC,
        D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC, D3D11_VIDEO_PROCESSOR_STREAM,
        D3D11_VIDEO_USAGE_PLAYBACK_NORMAL, D3D11_VPIV_DIMENSION_TEXTURE2D,
        D3D11_VPOV_DIMENSION_TEXTURE2D, ID3D11Device, ID3D11DeviceContext, ID3D11Resource,
        ID3D11Texture2D, ID3D11VideoContext, ID3D11VideoDevice, ID3D11VideoProcessor,
        ID3D11VideoProcessorEnumerator, ID3D11VideoProcessorInputView,
        ID3D11VideoProcessorOutputView,
    };
    use windows::Win32::Graphics::Direct3D11on12::{
        D3D11_RESOURCE_FLAGS, D3D11On12CreateDevice, ID3D11On12Device,
    };
    use windows::Win32::Graphics::Direct3D12::{ID3D12CommandQueue, ID3D12Resource};
    use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_NV12};
    use windows::Win32::Media::MediaFoundation::{
        IMFAttributes, IMFMediaBuffer, IMFSample, IMFSourceReader,
        MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, MF_SOURCE_READER_D3D_MANAGER,
        MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, MF_SOURCE_READER_FIRST_VIDEO_STREAM,
        MF_SOURCE_READERF_ENDOFSTREAM, MF_SOURCE_READERF_NATIVEMEDIATYPECHANGED,
        MF_SOURCE_READERF_STREAMTICK, MF_VERSION, MFCreateAttributes, MFCreateDXGIDeviceManager,
        MFCreateMediaType, MFCreateSourceReaderFromURL, MFMediaType_Video, MFNominalRange,
        MFNominalRange_0_255, MFNominalRange_16_235, MFShutdown, MFStartup, MFVideoFormat_NV12,
        MFVideoFormat_RGB32, MFVideoPrimaries, MFVideoPrimaries_BT709, MFVideoPrimaries_BT2020,
        MFVideoPrimaries_DCI_P3, MFVideoTransFunc_709, MFVideoTransFunc_2084, MFVideoTransFunc_HLG,
        MFVideoTransFunc_Unknown, MFVideoTransFunc_sRGB, MFVideoTransferFunction,
        MFVideoTransferMatrix, MFVideoTransferMatrix_BT601, MFVideoTransferMatrix_BT709,
        MFVideoTransferMatrix_BT2020_10, MFVideoTransferMatrix_Unknown,
    };
    use windows::Win32::Media::MediaFoundation::{IMFDXGIBuffer, IMFDXGIDeviceManager};
    use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};
    use windows::core::BOOL;
    use windows::core::HSTRING;
    use windows::core::Interface as _;

    const VIDEO_FILE_EXTS: &[&str] = &["mp4", "m4v", "mov", "wmv", "avi"];

    pub struct VideoFrame {
        pub size: (u32, u32),
        pub bytes_per_row: u32,
        pub bgra8: Vec<u8>,
    }

    pub struct MfVideoReader {
        path: String,
        reader: IMFSourceReader,
        size: (u32, u32),
        bytes_per_row: u32,
        color_encoding: RenderTargetColorEncoding,
    }

    impl MfVideoReader {
        pub fn new(path: impl Into<String>) -> anyhow::Result<Self> {
            let raw_path = path.into();
            let path = resolve_source_reader_url(&raw_path)?;

            unsafe {
                // Best-effort: if COM is already initialized in another mode, keep going.
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                MFStartup(MF_VERSION, 0).context("MFStartup")?;
            }

            let mut attributes: Option<IMFAttributes> = None;
            unsafe {
                MFCreateAttributes(&mut attributes, 1).context("MFCreateAttributes")?;
            }
            let attributes = attributes.context("MFCreateAttributes returned None")?;
            unsafe {
                attributes
                    .SetUINT32(&MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING, 1)
                    .context("SetUINT32(MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING)")?;
            }

            let hpath = HSTRING::from(path.as_str());
            let reader = unsafe {
                MFCreateSourceReaderFromURL(&hpath, Some(&attributes))
                    .with_context(|| format!("MFCreateSourceReaderFromURL({path})"))?
            };

            // Request a predictable software-decoded output format first: RGB32 (little-endian BGRA).
            // This keeps stage M2A focused on contract/metadata/gates; a D3D12-only fast path is M2B.
            let media_type = unsafe { MFCreateMediaType().context("MFCreateMediaType")? };
            unsafe {
                media_type
                    .SetGUID(
                        &windows::Win32::Media::MediaFoundation::MF_MT_MAJOR_TYPE,
                        &MFMediaType_Video,
                    )
                    .context("SetGUID major")?;
                media_type
                    .SetGUID(
                        &windows::Win32::Media::MediaFoundation::MF_MT_SUBTYPE,
                        &MFVideoFormat_RGB32,
                    )
                    .context("SetGUID subtype")?;
                reader
                    .SetCurrentMediaType(
                        MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                        None,
                        &media_type,
                    )
                    .context("SetCurrentMediaType")?;
            }

            let (size, bytes_per_row) = Self::query_video_layout(&reader)?;
            let color_encoding = Self::query_color_encoding(&reader)?;

            Ok(Self {
                path,
                reader,
                size,
                bytes_per_row,
                color_encoding,
            })
        }

        pub fn color_encoding(&self) -> RenderTargetColorEncoding {
            self.color_encoding
        }

        fn query_video_layout(reader: &IMFSourceReader) -> anyhow::Result<((u32, u32), u32)> {
            let media_type = unsafe {
                reader
                    .GetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
                    .context("GetCurrentMediaType")?
            };

            let frame_size = unsafe {
                media_type
                    .GetUINT64(&windows::Win32::Media::MediaFoundation::MF_MT_FRAME_SIZE)
                    .context("GetUINT64(MF_MT_FRAME_SIZE)")?
            };
            let w = (frame_size >> 32) as u32;
            let h = (frame_size & 0xffff_ffff) as u32;

            // Default stride can be absent; for RGB32 it's usually width*4.
            let bytes_per_row = unsafe {
                media_type
                    .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_DEFAULT_STRIDE)
                    .unwrap_or(w.saturating_mul(4))
            };

            Ok(((w.max(1), h.max(1)), bytes_per_row.max(w.saturating_mul(4))))
        }

        #[allow(non_upper_case_globals)]
        fn query_color_encoding(
            reader: &IMFSourceReader,
        ) -> anyhow::Result<RenderTargetColorEncoding> {
            let media_type = unsafe {
                reader
                    .GetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
                    .context("GetCurrentMediaType")?
            };

            let primaries = unsafe {
                media_type
                    .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_VIDEO_PRIMARIES)
                    .ok()
                    .map(|v| MFVideoPrimaries(v as i32))
            };
            let transfer = unsafe {
                media_type
                    .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_TRANSFER_FUNCTION)
                    .ok()
                    .map(|v| MFVideoTransferFunction(v as i32))
            };
            let matrix = unsafe {
                media_type
                    .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_YUV_MATRIX)
                    .ok()
                    .map(|v| MFVideoTransferMatrix(v as i32))
            };
            let range = unsafe {
                media_type
                    .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_VIDEO_NOMINAL_RANGE)
                    .ok()
                    .map(|v| MFNominalRange(v as i32))
            };

            Ok(RenderTargetColorEncoding {
                primaries: match primaries {
                    Some(MFVideoPrimaries_BT709) => RenderTargetColorPrimaries::Bt709,
                    Some(MFVideoPrimaries_DCI_P3) => RenderTargetColorPrimaries::DisplayP3,
                    Some(MFVideoPrimaries_BT2020) => RenderTargetColorPrimaries::Bt2020,
                    _ => RenderTargetColorPrimaries::Unknown,
                },
                transfer: match transfer {
                    Some(MFVideoTransFunc_sRGB) => RenderTargetTransferFunction::Srgb,
                    Some(MFVideoTransFunc_709) => RenderTargetTransferFunction::Bt1886,
                    Some(MFVideoTransFunc_2084) => RenderTargetTransferFunction::Pq,
                    Some(MFVideoTransFunc_HLG) => RenderTargetTransferFunction::Hlg,
                    Some(MFVideoTransFunc_Unknown) | None => RenderTargetTransferFunction::Unknown,
                    _ => RenderTargetTransferFunction::Unknown,
                },
                matrix: match matrix {
                    Some(MFVideoTransferMatrix_BT601) => RenderTargetMatrixCoefficients::Bt601,
                    Some(MFVideoTransferMatrix_BT709) => RenderTargetMatrixCoefficients::Bt709,
                    Some(MFVideoTransferMatrix_BT2020_10) => {
                        RenderTargetMatrixCoefficients::Bt2020Ncl
                    }
                    // Our stage M2A output is RGB32, so a missing/unknown YUV matrix is best treated
                    // as identity for now.
                    Some(MFVideoTransferMatrix_Unknown) | None => {
                        RenderTargetMatrixCoefficients::Rgb
                    }
                    _ => RenderTargetMatrixCoefficients::Unknown,
                },
                range: match range {
                    Some(MFNominalRange_0_255) => RenderTargetColorRange::Full,
                    Some(MFNominalRange_16_235) => RenderTargetColorRange::Limited,
                    _ => RenderTargetColorRange::Unknown,
                },
            })
        }

        fn reset(&mut self) -> anyhow::Result<()> {
            // The simplest deterministic reset is to recreate the reader.
            // Stage M2B can replace this with a true seek once the path is stable.
            *self = Self::new(self.path.clone())?;
            Ok(())
        }

        pub fn read_next(&mut self) -> anyhow::Result<Option<VideoFrame>> {
            loop {
                let mut actual_stream_index = 0u32;
                let mut flags = 0u32;
                let mut timestamp = 0i64;
                let mut sample: Option<IMFSample> = None;

                unsafe {
                    self.reader
                        .ReadSample(
                            MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                            0,
                            Some(&mut actual_stream_index),
                            Some(&mut flags),
                            Some(&mut timestamp),
                            Some(&mut sample),
                        )
                        .context("ReadSample")?;
                }

                if (flags & (MF_SOURCE_READERF_STREAMTICK.0 as u32)) != 0 {
                    continue;
                }

                if (flags & (MF_SOURCE_READERF_NATIVEMEDIATYPECHANGED.0 as u32)) != 0 {
                    let (size, bytes_per_row) = Self::query_video_layout(&self.reader)?;
                    self.size = size;
                    self.bytes_per_row = bytes_per_row;
                    self.color_encoding = Self::query_color_encoding(&self.reader)?;
                }

                if (flags & (MF_SOURCE_READERF_ENDOFSTREAM.0 as u32)) != 0 {
                    self.reset()?;
                    continue;
                }

                let Some(sample) = sample else {
                    return Ok(None);
                };

                let buffer = unsafe {
                    sample
                        .ConvertToContiguousBuffer()
                        .context("ConvertToContiguousBuffer")?
                };
                let mut bytes = lock_and_copy(&buffer)?;

                // Media Foundation's RGB32 output is "opaque video" in most cases, but some
                // decoders may leave alpha at 0. Normalize to opaque so the demo is visible.
                for px in bytes.chunks_exact_mut(4) {
                    px[3] = 0xff;
                }

                let (w, h) = self.size;
                let row_bytes = w.saturating_mul(4);
                let effective_bytes_per_row = if h > 0
                    && bytes.len() % (h as usize) == 0
                    && (bytes.len() / (h as usize)) >= (row_bytes as usize)
                {
                    (bytes.len() / (h as usize)) as u32
                } else {
                    self.bytes_per_row.max(row_bytes)
                };
                return Ok(Some(VideoFrame {
                    size: self.size,
                    bytes_per_row: effective_bytes_per_row,
                    bgra8: bytes,
                }));
            }
        }
    }

    pub struct Dx12Interop {
        on12: ID3D11On12Device,
        video_device: ID3D11VideoDevice,
        video_context: ID3D11VideoContext,
        context: ID3D11DeviceContext,
        mf_reader: IMFSourceReader,
        size: (u32, u32),
        color_encoding: RenderTargetColorEncoding,
        tmp_bgra: Option<ID3D11Texture2D>,
        tmp_bgra_size: (u32, u32),
        vp_enum: Option<ID3D11VideoProcessorEnumerator>,
        vp: Option<ID3D11VideoProcessor>,
    }

    impl Dx12Interop {
        pub fn new(
            d3d12_queue: &ID3D12CommandQueue,
            d3d12_resource: &ID3D12Resource,
            path: &str,
        ) -> anyhow::Result<Self> {
            unsafe {
                // Best-effort: if COM is already initialized in another mode, keep going.
                let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
                MFStartup(MF_VERSION, 0).context("MFStartup")?;
            }

            // Resolve the D3D12 device from the resource; the interop device must match.
            let mut d3d12_device: Option<windows::Win32::Graphics::Direct3D12::ID3D12Device> = None;
            unsafe {
                d3d12_resource
                    .GetDevice(&mut d3d12_device)
                    .context("ID3D12Resource::GetDevice")?;
            }
            let d3d12_device = d3d12_device.context("ID3D12Resource::GetDevice returned None")?;

            // Build a D3D11On12 device on top of the runner's D3D12 queue. We pass this D3D11
            // device to Media Foundation so decoded frames arrive as DXGI-backed buffers.
            let mut d3d11: Option<ID3D11Device> = None;
            let mut context: Option<ID3D11DeviceContext> = None;
            let queue_unk = d3d12_queue.cast::<windows::core::IUnknown>()?;
            unsafe {
                D3D11On12CreateDevice(
                    &d3d12_device,
                    (D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_VIDEO_SUPPORT).0,
                    None,
                    Some(&[Some(queue_unk)]),
                    0,
                    Some(&mut d3d11),
                    Some(&mut context),
                    None,
                )
                .context("D3D11On12CreateDevice")?;
            }
            let d3d11 = d3d11.context("D3D11On12CreateDevice returned None device")?;
            let context = context.context("D3D11On12CreateDevice returned None context")?;
            let on12: ID3D11On12Device = d3d11.cast().context("cast to ID3D11On12Device")?;
            let video_device: ID3D11VideoDevice =
                d3d11.cast().context("cast to ID3D11VideoDevice")?;
            let video_context: ID3D11VideoContext =
                context.cast().context("cast to ID3D11VideoContext")?;

            // Create an MF DXGI device manager backed by our D3D11On12 device.
            let mut reset_token = 0u32;
            let mut device_manager: Option<IMFDXGIDeviceManager> = None;
            unsafe {
                MFCreateDXGIDeviceManager(&mut reset_token, &mut device_manager)
                    .context("MFCreateDXGIDeviceManager")?;
            }
            let device_manager =
                device_manager.context("MFCreateDXGIDeviceManager returned None")?;
            unsafe {
                device_manager
                    .ResetDevice(&d3d11, reset_token)
                    .context("IMFDXGIDeviceManager::ResetDevice")?;
            }

            let resolved_path = resolve_source_reader_url(path)?;
            let hpath = HSTRING::from(resolved_path.as_str());

            let mut attributes: Option<IMFAttributes> = None;
            unsafe {
                MFCreateAttributes(&mut attributes, 2).context("MFCreateAttributes")?;
            }
            let attributes = attributes.context("MFCreateAttributes returned None")?;
            unsafe {
                attributes
                    .SetUnknown(&MF_SOURCE_READER_D3D_MANAGER, &device_manager)
                    .context("SetUnknown(MF_SOURCE_READER_D3D_MANAGER)")?;
            }

            fn try_create_source_reader(
                hpath: &HSTRING,
                attributes: &IMFAttributes,
                resolved_path: &str,
                enable_hw_transforms: u32,
                enable_video_processing: u32,
            ) -> anyhow::Result<IMFSourceReader> {
                unsafe {
                    attributes
                        .SetUINT32(
                            &MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS,
                            enable_hw_transforms,
                        )
                        .context("SetUINT32(MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS)")?;
                    attributes
                        .SetUINT32(
                            &MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING,
                            enable_video_processing,
                        )
                        .context("SetUINT32(MF_SOURCE_READER_ENABLE_VIDEO_PROCESSING)")?;
                }
                unsafe {
                    MFCreateSourceReaderFromURL(hpath, Some(attributes)).with_context(|| {
                        format!(
                            "MFCreateSourceReaderFromURL({resolved_path}) hw_transforms={enable_hw_transforms} video_processing={enable_video_processing}"
                        )
                    })
                }
            }

            // Prefer enabling video processing so MF can deliver RGB32 frames without forcing us to
            // run an explicit NV12->BGRA conversion path in the demo. If this fails on a given
            // machine/codec, fall back to the conservative configuration.
            let reader = match try_create_source_reader(&hpath, &attributes, &resolved_path, 1, 1) {
                Ok(v) => v,
                Err(err) => {
                    tracing::warn!(
                        ?err,
                        "MF DX12 interop: SourceReader init failed with video processing enabled; retrying with conservative flags"
                    );
                    try_create_source_reader(&hpath, &attributes, &resolved_path, 0, 0)?
                }
            };

            // Prefer RGB32 output so the `read_next_dxgi_texture` path can hand us a BGRA DXGI
            // texture without additional per-frame conversion work in this demo.
            let media_type = unsafe { MFCreateMediaType().context("MFCreateMediaType")? };
            unsafe {
                media_type
                    .SetGUID(
                        &windows::Win32::Media::MediaFoundation::MF_MT_MAJOR_TYPE,
                        &MFMediaType_Video,
                    )
                    .context("SetGUID major")?;
                media_type
                    .SetGUID(
                        &windows::Win32::Media::MediaFoundation::MF_MT_SUBTYPE,
                        &MFVideoFormat_RGB32,
                    )
                    .context("SetGUID subtype")?;
                reader
                    .SetCurrentMediaType(
                        MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                        None,
                        &media_type,
                    )
                    .context("SetCurrentMediaType(RGB32)")?;
            }

            let (size, _bytes_per_row) = MfVideoReader::query_video_layout(&reader)?;
            let color_encoding = MfVideoReader::query_color_encoding(&reader)?;

            Ok(Self {
                on12,
                video_device,
                video_context,
                context,
                mf_reader: reader,
                size,
                color_encoding,
                tmp_bgra: None,
                tmp_bgra_size: (0, 0),
                vp_enum: None,
                vp: None,
            })
        }

        pub fn color_encoding(&self) -> RenderTargetColorEncoding {
            self.color_encoding
        }

        pub fn size(&self) -> (u32, u32) {
            self.size
        }

        pub fn read_next_dxgi_texture(&mut self) -> anyhow::Result<Option<ID3D11Texture2D>> {
            loop {
                let mut actual_stream_index = 0u32;
                let mut flags = 0u32;
                let mut timestamp = 0i64;
                let mut sample: Option<IMFSample> = None;

                unsafe {
                    self.mf_reader
                        .ReadSample(
                            MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                            0,
                            Some(&mut actual_stream_index),
                            Some(&mut flags),
                            Some(&mut timestamp),
                            Some(&mut sample),
                        )
                        .context("ReadSample")?;
                }

                if (flags & (MF_SOURCE_READERF_STREAMTICK.0 as u32)) != 0 {
                    continue;
                }

                if (flags & (MF_SOURCE_READERF_NATIVEMEDIATYPECHANGED.0 as u32)) != 0 {
                    let (size, _bytes_per_row) =
                        MfVideoReader::query_video_layout(&self.mf_reader)?;
                    self.size = size;
                    self.color_encoding = MfVideoReader::query_color_encoding(&self.mf_reader)?;
                }

                if (flags & (MF_SOURCE_READERF_ENDOFSTREAM.0 as u32)) != 0 {
                    // Deterministic reset: recreate the reader/interop state on next init.
                    return Ok(None);
                }

                let Some(sample) = sample else {
                    continue;
                };

                let buffer = unsafe { sample.GetBufferByIndex(0) }.context("GetBufferByIndex")?;

                let dxgi: IMFDXGIBuffer = match buffer.cast() {
                    Ok(v) => v,
                    Err(_) => {
                        anyhow::bail!("MF sample did not provide an IMFDXGIBuffer");
                    }
                };

                let mut tex: Option<ID3D11Texture2D> = None;
                unsafe {
                    dxgi.GetResource(
                        &<ID3D11Texture2D as windows::core::Interface>::IID,
                        &mut tex as *mut _ as _,
                    )
                    .context("IMFDXGIBuffer::GetResource")?;
                    let _subresource = dxgi
                        .GetSubresourceIndex()
                        .context("IMFDXGIBuffer::GetSubresourceIndex")?;
                }
                return Ok(tex);
            }
        }

        pub fn copy_into_dx12_shared_allocation(
            &mut self,
            dst_resource: &ID3D12Resource,
            src_texture: &ID3D11Texture2D,
        ) -> anyhow::Result<()> {
            let mut desc = D3D11_TEXTURE2D_DESC::default();
            unsafe {
                src_texture.GetDesc(&mut desc);
            }

            if desc.Format == DXGI_FORMAT_NV12 {
                self.convert_nv12_into_bgra_tmp(src_texture, desc.Width, desc.Height)?;
                let tmp = self
                    .tmp_bgra
                    .clone()
                    .context("NV12 conversion produced no temporary BGRA texture")?;
                return self.copy_resource_into_dx12_shared_allocation(dst_resource, &tmp);
            }

            self.copy_resource_into_dx12_shared_allocation(dst_resource, src_texture)
        }

        fn copy_resource_into_dx12_shared_allocation(
            &mut self,
            dst_resource: &ID3D12Resource,
            src_texture: &ID3D11Texture2D,
        ) -> anyhow::Result<()> {
            // Wrap the destination D3D12 resource for D3D11On12 CopyResource.
            let flags11 = D3D11_RESOURCE_FLAGS {
                BindFlags: D3D11_BIND_SHADER_RESOURCE.0 as u32,
                ..Default::default()
            };
            let mut wrapped: Option<ID3D11Resource> = None;
            unsafe {
                self.on12
                    .CreateWrappedResource(
                        dst_resource,
                        &flags11,
                        windows::Win32::Graphics::Direct3D12::D3D12_RESOURCE_STATE_COPY_DEST,
                        windows::Win32::Graphics::Direct3D12::D3D12_RESOURCE_STATE_COPY_DEST,
                        &mut wrapped,
                    )
                    .context("CreateWrappedResource")?;
            }
            let wrapped = wrapped.context("CreateWrappedResource returned None")?;
            let src: ID3D11Resource = src_texture.cast().context("cast src to ID3D11Resource")?;

            unsafe {
                self.on12.AcquireWrappedResources(&[Some(wrapped.clone())]);
                self.context.CopyResource(&wrapped, &src);
                self.on12.ReleaseWrappedResources(&[Some(wrapped)]);
                self.context.Flush();
            }
            Ok(())
        }

        fn convert_nv12_into_bgra_tmp(
            &mut self,
            src_texture: &ID3D11Texture2D,
            width: u32,
            height: u32,
        ) -> anyhow::Result<()> {
            if self.tmp_bgra.is_none() || self.tmp_bgra_size != (width, height) {
                let desc = D3D11_TEXTURE2D_DESC {
                    Width: width,
                    Height: height,
                    MipLevels: 1,
                    ArraySize: 1,
                    Format: DXGI_FORMAT_B8G8R8A8_UNORM,
                    SampleDesc: windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC {
                        Count: 1,
                        Quality: 0,
                    },
                    Usage: D3D11_USAGE_DEFAULT,
                    BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
                    CPUAccessFlags: 0,
                    MiscFlags: 0,
                };

                let d3d11: ID3D11Device = self
                    .on12
                    .cast()
                    .context("cast ID3D11On12Device to ID3D11Device")?;
                let mut tmp: Option<ID3D11Texture2D> = None;
                unsafe {
                    d3d11
                        .CreateTexture2D(&desc, None, Some(&mut tmp))
                        .context("CreateTexture2D(tmp_bgra)")?;
                }
                self.tmp_bgra = tmp;
                self.tmp_bgra_size = (width, height);
                self.vp_enum = None;
                self.vp = None;
            }

            if self.vp_enum.is_none() || self.vp.is_none() {
                let content = D3D11_VIDEO_PROCESSOR_CONTENT_DESC {
                    InputFrameFormat: D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
                    InputFrameRate: windows::Win32::Graphics::Dxgi::Common::DXGI_RATIONAL {
                        Numerator: 60,
                        Denominator: 1,
                    },
                    InputWidth: width,
                    InputHeight: height,
                    OutputFrameRate: windows::Win32::Graphics::Dxgi::Common::DXGI_RATIONAL {
                        Numerator: 60,
                        Denominator: 1,
                    },
                    OutputWidth: width,
                    OutputHeight: height,
                    Usage: D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
                };
                let en = unsafe {
                    self.video_device
                        .CreateVideoProcessorEnumerator(&content)
                        .context("CreateVideoProcessorEnumerator")?
                };
                let vp = unsafe {
                    self.video_device
                        .CreateVideoProcessor(&en, 0)
                        .context("CreateVideoProcessor")?
                };
                self.vp_enum = Some(en);
                self.vp = Some(vp);
            }

            let en = self
                .vp_enum
                .as_ref()
                .context("video processor enumerator missing")?;
            let vp = self.vp.as_ref().context("video processor missing")?;
            let tmp = self.tmp_bgra.as_ref().context("tmp_bgra missing")?.clone();

            let mut input_desc: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC =
                unsafe { std::mem::zeroed() };
            input_desc.FourCC = 0;
            input_desc.ViewDimension = D3D11_VPIV_DIMENSION_TEXTURE2D;
            unsafe {
                input_desc.Anonymous.Texture2D = D3D11_TEX2D_VPIV {
                    MipSlice: 0,
                    ArraySlice: 0,
                };
            }
            let mut input_view: Option<ID3D11VideoProcessorInputView> = None;
            unsafe {
                self.video_device
                    .CreateVideoProcessorInputView(
                        src_texture,
                        en,
                        &input_desc,
                        Some(&mut input_view),
                    )
                    .context("CreateVideoProcessorInputView")?;
            }
            let input_view = input_view.context("CreateVideoProcessorInputView returned None")?;

            let mut output_desc: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC =
                unsafe { std::mem::zeroed() };
            output_desc.ViewDimension = D3D11_VPOV_DIMENSION_TEXTURE2D;
            unsafe {
                output_desc.Anonymous.Texture2D = D3D11_TEX2D_VPOV { MipSlice: 0 };
            }
            let mut output_view: Option<ID3D11VideoProcessorOutputView> = None;
            unsafe {
                self.video_device
                    .CreateVideoProcessorOutputView(&tmp, en, &output_desc, Some(&mut output_view))
                    .context("CreateVideoProcessorOutputView")?;
            }
            let output_view =
                output_view.context("CreateVideoProcessorOutputView returned None")?;

            let rect = windows::Win32::Foundation::RECT {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            };
            unsafe {
                self.video_context.VideoProcessorSetStreamFrameFormat(
                    vp,
                    0,
                    D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE,
                );
                self.video_context
                    .VideoProcessorSetStreamSourceRect(vp, 0, true, Some(&rect));
                self.video_context
                    .VideoProcessorSetStreamDestRect(vp, 0, true, Some(&rect));
                self.video_context
                    .VideoProcessorSetOutputTargetRect(vp, true, Some(&rect));
            }

            let mut stream = D3D11_VIDEO_PROCESSOR_STREAM::default();
            stream.Enable = BOOL(1);
            stream.OutputIndex = 0;
            stream.InputFrameOrField = 0;
            stream.PastFrames = 0;
            stream.FutureFrames = 0;
            stream.ppPastSurfaces = std::ptr::null_mut();
            stream.pInputSurface = std::mem::ManuallyDrop::new(Some(input_view));
            stream.ppFutureSurfaces = std::ptr::null_mut();
            stream.ppPastSurfacesRight = std::ptr::null_mut();
            stream.pInputSurfaceRight = std::mem::ManuallyDrop::new(None);
            stream.ppFutureSurfacesRight = std::ptr::null_mut();

            unsafe {
                self.video_context
                    .VideoProcessorBlt(vp, &output_view, 0, std::slice::from_ref(&stream))
                    .context("VideoProcessorBlt")?;
                self.context.Flush();
            }

            Ok(())
        }
    }

    impl Drop for MfVideoReader {
        fn drop(&mut self) {
            unsafe {
                let _ = MFShutdown();
            }
        }
    }

    fn lock_and_copy(buffer: &IMFMediaBuffer) -> anyhow::Result<Vec<u8>> {
        let mut ptr = std::ptr::null_mut::<u8>();
        let mut max_len = 0u32;
        let mut cur_len = 0u32;
        unsafe {
            buffer
                .Lock(&mut ptr, Some(&mut max_len), Some(&mut cur_len))
                .context("IMFMediaBuffer::Lock")?;
        }
        let len = cur_len.min(max_len) as usize;
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) }.to_vec();
        unsafe {
            buffer.Unlock().ok().context("IMFMediaBuffer::Unlock")?;
        }
        Ok(bytes)
    }

    fn resolve_source_reader_url(raw: &str) -> anyhow::Result<String> {
        let raw = raw.trim();
        anyhow::ensure!(!raw.is_empty(), "empty Media Foundation source URL/path");

        // Allow callers to pass a real URL (including `file://...`) without trying to normalize it.
        if raw.contains("://") {
            tracing::info!(raw, "using Media Foundation source URL");
            return Ok(raw.to_string());
        }

        let candidate = PathBuf::from(raw);
        let metadata = std::fs::metadata(&candidate)
            .with_context(|| format!("source path does not exist: {raw}"))?;

        let resolved = if metadata.is_dir() {
            let picked = pick_first_video_file_from_dir(&candidate)?;
            tracing::info!(
                raw_dir = %candidate.display(),
                picked = %picked.display(),
                "FRET_MF_VIDEO_PATH points to a directory; picked a video file"
            );
            picked
        } else {
            candidate
        };

        let resolved = std::fs::canonicalize(&resolved).with_context(|| {
            format!("failed to canonicalize source path: {}", resolved.display())
        })?;

        let url = path_to_file_url(&resolved);
        tracing::info!(url, "using Media Foundation source file URL");
        Ok(url)
    }

    fn pick_first_video_file_from_dir(dir: &Path) -> anyhow::Result<PathBuf> {
        let mut candidates = Vec::<PathBuf>::new();
        for entry in std::fs::read_dir(dir)
            .with_context(|| format!("failed to read directory: {}", dir.display()))?
        {
            let entry = entry.context("read_dir entry")?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let ext = path
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or_default()
                .to_ascii_lowercase();
            if VIDEO_FILE_EXTS.contains(&ext.as_str()) {
                candidates.push(path);
            }
        }

        candidates.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        candidates.into_iter().next().with_context(|| {
            format!(
                "no supported video files found in directory: {} (expected one of: {})",
                dir.display(),
                VIDEO_FILE_EXTS.join(", ")
            )
        })
    }

    fn path_to_file_url(path: &Path) -> String {
        let mut p = path.to_string_lossy().to_string();
        if let Some(stripped) = p.strip_prefix(r"\\?\") {
            p = stripped.to_string();
        }
        let mut p = p.replace('\\', "/");

        // Windows drive path: `C:\foo\bar` -> `file:///C:/foo/bar`.
        if p.len() >= 2 && p.as_bytes()[1] == b':' {
            p.insert(0, '/');
            return format!("file://{}", percent_encode_url_path(&p));
        }

        // Best-effort fallback: treat it as an already-normalized absolute path.
        // (UNC paths are not supported by this helper yet.)
        format!("file://{}", percent_encode_url_path(&p))
    }

    fn percent_encode_url_path(path: &str) -> String {
        let mut out = String::with_capacity(path.len());
        for b in path.as_bytes() {
            let c = *b as char;
            let keep = c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '~' | '/' | ':');
            if keep {
                out.push(c);
            } else {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
        out
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn resolve_source_reader_url_picks_first_video_file_from_dir() {
            let root = std::env::temp_dir().join(format!(
                "fret_mf_video_path_test_{}_{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            ));
            std::fs::create_dir_all(&root).unwrap();

            std::fs::write(root.join("z.txt"), b"not video").unwrap();
            std::fs::write(root.join("b.mp4"), b"dummy").unwrap();
            std::fs::write(root.join("a.mp4"), b"dummy").unwrap();

            let resolved = resolve_source_reader_url(root.to_str().unwrap()).unwrap();
            assert!(resolved.to_ascii_lowercase().ends_with("/a.mp4"));

            let _ = std::fs::remove_dir_all(&root);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExternalVideoImportsMode {
    CheckerGpu,
    #[cfg(target_os = "windows")]
    MfVideoCpuUpload,
    #[cfg(target_os = "windows")]
    MfVideoDx12GpuCopy,
}

struct ExternalVideoImportsState {
    show: fret_runtime::Model<bool>,
    mode: ExternalVideoImportsMode,

    target: ImportedViewportRenderTarget,
    target_px_size: (u32, u32),
    desired_target_px_size: (u32, u32),
    texture: Option<wgpu::Texture>,

    #[cfg(target_os = "windows")]
    mf: Option<wmf::MfVideoReader>,
    #[cfg(target_os = "windows")]
    mf_dx12: Option<wmf::Dx12Interop>,
}

fn init_window(app: &mut App, _window: AppWindowId) -> ExternalVideoImportsState {
    ExternalVideoImportsState {
        show: app.models_mut().insert(true),
        // Use BGRA to align with Media Foundation's RGB32 output (little-endian BGRA).
        target: ImportedViewportRenderTarget::new(
            wgpu::TextureFormat::Bgra8UnormSrgb,
            RenderTargetColorSpace::Srgb,
        ),
        target_px_size: (1, 1),
        desired_target_px_size: (1280, 720),
        texture: None,
        mode: ExternalVideoImportsMode::CheckerGpu,
        #[cfg(target_os = "windows")]
        mf: None,
        #[cfg(target_os = "windows")]
        mf_dx12: None,
    }
}

fn on_event(
    app: &mut App,
    _services: &mut dyn fret_core::UiServices,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut ExternalVideoImportsState,
    event: &Event,
) {
    if let Event::KeyDown { key, .. } = event
        && *key == KeyCode::KeyV
    {
        let _ = app.models_mut().update(&st.show, |v| *v = !*v);
        app.request_redraw(window);
    }

    if let Event::KeyDown { key, .. } = event
        && *key == KeyCode::KeyI
    {
        st.mode = match st.mode {
            ExternalVideoImportsMode::CheckerGpu => {
                #[cfg(target_os = "windows")]
                {
                    ExternalVideoImportsMode::MfVideoCpuUpload
                }
                #[cfg(not(target_os = "windows"))]
                {
                    ExternalVideoImportsMode::CheckerGpu
                }
            }
            #[cfg(target_os = "windows")]
            ExternalVideoImportsMode::MfVideoCpuUpload => {
                if env_flag_default_false("FRET_EXTV2_MF_DX12_GPU_COPY") {
                    ExternalVideoImportsMode::MfVideoDx12GpuCopy
                } else {
                    ExternalVideoImportsMode::CheckerGpu
                }
            }
            #[cfg(target_os = "windows")]
            ExternalVideoImportsMode::MfVideoDx12GpuCopy => ExternalVideoImportsMode::CheckerGpu,
        };
        app.request_redraw(window);
    }
}

fn view(cx: &mut ElementContext<'_, App>, st: &mut ExternalVideoImportsState) -> Elements {
    cx.observe_model(&st.show, Invalidation::Layout);

    let scale_factor = cx.environment_scale_factor(Invalidation::Layout);
    let w_px = (cx.bounds.size.width.0.max(1.0) * scale_factor).round() as u32;
    let h_px = (cx.bounds.size.height.0.max(1.0) * scale_factor).round() as u32;
    st.desired_target_px_size = (w_px.max(1).min(4096), h_px.max(1).min(4096));

    let show = cx.app.models().read(&st.show, |v| *v).unwrap_or(true);

    let theme = Theme::global(&*cx.app).snapshot();

    let mut fill = LayoutStyle::default();
    fill.size.width = Length::Fill;
    fill.size.height = Length::Fill;

    let mut panel_layout = LayoutStyle::default();
    panel_layout.size.width = Length::Px(fret_core::Px(980.0));
    panel_layout.size.height = Length::Px(fret_core::Px(720.0));

    let mut row = FlexProps {
        layout: fill,
        direction: fret_core::Axis::Horizontal,
        gap: fret_core::Px(12.0),
        padding: fret_core::Edges::all(fret_core::Px(16.0)),
        justify: MainAlign::Start,
        align: CrossAlign::Start,
        wrap: false,
    };
    row.layout.size.width = Length::Fill;
    row.layout.size.height = Length::Fill;

    let target = st.target.id();
    let target_px_size = st.target_px_size;

    vec![
        cx.container(
            ContainerProps {
                layout: fill,
                background: Some(theme.color_token("background")),
                ..Default::default()
            },
            |cx| {
                vec![cx.flex(row, |cx| {
                    vec![
                        cx.container(
                            ContainerProps {
                                layout: panel_layout,
                                border: fret_core::Edges::all(fret_core::Px(1.0)),
                                border_paint: Some(fret_core::scene::Paint::Solid(
                                    theme.color_token("border"),
                                )),
                                background: Some(theme.color_token("muted")),
                                corner_radii: fret_core::Corners::all(fret_core::Px(10.0)),
                                ..Default::default()
                            },
                            |cx| {
                                let mut layout = LayoutStyle::default();
                                layout.size.width = Length::Fill;
                                layout.size.height = Length::Fill;
                                vec![
                                    cx.viewport_surface_props(ViewportSurfaceProps {
                                        layout,
                                        target,
                                        target_px_size,
                                        fit: fret_core::ViewportFit::Contain,
                                        opacity: if show { 1.0 } else { 0.0 },
                                    })
                                    .test_id("external-video-imports-mf-surface"),
                                ]
                            },
                        )
                        .test_id("external-video-imports-mf-root"),
                    ]
                })]
            },
        )
        .test_id("external-video-imports-mf-app"),
    ]
    .into()
}

fn record_engine_frame(
    app: &mut App,
    window: AppWindowId,
    _ui: &mut fret_ui::UiTree<App>,
    st: &mut ExternalVideoImportsState,
    context: &WgpuContext,
    renderer: &mut Renderer,
    _scale_factor: f32,
    _tick_id: fret_runtime::TickId,
    frame_id: fret_runtime::FrameId,
) -> EngineFrameUpdate {
    let show = app.models().read(&st.show, |v| *v).unwrap_or(true);
    let mut update = EngineFrameUpdate::default();

    if !show {
        st.target.push_unregister(&mut update);
        st.texture = None;
        st.target_px_size = (1, 1);
        return update;
    }

    #[cfg(target_os = "windows")]
    {
        if st.mode == ExternalVideoImportsMode::MfVideoCpuUpload && st.mf.is_none() {
            if let Ok(path) = std::env::var("FRET_MF_VIDEO_PATH")
                && !path.trim().is_empty()
            {
                match wmf::MfVideoReader::new(path) {
                    Ok(reader) => st.mf = Some(reader),
                    Err(err) => {
                        tracing::warn!(?err, "failed to initialize MF reader; falling back");
                        st.mode = ExternalVideoImportsMode::CheckerGpu;
                    }
                }
            } else {
                tracing::info!("FRET_MF_VIDEO_PATH is not set; staying on checker mode");
                st.mode = ExternalVideoImportsMode::CheckerGpu;
            }
        }
    }

    // Decide target size:
    // - checker mode: follow viewport size (stress the contract-path update shapes),
    // - MF mode: follow decoded frame size (avoid per-frame rescale churn in M2A).
    let mut decoded_frame: Option<((u32, u32), u32, Vec<u8>)> = None;
    let desired = match st.mode {
        ExternalVideoImportsMode::CheckerGpu => st.desired_target_px_size,
        #[cfg(target_os = "windows")]
        ExternalVideoImportsMode::MfVideoCpuUpload => {
            if let Some(reader) = st.mf.as_mut() {
                if let Some(frame) = reader.read_next().ok().flatten() {
                    decoded_frame = Some((frame.size, frame.bytes_per_row, frame.bgra8));
                    frame.size
                } else {
                    st.desired_target_px_size
                }
            } else {
                st.desired_target_px_size
            }
        }
        #[cfg(target_os = "windows")]
        ExternalVideoImportsMode::MfVideoDx12GpuCopy => st
            .mf_dx12
            .as_ref()
            .map(|v| v.size())
            .unwrap_or(st.desired_target_px_size),
    };

    let needs_realloc = st.texture.is_none() || st.target_px_size != desired;
    if needs_realloc {
        // Allocate the shared-allocation texture as *linear* BGRA8, but expose an sRGB view for
        // the UI contract-path surface. This avoids relying on backend-specific support for
        // wrapping SRGB-format resources in D3D11On12 interop.
        let view_formats = [wgpu::TextureFormat::Bgra8UnormSrgb];
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("external video imports texture"),
            size: wgpu::Extent3d {
                width: desired.0.max(1),
                height: desired.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &view_formats,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(st.target.format()),
            ..Default::default()
        });

        if !st.target.is_registered() {
            let _ = st.target.ensure_registered(renderer, view.clone(), desired);
        }

        st.texture = Some(texture);
        st.target_px_size = desired;
    }

    let texture = st.texture.as_ref().expect("texture allocated");
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        format: Some(st.target.format()),
        ..Default::default()
    });

    let mut metadata = RenderTargetMetadata::default();
    match st.mode {
        ExternalVideoImportsMode::CheckerGpu => {
            st.target.push_update_with_ingest_strategies(
                &mut update,
                view.clone(),
                st.target_px_size,
                metadata,
                RenderTargetIngestStrategy::Owned,
                RenderTargetIngestStrategy::Owned,
            );

            // A tiny animated clear tint is enough to keep the contract-path hot.
            let t = frame_id.0 as f32 * (1.0 / 60.0);
            let pulse = (t * 0.5).sin() * 0.5 + 0.5;
            let color = wgpu::Color {
                r: 0.06 + pulse as f64 * 0.02,
                g: 0.08 + pulse as f64 * 0.03,
                b: 0.11 + pulse as f64 * 0.04,
                a: 1.0,
            };
            let mut encoder =
                context
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("external video imports checker encoder"),
                    });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("external video imports checker pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(color),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                let _ = &mut pass;
            }
            update.push_command_buffer(encoder.finish());
        }
        #[cfg(target_os = "windows")]
        ExternalVideoImportsMode::MfVideoCpuUpload => {
            if let Some(reader) = st.mf.as_ref() {
                metadata.color_encoding = reader.color_encoding();
            }

            // Stage M2A: request the v2 ceiling, but deterministically degrade to CPU upload.
            // The requested/effective split is visible in perf bundles.
            st.target.push_update_with_ingest_strategies(
                &mut update,
                view.clone(),
                st.target_px_size,
                metadata,
                RenderTargetIngestStrategy::ExternalZeroCopy,
                RenderTargetIngestStrategy::CpuUpload,
            );

            if let Some((size, bytes_per_row, bgra8)) = decoded_frame.as_ref() {
                // The helper pads rows to wgpu alignment; the bytes are BGRA8 matching the texture format.
                fret_render::write_rgba8_texture_region(
                    &context.queue,
                    texture,
                    (0, 0),
                    *size,
                    *bytes_per_row,
                    bgra8,
                );
            }
        }
        #[cfg(target_os = "windows")]
        ExternalVideoImportsMode::MfVideoDx12GpuCopy => {
            // Stage M2B (shared allocation / GPU copy): request v2's ceiling, but deterministically
            // degrade to a GPU copy into a renderer-owned texture on capable backends.
            let Some(path) = std::env::var("FRET_MF_VIDEO_PATH")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
            else {
                tracing::info!("FRET_MF_VIDEO_PATH is not set; falling back to checker mode");
                st.mode = ExternalVideoImportsMode::CheckerGpu;
                return update;
            };

            if st.mf_dx12.is_none() {
                let export =
                    match fret_launch::runner::dx12::Dx12SharedAllocationWriteGuard::export_raw(
                        context, texture,
                    ) {
                        Ok(export) => export,
                        Err(err) => {
                            tracing::warn!(
                                ?err,
                                "MF DX12 GPU copy mode requested but backend is not DX12; falling back"
                            );
                            st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                            return update;
                        }
                    };

                match wmf::Dx12Interop::new(&export.queue, &export.resource, &path)
                    .context("init MF DX12 interop")
                {
                    Ok(v) => st.mf_dx12 = Some(v),
                    Err(err) => {
                        tracing::warn!(?err, "failed to init MF DX12 interop; falling back");
                        st.mf_dx12 = None;
                        st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                        return update;
                    }
                }
            }

            let Some(interop) = st.mf_dx12.as_mut() else {
                st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                return update;
            };
            metadata.color_encoding = interop.color_encoding();

            st.target.push_update_with_ingest_strategies(
                &mut update,
                view.clone(),
                st.target_px_size,
                metadata,
                RenderTargetIngestStrategy::ExternalZeroCopy,
                RenderTargetIngestStrategy::GpuCopy,
            );

            let src = match interop.read_next_dxgi_texture() {
                Ok(Some(tex)) => tex,
                Ok(None) => {
                    // End-of-stream: drop state and restart on next frame.
                    st.mf_dx12 = None;
                    return update;
                }
                Err(err) => {
                    tracing::warn!(?err, "MF DX12 GPU copy read failed; falling back");
                    st.mf_dx12 = None;
                    st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                    return update;
                }
            };

            let guard = match fret_launch::runner::dx12::Dx12SharedAllocationWriteGuard::begin(
                context,
                texture,
                wgpu::wgt::TextureUses::COPY_DST,
            ) {
                Ok(guard) => guard,
                Err(err) => {
                    tracing::warn!(
                        ?err,
                        "MF DX12 GPU copy mode: shared allocation write unavailable; falling back"
                    );
                    st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                    return update;
                }
            };

            if let Err(err) = interop.copy_into_dx12_shared_allocation(guard.resource(), &src) {
                tracing::warn!(?err, "MF DX12 GPU copy mode: copy failed; falling back");
                st.mf_dx12 = None;
                st.mode = ExternalVideoImportsMode::MfVideoCpuUpload;
                return update;
            }
            guard.finish();
        }
    }

    app.push_effect(fret_app::Effect::RequestAnimationFrame(window));
    update
}

pub fn run() -> anyhow::Result<()> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("fret=info".parse().unwrap())
                .add_directive("fret_render=info".parse().unwrap())
                .add_directive("fret_launch=info".parse().unwrap()),
        )
        .try_init();

    let builder = fret::app_with_hooks("external-video-imports-mf", init_window, view, |driver| {
        driver
            .on_event(on_event)
            .record_engine_frame(record_engine_frame)
    })?
    .init_app(|app| {
        app.set_global(PlatformCapabilities::default());
    })
    .with_main_window(
        "fret-demo external_video_imports_mf_demo (V toggles visibility, I toggles source)",
        (980.0, 720.0),
    );

    builder.run().context("run external_video_imports_mf_demo")
}
