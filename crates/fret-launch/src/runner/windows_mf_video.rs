//! Windows Media Foundation helpers for EXTV2 native frame sources.
//!
//! This module is runner-facing and intentionally capability-gated. It is used by demos to wire
//! a real native video source into the EXTV2 staging shape without leaking backend handles into
//! UI code.

use anyhow::Context as _;
use fret_render::{
    RenderTargetColorEncoding, RenderTargetColorPrimaries, RenderTargetColorRange,
    RenderTargetMatrixCoefficients, RenderTargetTransferFunction,
};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use windows::Win32::Graphics::Direct3D11::{
    D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_CREATE_DEVICE_BGRA_SUPPORT,
    D3D11_CREATE_DEVICE_VIDEO_SUPPORT, D3D11_TEX2D_VPIV, D3D11_TEX2D_VPOV, D3D11_TEXTURE2D_DESC,
    D3D11_USAGE_DEFAULT, D3D11_VIDEO_FRAME_FORMAT_PROGRESSIVE, D3D11_VIDEO_PROCESSOR_CONTENT_DESC,
    D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC, D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC,
    D3D11_VIDEO_PROCESSOR_STREAM, D3D11_VIDEO_USAGE_PLAYBACK_NORMAL,
    D3D11_VPIV_DIMENSION_TEXTURE2D, D3D11_VPOV_DIMENSION_TEXTURE2D, ID3D11Device,
    ID3D11DeviceContext, ID3D11Resource, ID3D11Texture2D, ID3D11VideoContext, ID3D11VideoDevice,
    ID3D11VideoProcessor, ID3D11VideoProcessorEnumerator, ID3D11VideoProcessorInputView,
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
    MFNominalRange_0_255, MFNominalRange_16_235, MFShutdown, MFStartup, MFVideoFormat_RGB32,
    MFVideoPrimaries, MFVideoPrimaries_BT709, MFVideoPrimaries_BT2020, MFVideoPrimaries_DCI_P3,
    MFVideoTransFunc_709, MFVideoTransFunc_2084, MFVideoTransFunc_HLG, MFVideoTransFunc_Unknown,
    MFVideoTransFunc_sRGB, MFVideoTransferFunction, MFVideoTransferMatrix,
    MFVideoTransferMatrix_BT601, MFVideoTransferMatrix_BT709, MFVideoTransferMatrix_BT2020_10,
    MFVideoTransferMatrix_Unknown,
};
use windows::Win32::Media::MediaFoundation::{IMFDXGIBuffer, IMFDXGIDeviceManager};
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};
use windows::core::{BOOL, HSTRING, Interface as _};

use super::{EngineFrameKeepalive, NativeExternalImportError, NativeExternalImportedFrame};
use super::{NativeExternalTextureFrame, SharedAllocationExportError};
use fret_render::{
    RenderTargetIngestStrategy, RenderTargetMetadata, RendererCapabilities, WgpuContext,
};

use super::dx12::Dx12SharedAllocationWriteGuard;

const VIDEO_FILE_EXTS: &[&str] = &["mp4", "m4v", "mov", "wmv", "avi", "mkv", "webm"];

const MF_E_UNSUPPORTED_BYTESTREAM_TYPE: u32 = 0xC00D_36C4;

fn file_url_from_canonical_path(path: &PathBuf) -> String {
    let mut s = path.to_string_lossy().replace('\\', "/");
    if s.len() >= 2 && s.as_bytes()[1] == b':' {
        s.insert(0, '/');
    }
    s = s.replace(' ', "%20");
    format!("file://{s}")
}

fn mf_open_hint_for_hresult(hr: u32) -> Option<&'static str> {
    match hr {
        MF_E_UNSUPPORTED_BYTESTREAM_TYPE => Some(
            "MF_E_UNSUPPORTED_BYTESTREAM_TYPE: this file type/codec may not be supported on this machine (common on Windows N without the Media Feature Pack). Try a different file or install missing codecs.",
        ),
        _ => None,
    }
}

fn source_reader_candidates(raw: &str) -> anyhow::Result<Vec<String>> {
    let raw = raw.trim();
    anyhow::ensure!(!raw.is_empty(), "empty Media Foundation source URL/path");

    // Allow callers to pass a real URL (including `file://...`) without trying to normalize it.
    if raw.contains("://") {
        tracing::info!(raw, "using Media Foundation source URL");
        return Ok(vec![raw.to_string()]);
    }

    let path = PathBuf::from(raw);
    if path.is_dir() {
        let mut entries: Vec<_> = std::fs::read_dir(&path)
            .with_context(|| format!("read_dir({})", path.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(OsStr::to_str)
                    .map(|ext| VIDEO_FILE_EXTS.iter().any(|e| ext.eq_ignore_ascii_case(e)))
                    .unwrap_or(false)
            })
            .collect();

        entries.sort();

        let mut out = Vec::with_capacity(entries.len());
        for picked in entries {
            let picked = std::fs::canonicalize(&picked)
                .with_context(|| format!("canonicalize({})", picked.display()))?;
            out.push(picked.to_string_lossy().to_string());
            out.push(file_url_from_canonical_path(&picked));
        }

        return Ok(out);
    }

    let path = std::fs::canonicalize(&path)
        .with_context(|| format!("canonicalize({})", path.display()))?;
    tracing::info!(path = %path.display(), "using Media Foundation source file path");
    Ok(vec![
        path.to_string_lossy().to_string(),
        file_url_from_canonical_path(&path),
    ])
}

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
        let candidates = source_reader_candidates(&raw_path)?;
        anyhow::ensure!(
            !candidates.is_empty(),
            "FRET_MF_VIDEO_PATH points to a directory but no candidate video files were found (extensions: {:?})",
            VIDEO_FILE_EXTS
        );

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

        let mut reader: Option<IMFSourceReader> = None;
        let mut resolved_path: Option<String> = None;
        let mut last_err: Option<anyhow::Error> = None;
        for candidate in &candidates {
            let hpath = HSTRING::from(candidate.as_str());
            match unsafe { MFCreateSourceReaderFromURL(&hpath, Some(&attributes)) } {
                Ok(v) => {
                    reader = Some(v);
                    resolved_path = Some(candidate.clone());
                    break;
                }
                Err(err) => {
                    let hr = err.code().0 as u32;
                    if let Some(hint) = mf_open_hint_for_hresult(hr) {
                        tracing::warn!(
                            hr = format_args!("0x{hr:08X}"),
                            candidate,
                            hint,
                            "MF source reader init failed"
                        );
                    }
                    last_err = Some(anyhow::Error::new(err).context(format!(
                        "MFCreateSourceReaderFromURL({candidate}) hr=0x{hr:08X}"
                    )));
                }
            }
        }

        let reader = reader.with_context(|| {
            let last = last_err
                .map(|e| format!("{e:?}"))
                .unwrap_or_else(|| "unknown error".to_string());
            format!(
                "failed to initialize MF source reader from any candidate (count={}); last_error={last}",
                candidates.len()
            )
        })?;
        let resolved = resolved_path.unwrap_or_else(|| raw_path.clone());
        if resolved != raw_path {
            tracing::info!(
                raw = raw_path,
                resolved,
                "resolved MF video source from candidates"
            );
        }

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
                .context("SetCurrentMediaType(RGB32)")?;
        }

        let (size, bytes_per_row) = Self::query_video_layout(&reader)?;
        let color_encoding = Self::query_color_encoding(&reader)?;

        Ok(Self {
            path: raw_path,
            reader,
            size,
            bytes_per_row,
            color_encoding,
        })
    }

    pub fn color_encoding(&self) -> RenderTargetColorEncoding {
        self.color_encoding
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
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

            let buffer = unsafe { sample.ConvertToContiguousBuffer() }
                .context("ConvertToContiguousBuffer")?;
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

    fn query_video_layout(reader: &IMFSourceReader) -> anyhow::Result<((u32, u32), u32)> {
        let media_type = unsafe {
            reader
                .GetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
                .context("GetCurrentMediaType")?
        };

        let size = unsafe {
            media_type
                .GetUINT64(&windows::Win32::Media::MediaFoundation::MF_MT_FRAME_SIZE)
                .context("GetUINT64(MF_MT_FRAME_SIZE)")?
        };
        let w = (size >> 32) as u32;
        let h = (size & 0xffff_ffff) as u32;

        let bytes_per_row = unsafe {
            media_type
                .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_DEFAULT_STRIDE)
                .unwrap_or(w.saturating_mul(4))
        };

        Ok(((w, h), bytes_per_row))
    }

    #[allow(non_upper_case_globals)]
    fn query_color_encoding(reader: &IMFSourceReader) -> anyhow::Result<RenderTargetColorEncoding> {
        let media_type = unsafe {
            reader
                .GetCurrentMediaType(MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32)
                .context("GetCurrentMediaType")?
        };

        Ok(RenderTargetColorEncoding {
            primaries: unsafe {
                media_type
                    .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_VIDEO_PRIMARIES)
                    .ok()
                    .map(|v| MFVideoPrimaries(v as i32))
            }
            .map(|primaries| match primaries {
                MFVideoPrimaries_BT709 => RenderTargetColorPrimaries::Bt709,
                MFVideoPrimaries_DCI_P3 => RenderTargetColorPrimaries::DisplayP3,
                MFVideoPrimaries_BT2020 => RenderTargetColorPrimaries::Bt2020,
                _ => RenderTargetColorPrimaries::Unknown,
            })
            .unwrap_or(RenderTargetColorPrimaries::Unknown),
            transfer: unsafe {
                media_type
                    .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_TRANSFER_FUNCTION)
                    .ok()
                    .map(|v| MFVideoTransferFunction(v as i32))
            }
            .map(|transfer| match transfer {
                MFVideoTransFunc_sRGB => RenderTargetTransferFunction::Srgb,
                MFVideoTransFunc_709 => RenderTargetTransferFunction::Bt1886,
                MFVideoTransFunc_2084 => RenderTargetTransferFunction::Pq,
                MFVideoTransFunc_HLG => RenderTargetTransferFunction::Hlg,
                MFVideoTransFunc_Unknown => RenderTargetTransferFunction::Unknown,
                _ => RenderTargetTransferFunction::Unknown,
            })
            .unwrap_or(RenderTargetTransferFunction::Unknown),
            matrix: unsafe {
                media_type
                    .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_YUV_MATRIX)
                    .ok()
                    .map(|v| MFVideoTransferMatrix(v as i32))
            }
            .map(|matrix| match matrix {
                MFVideoTransferMatrix_BT601 => RenderTargetMatrixCoefficients::Bt601,
                MFVideoTransferMatrix_BT709 => RenderTargetMatrixCoefficients::Bt709,
                MFVideoTransferMatrix_BT2020_10 => RenderTargetMatrixCoefficients::Bt2020Ncl,
                // Our stage M2A output is RGB32, so a missing/unknown YUV matrix is best treated
                // as identity for now.
                MFVideoTransferMatrix_Unknown => RenderTargetMatrixCoefficients::Rgb,
                _ => RenderTargetMatrixCoefficients::Unknown,
            })
            .unwrap_or(RenderTargetMatrixCoefficients::Rgb),
            range: unsafe {
                media_type
                    .GetUINT32(&windows::Win32::Media::MediaFoundation::MF_MT_VIDEO_NOMINAL_RANGE)
                    .ok()
                    .map(|v| MFNominalRange(v as i32))
            }
            .map(|range| match range {
                MFNominalRange_0_255 => RenderTargetColorRange::Full,
                MFNominalRange_16_235 => RenderTargetColorRange::Limited,
                _ => RenderTargetColorRange::Unknown,
            })
            .unwrap_or(RenderTargetColorRange::Unknown),
        })
    }
}

impl Drop for MfVideoReader {
    fn drop(&mut self) {
        // Avoid hard failures on shutdown: MF is process-global and may have been initialized by
        // other subsystems.
        unsafe {
            let _ = MFShutdown();
        }
        tracing::debug!(path = self.path, "dropped MF video reader");
    }
}

#[derive(Debug)]
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
        let video_device: ID3D11VideoDevice = d3d11.cast().context("cast to ID3D11VideoDevice")?;
        let video_context: ID3D11VideoContext =
            context.cast().context("cast to ID3D11VideoContext")?;

        // Create an MF DXGI device manager backed by our D3D11On12 device.
        let mut reset_token = 0u32;
        let mut device_manager: Option<IMFDXGIDeviceManager> = None;
        unsafe {
            MFCreateDXGIDeviceManager(&mut reset_token, &mut device_manager)
                .context("MFCreateDXGIDeviceManager")?;
        }
        let device_manager = device_manager.context("MFCreateDXGIDeviceManager returned None")?;
        unsafe {
            device_manager
                .ResetDevice(&d3d11, reset_token)
                .context("IMFDXGIDeviceManager::ResetDevice")?;
        }

        let candidates = source_reader_candidates(path)?;
        anyhow::ensure!(
            !candidates.is_empty(),
            "FRET_MF_VIDEO_PATH points to a directory but no candidate video files were found (extensions: {:?})",
            VIDEO_FILE_EXTS
        );

        let mut attributes: Option<IMFAttributes> = None;
        unsafe {
            MFCreateAttributes(&mut attributes, 2).context("MFCreateAttributes")?;
        }
        let attributes = attributes.context("MFCreateAttributes returned None")?;
        unsafe {
            attributes
                .SetUnknown(
                    &MF_SOURCE_READER_D3D_MANAGER,
                    Some(&device_manager.cast::<windows::core::IUnknown>()?),
                )
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
                MFCreateSourceReaderFromURL(hpath, Some(attributes)).with_context(|| {
                    format!(
                        "MFCreateSourceReaderFromURL({resolved_path}) hw_transforms={enable_hw_transforms} video_processing={enable_video_processing}"
                    )
                })
            }
        }

        let mut reader: Option<IMFSourceReader> = None;
        let mut resolved_path: Option<String> = None;
        let mut last_err: Option<anyhow::Error> = None;
        for candidate in &candidates {
            let hpath = HSTRING::from(candidate.as_str());
            // Prefer enabling video processing so MF can deliver RGB32 frames without forcing us to
            // run an explicit NV12->BGRA conversion path in the demo. If this fails on a given
            // machine/codec, fall back to the conservative configuration.
            match try_create_source_reader(&hpath, &attributes, candidate, 1, 1) {
                Ok(v) => {
                    reader = Some(v);
                    resolved_path = Some(candidate.clone());
                    break;
                }
                Err(err) => {
                    if let Some(werr) = err.downcast_ref::<windows::core::Error>() {
                        let hr = werr.code().0 as u32;
                        if let Some(hint) = mf_open_hint_for_hresult(hr) {
                            tracing::warn!(
                                hr = format_args!("0x{hr:08X}"),
                                candidate,
                                hint,
                                "MF DX12 interop: source reader init failed"
                            );
                        }
                    }
                    tracing::warn!(
                        ?err,
                        candidate,
                        "MF DX12 interop: SourceReader init failed with video processing enabled; retrying with conservative flags"
                    );
                    match try_create_source_reader(&hpath, &attributes, candidate, 0, 0) {
                        Ok(v) => {
                            reader = Some(v);
                            resolved_path = Some(candidate.clone());
                            break;
                        }
                        Err(err2) => {
                            if let Some(werr) = err2.downcast_ref::<windows::core::Error>() {
                                let hr = werr.code().0 as u32;
                                if let Some(hint) = mf_open_hint_for_hresult(hr) {
                                    tracing::warn!(
                                        hr = format_args!("0x{hr:08X}"),
                                        candidate,
                                        hint,
                                        "MF DX12 interop: source reader init failed (conservative flags)"
                                    );
                                }
                            }
                            last_err = Some(err2);
                        }
                    }
                }
            }
        }

        let reader = reader.with_context(|| {
            let last = last_err
                .map(|e| format!("{e:?}"))
                .unwrap_or_else(|| "unknown error".to_string());
            format!(
                "MF DX12 interop: failed to initialize MF source reader from any candidate (count={}); last_error={last}",
                candidates.len()
            )
        })?;
        let resolved_path = resolved_path.unwrap_or_else(|| path.to_string());
        if resolved_path != path {
            tracing::info!(
                raw = path,
                resolved = resolved_path,
                "MF DX12 interop: resolved video source from candidates"
            );
        }

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
                let (size, _bytes_per_row) = MfVideoReader::query_video_layout(&self.mf_reader)?;
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

            let vp_enum = unsafe {
                self.video_device
                    .CreateVideoProcessorEnumerator(&content)
                    .context("CreateVideoProcessorEnumerator")?
            };
            let vp = unsafe {
                self.video_device
                    .CreateVideoProcessor(&vp_enum, 0)
                    .context("CreateVideoProcessor")?
            };

            self.vp_enum = Some(vp_enum);
            self.vp = Some(vp);
        }

        let en = self
            .vp_enum
            .as_ref()
            .context("video processor enumerator missing")?;
        let vp = self.vp.as_ref().context("video processor missing")?;
        let tmp = self.tmp_bgra.as_ref().context("tmp_bgra missing")?.clone();

        let mut input_desc: D3D11_VIDEO_PROCESSOR_INPUT_VIEW_DESC = unsafe { std::mem::zeroed() };
        input_desc.FourCC = 0;
        input_desc.ViewDimension = D3D11_VPIV_DIMENSION_TEXTURE2D;
        input_desc.Anonymous.Texture2D = D3D11_TEX2D_VPIV {
            MipSlice: 0,
            ArraySlice: 0,
        };
        let mut input_view: Option<ID3D11VideoProcessorInputView> = None;
        unsafe {
            self.video_device
                .CreateVideoProcessorInputView(src_texture, en, &input_desc, Some(&mut input_view))
                .context("CreateVideoProcessorInputView")?;
        }
        let input_view = input_view.context("CreateVideoProcessorInputView returned None")?;

        let mut output_desc: D3D11_VIDEO_PROCESSOR_OUTPUT_VIEW_DESC = unsafe { std::mem::zeroed() };
        output_desc.ViewDimension = D3D11_VPOV_DIMENSION_TEXTURE2D;
        output_desc.Anonymous.Texture2D = D3D11_TEX2D_VPOV { MipSlice: 0 };
        let mut output_view: Option<ID3D11VideoProcessorOutputView> = None;
        unsafe {
            self.video_device
                .CreateVideoProcessorOutputView(&tmp, en, &output_desc, Some(&mut output_view))
                .context("CreateVideoProcessorOutputView")?;
        }
        let output_view = output_view.context("CreateVideoProcessorOutputView returned None")?;

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

impl Drop for Dx12Interop {
    fn drop(&mut self) {
        unsafe {
            let _ = MFShutdown();
        }
    }
}

#[derive(Debug)]
pub enum Dx12GpuCopyTick {
    Copied {
        size: (u32, u32),
        color_encoding: RenderTargetColorEncoding,
    },
    EndOfStream,
}

#[derive(Debug, Error)]
pub enum Dx12GpuCopySessionError {
    #[error(transparent)]
    SharedAllocation(#[from] SharedAllocationExportError),
    #[error(transparent)]
    Mf(#[from] anyhow::Error),
}

#[derive(Debug)]
pub struct Dx12GpuCopySession {
    interop: Dx12Interop,
}

impl Dx12GpuCopySession {
    pub fn new(
        ctx: &fret_render::WgpuContext,
        texture: &wgpu::Texture,
        path: &str,
    ) -> Result<Self, Dx12GpuCopySessionError> {
        let export = Dx12SharedAllocationWriteGuard::export_raw(ctx, texture)?;
        let interop = Dx12Interop::new(&export.queue, &export.resource, path)?;
        Ok(Self { interop })
    }

    pub fn size(&self) -> (u32, u32) {
        self.interop.size()
    }

    pub fn tick(
        &mut self,
        ctx: &fret_render::WgpuContext,
        texture: &wgpu::Texture,
    ) -> Result<Dx12GpuCopyTick, Dx12GpuCopySessionError> {
        let src = match self.interop.read_next_dxgi_texture()? {
            Some(v) => v,
            None => return Ok(Dx12GpuCopyTick::EndOfStream),
        };

        let guard =
            Dx12SharedAllocationWriteGuard::begin(ctx, texture, wgpu::wgt::TextureUses::COPY_DST)?;
        self.interop
            .copy_into_dx12_shared_allocation(guard.resource(), &src)?;
        guard.finish();

        Ok(Dx12GpuCopyTick::Copied {
            size: self.interop.size(),
            color_encoding: self.interop.color_encoding(),
        })
    }
}

#[derive(Clone)]
pub struct MfVideoNativeExternalImporter {
    inner: Arc<Mutex<MfVideoNativeExternalState>>,
}

struct MfVideoNativeExternalState {
    path: String,
    prefer_dx12_gpu_copy: bool,
    cpu_reader: Option<MfVideoReader>,
    dx12_session: Option<Dx12GpuCopySession>,
    texture: Option<wgpu::Texture>,
    texture_size: (u32, u32),
}

struct MfVideoNativeExternalFrame {
    inner: Arc<Mutex<MfVideoNativeExternalState>>,
}

struct MfVideoNativeExternalKeepalive {
    _inner: Arc<Mutex<MfVideoNativeExternalState>>,
}

impl MfVideoNativeExternalImporter {
    pub fn new(path: impl Into<String>, prefer_dx12_gpu_copy: bool) -> Self {
        Self {
            inner: Arc::new(Mutex::new(MfVideoNativeExternalState {
                path: path.into(),
                prefer_dx12_gpu_copy,
                cpu_reader: None,
                dx12_session: None,
                texture: None,
                texture_size: (0, 0),
            })),
        }
    }

    pub fn path(&self) -> String {
        self.inner
            .lock()
            .ok()
            .map(|v| v.path.clone())
            .unwrap_or_default()
    }

    pub fn prefer_dx12_gpu_copy(&self) -> bool {
        self.inner
            .lock()
            .ok()
            .map(|v| v.prefer_dx12_gpu_copy)
            .unwrap_or(false)
    }

    pub fn last_size(&self) -> Option<(u32, u32)> {
        self.inner.lock().ok().and_then(|v| {
            if v.texture.is_some() && v.texture_size.0 > 0 && v.texture_size.1 > 0 {
                Some(v.texture_size)
            } else {
                None
            }
        })
    }

    pub fn frame(&self) -> Box<dyn NativeExternalTextureFrame> {
        Box::new(MfVideoNativeExternalFrame {
            inner: self.inner.clone(),
        })
    }
}

impl MfVideoNativeExternalState {
    fn ensure_bgra_shared_texture(ctx: &WgpuContext, size: (u32, u32)) -> wgpu::Texture {
        let (w, h) = size;
        let view_formats = [wgpu::TextureFormat::Bgra8UnormSrgb];
        ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("mf video native external texture (bgra)"),
            size: wgpu::Extent3d {
                width: w.max(1),
                height: h.max(1),
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
        })
    }

    fn view_srgb(texture: &wgpu::Texture) -> wgpu::TextureView {
        texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(wgpu::TextureFormat::Bgra8UnormSrgb),
            ..Default::default()
        })
    }
}

impl NativeExternalTextureFrame for MfVideoNativeExternalFrame {
    fn import(
        self: Box<Self>,
        ctx: &WgpuContext,
        _caps: &RendererCapabilities,
    ) -> Result<NativeExternalImportedFrame, NativeExternalImportError> {
        let keepalive = EngineFrameKeepalive::new(MfVideoNativeExternalKeepalive {
            _inner: self.inner.clone(),
        });

        let mut guard = self
            .inner
            .lock()
            .map_err(|_| NativeExternalImportError::Failed {
                reason: "mf_native_external_state_lock_poisoned",
            })?;

        let path = guard.path.trim().to_string();
        if path.is_empty() {
            return Err(NativeExternalImportError::Failed {
                reason: "mf_video_path_empty",
            });
        }

        let mut metadata = RenderTargetMetadata::default();
        metadata.requested_ingest_strategy = RenderTargetIngestStrategy::ExternalZeroCopy;

        if guard.prefer_dx12_gpu_copy {
            if guard.texture.is_none() {
                let texture = MfVideoNativeExternalState::ensure_bgra_shared_texture(ctx, (1, 1));
                guard.texture = Some(texture);
                guard.texture_size = (1, 1);
            }

            let mut ensure_session = || -> Result<(), Dx12GpuCopySessionError> {
                let texture = guard.texture.as_ref().expect("texture exists");
                if guard.dx12_session.is_none() {
                    guard.dx12_session = Some(Dx12GpuCopySession::new(ctx, texture, &path)?);
                }
                Ok(())
            };

            match ensure_session() {
                Ok(()) => {}
                Err(Dx12GpuCopySessionError::SharedAllocation(
                    SharedAllocationExportError::UnsupportedBackend,
                )) => {
                    // Deterministic downgrade: DX12-only path unavailable.
                    guard.prefer_dx12_gpu_copy = false;
                }
                Err(_) => {
                    // Deterministic downgrade: if the fast path cannot be established, fall back to CPU upload.
                    guard.prefer_dx12_gpu_copy = false;
                    guard.dx12_session = None;
                }
            }
        }

        if guard.prefer_dx12_gpu_copy {
            let session_size = guard
                .dx12_session
                .as_ref()
                .map(|s| s.size())
                .unwrap_or((1, 1));

            if session_size != guard.texture_size {
                guard.texture = Some(MfVideoNativeExternalState::ensure_bgra_shared_texture(
                    ctx,
                    session_size,
                ));
                guard.texture_size = session_size;
                let texture = guard.texture.as_ref().expect("texture allocated");
                match Dx12GpuCopySession::new(ctx, texture, &path) {
                    Ok(session) => {
                        guard.dx12_session = Some(session);
                    }
                    Err(_) => {
                        guard.prefer_dx12_gpu_copy = false;
                        guard.dx12_session = None;
                    }
                }
            }

            if guard.prefer_dx12_gpu_copy {
                let texture = guard.texture.as_ref().expect("texture exists").clone();
                let tick = {
                    let session = guard.dx12_session.as_mut().expect("session exists");
                    session.tick(ctx, &texture)
                };
                match tick {
                    Ok(Dx12GpuCopyTick::Copied {
                        size,
                        color_encoding,
                    }) => {
                        guard.texture_size = size;
                        metadata.ingest_strategy = RenderTargetIngestStrategy::GpuCopy;
                        metadata.color_encoding = color_encoding;

                        let view = MfVideoNativeExternalState::view_srgb(&texture);
                        return Ok(NativeExternalImportedFrame {
                            view,
                            size,
                            metadata,
                            keepalive,
                        });
                    }
                    Ok(Dx12GpuCopyTick::EndOfStream) => {
                        // Deterministic loop: recreate the session and try CPU upload on the next frame.
                        guard.dx12_session = None;
                        guard.prefer_dx12_gpu_copy = false;
                    }
                    Err(_) => {
                        guard.dx12_session = None;
                        guard.prefer_dx12_gpu_copy = false;
                    }
                }
            }
        }

        if guard.cpu_reader.is_none() {
            guard.cpu_reader = Some(MfVideoReader::new(path.clone()).map_err(|_| {
                NativeExternalImportError::Failed {
                    reason: "mf_reader_init_failed",
                }
            })?);
        }
        let reader = guard.cpu_reader.as_mut().expect("cpu reader exists");

        let frame = match reader
            .read_next()
            .map_err(|_| NativeExternalImportError::Failed {
                reason: "mf_reader_read_next_failed",
            })? {
            Some(v) => v,
            None => {
                reader.reset().ok();
                reader
                    .read_next()
                    .map_err(|_| NativeExternalImportError::Failed {
                        reason: "mf_reader_read_next_failed_after_reset",
                    })?
                    .ok_or(NativeExternalImportError::Failed {
                        reason: "mf_reader_end_of_stream",
                    })?
            }
        };

        metadata.ingest_strategy = RenderTargetIngestStrategy::CpuUpload;
        metadata.color_encoding = reader.color_encoding();

        if guard.texture.is_none() || guard.texture_size != frame.size {
            guard.texture = Some(MfVideoNativeExternalState::ensure_bgra_shared_texture(
                ctx, frame.size,
            ));
            guard.texture_size = frame.size;
        }
        let texture = guard.texture.as_ref().expect("texture allocated");
        fret_render::write_rgba8_texture_region(
            &ctx.queue,
            texture,
            (0, 0),
            frame.size,
            frame.bytes_per_row,
            &frame.bgra8,
        );

        let view = MfVideoNativeExternalState::view_srgb(texture);
        Ok(NativeExternalImportedFrame {
            view,
            size: frame.size,
            metadata,
            keepalive,
        })
    }
}

fn lock_and_copy(buffer: &IMFMediaBuffer) -> anyhow::Result<Vec<u8>> {
    let mut raw_ptr = std::ptr::null_mut::<u8>();
    let mut max_len = 0u32;
    let mut cur_len = 0u32;
    unsafe {
        buffer
            .Lock(&mut raw_ptr, Some(&mut max_len), Some(&mut cur_len))
            .context("IMFMediaBuffer::Lock")?;
    }
    let len = cur_len.min(max_len) as usize;
    let out = unsafe { std::slice::from_raw_parts(raw_ptr, len) }.to_vec();
    unsafe {
        buffer.Unlock().ok().context("IMFMediaBuffer::Unlock")?;
    }
    Ok(out)
}

/// Normalize a user-provided URL or filesystem path into a string accepted by
/// `MFCreateSourceReaderFromURL`.
///
/// - If `raw` looks like a URL (contains `://`), it is returned as-is.
/// - Otherwise the path is canonicalized (and directories pick the first matching video file).
pub fn resolve_source_reader_url(raw: &str) -> anyhow::Result<String> {
    source_reader_candidates(raw)?
        .into_iter()
        .next()
        .context("no Media Foundation source URL/path candidates")
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
        let resolved = resolved.replace('\\', "/").to_ascii_lowercase();
        assert!(
            !resolved.contains("://"),
            "expected a file path, got: {resolved}"
        );
        assert!(
            resolved.ends_with("/a.mp4"),
            "expected a.mp4, got: {resolved}"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn source_reader_candidates_includes_file_url_variant_for_file() {
        let root = std::env::temp_dir().join(format!(
            "fret_mf_video_candidates_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        ));
        std::fs::create_dir_all(&root).unwrap();

        let path = root.join("a.mp4");
        std::fs::write(&path, b"dummy").unwrap();

        let candidates = source_reader_candidates(path.to_str().unwrap()).unwrap();
        assert!(
            candidates.len() >= 2,
            "expected at least 2 candidates (path + file:// url), got {:?}",
            candidates
        );
        assert!(
            candidates[0].to_ascii_lowercase().ends_with("a.mp4"),
            "expected a file path first, got {}",
            candidates[0]
        );
        assert!(
            candidates[1].to_ascii_lowercase().starts_with("file://"),
            "expected a file:// url second, got {}",
            candidates[1]
        );

        let _ = std::fs::remove_dir_all(&root);
    }
}
