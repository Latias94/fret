use super::*;
use crate::runner::{streaming_upload, yuv, yuv_gpu};
use fret_render::UploadedRgba8Image;

pub(super) struct UploadedImageEntry {
    pub(super) uploaded: UploadedRgba8Image,
    pub(super) stream_generation: u64,
    pub(super) alpha_mode: fret_core::AlphaMode,
    pub(super) nv12_planes: Option<crate::runner::yuv_gpu::Nv12Planes>,
}

pub(super) struct StreamingImageUpdateRgba8<'a> {
    pub(super) window: Option<fret_core::AppWindowId>,
    pub(super) token: fret_core::ImageUpdateToken,
    pub(super) image: fret_core::ImageId,
    pub(super) stream_generation: u64,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) update_rect_px: Option<fret_core::RectPx>,
    pub(super) bytes_per_row: u32,
    pub(super) bytes: &'a [u8],
    pub(super) color_info: fret_core::ImageColorInfo,
    pub(super) alpha_mode: fret_core::AlphaMode,
}

pub(super) struct StreamingImageUpdateNv12<'a> {
    pub(super) window: Option<fret_core::AppWindowId>,
    pub(super) token: fret_core::ImageUpdateToken,
    pub(super) image: fret_core::ImageId,
    pub(super) stream_generation: u64,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) update_rect_px: Option<fret_core::RectPx>,
    pub(super) y_bytes_per_row: u32,
    pub(super) y_plane: &'a [u8],
    pub(super) uv_bytes_per_row: u32,
    pub(super) uv_plane: &'a [u8],
    pub(super) color_info: fret_core::ImageColorInfo,
}
impl<D: WinitAppDriver> WinitRunner<D> {
    pub(super) fn apply_streaming_image_update_rgba8(
        &mut self,
        stats: &mut streaming_upload::StreamingUploadStats,
        update: StreamingImageUpdateRgba8<'_>,
    ) {
        let StreamingImageUpdateRgba8 {
            window,
            token,
            image,
            stream_generation,
            width,
            height,
            update_rect_px,
            bytes_per_row,
            bytes,
            color_info,
            alpha_mode,
        } = update;

        let Some(context) = self.context.as_ref() else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::RendererNotReady,
                        },
                    );
                }
            }
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::RendererNotReady,
                        },
                    );
                }
            }
            return;
        };
        let Some(entry) = self.uploaded_images.get_mut(&image) else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::UnknownImage,
                        },
                    );
                }
            }
            return;
        };

        if width == 0 || height == 0 {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        if stream_generation < entry.stream_generation {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::Coalesced,
                        },
                    );
                }
            }
            return;
        }
        entry.stream_generation = stream_generation;

        let rect = update_rect_px.unwrap_or_else(|| fret_core::RectPx::full(width, height));
        if rect.is_empty() {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        if rect.x > width
            || rect.y > height
            || rect.x.saturating_add(rect.w) > width
            || rect.y.saturating_add(rect.h) > height
        {
            tracing::warn!(
                image = ?image,
                width,
                height,
                rect = ?rect,
                "ignoring ImageUpdateRgba8 with out-of-bounds update rect"
            );
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        let color_space = match color_info.encoding {
            fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
            fret_core::ImageEncoding::Linear => fret_render::ImageColorSpace::Linear,
        };

        let row_bytes = rect.w.saturating_mul(4);
        if bytes_per_row < row_bytes {
            tracing::warn!(
                image = ?image,
                bytes_per_row,
                row_bytes,
                "ignoring ImageUpdateRgba8 with undersized bytes_per_row"
            );
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        let expected_len = (bytes_per_row as usize).saturating_mul(rect.h as usize);
        if bytes.len() != expected_len {
            tracing::warn!(
                image = ?image,
                got = bytes.len(),
                expected = expected_len,
                "ignoring ImageUpdateRgba8 with invalid byte length"
            );
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return;
        }

        if entry.alpha_mode != alpha_mode {
            if !renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view: entry.uploaded.view.clone(),
                    size: entry.uploaded.size,
                    format: entry.uploaded.format,
                    color_space: entry.uploaded.color_space,
                    alpha_mode,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::UnknownImage,
                            },
                        );
                    }
                }
                return;
            }
            entry.alpha_mode = alpha_mode;
        }

        let needs_replace =
            entry.uploaded.size != (width, height) || entry.uploaded.color_space != color_space;
        let applied_upload_bytes = if needs_replace {
            let is_full_update = rect.x == 0 && rect.y == 0 && rect.w == width && rect.h == height;
            if !is_full_update {
                tracing::warn!(
                    image = ?image,
                    old_size = ?entry.uploaded.size,
                    new_size = ?(width, height),
                    "ignoring partial ImageUpdateRgba8 while image storage needs replace"
                );
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::Unsupported,
                            },
                        );
                    }
                }
                return;
            }

            let (applied_upload_bytes, uploaded) = if bytes_per_row == width.saturating_mul(4)
                && bytes.len()
                    == (width as usize)
                        .saturating_mul(height as usize)
                        .saturating_mul(4)
            {
                (
                    streaming_upload::estimate_rgba8_upload_bytes_for_rect(
                        fret_core::RectPx::full(width, height),
                        width.saturating_mul(4),
                    ),
                    fret_render::upload_rgba8_image(
                        &context.device,
                        &context.queue,
                        (width, height),
                        bytes,
                        color_space,
                    ),
                )
            } else {
                let uploaded = fret_render::create_rgba8_image_storage(
                    &context.device,
                    (width, height),
                    color_space,
                );
                uploaded.write_region(
                    &context.queue,
                    (0, 0),
                    (width, height),
                    bytes_per_row,
                    bytes,
                );
                (
                    streaming_upload::estimate_rgba8_upload_bytes_for_rect(
                        fret_core::RectPx::full(width, height),
                        bytes_per_row,
                    ),
                    uploaded,
                )
            };

            let view = uploaded
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            if !renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view,
                    size: uploaded.size,
                    format: uploaded.format,
                    color_space: uploaded.color_space,
                    alpha_mode,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::UnknownImage,
                            },
                        );
                    }
                }
                return;
            }
            entry.uploaded = uploaded;
            entry.alpha_mode = alpha_mode;
            entry.nv12_planes = None;
            applied_upload_bytes
        } else {
            entry.uploaded.write_region(
                &context.queue,
                (rect.x, rect.y),
                (rect.w, rect.h),
                bytes_per_row,
                bytes,
            );
            streaming_upload::estimate_rgba8_upload_bytes_for_rect(rect, bytes_per_row)
        };
        stats.upload_bytes_applied = stats
            .upload_bytes_applied
            .saturating_add(applied_upload_bytes);

        if self.config.streaming_update_ack_enabled {
            let target = window
                .or(self.main_window)
                .or_else(|| self.windows.keys().next());
            if let Some(target) = target {
                self.deliver_window_event_now(target, &Event::ImageUpdateApplied { token, image });
            }
        }

        if let Some(state) = window.and_then(|w| self.windows.get(w)) {
            state.window.request_redraw();
        } else {
            for (_id, state) in self.windows.iter() {
                state.window.request_redraw();
            }
        }
    }

    pub(super) fn try_apply_streaming_image_update_nv12_gpu(
        &mut self,
        stats: &mut streaming_upload::StreamingUploadStats,
        update: StreamingImageUpdateNv12<'_>,
    ) -> bool {
        let StreamingImageUpdateNv12 {
            window,
            token,
            image,
            stream_generation,
            width,
            height,
            update_rect_px,
            y_bytes_per_row,
            y_plane,
            uv_bytes_per_row,
            uv_plane,
            color_info,
        } = update;

        let requested = self.config.streaming_nv12_gpu_convert_enabled
            || std::env::var_os("FRET_STREAMING_GPU_YUV").is_some_and(|v| !v.is_empty());
        if !requested {
            return false;
        }

        let supported = self
            .renderer_caps
            .as_ref()
            .is_some_and(|c| c.streaming_images.nv12_gpu_convert);
        if !supported {
            return false;
        }

        if width == 0 || height == 0 {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return true;
        }

        let Some(context) = self.context.as_ref() else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::RendererNotReady,
                        },
                    );
                }
            }
            return true;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::RendererNotReady,
                        },
                    );
                }
            }
            return true;
        };

        let Some(entry) = self.uploaded_images.get_mut(&image) else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::UnknownImage,
                        },
                    );
                }
            }
            return true;
        };

        if stream_generation < entry.stream_generation {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::Coalesced,
                        },
                    );
                }
            }
            return true;
        }
        entry.stream_generation = stream_generation;

        let Ok(rect) = yuv::normalize_update_rect_420(width, height, update_rect_px) else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return true;
        };

        let color_space = match color_info.encoding {
            fret_core::ImageEncoding::Srgb => fret_render::ImageColorSpace::Srgb,
            fret_core::ImageEncoding::Linear => fret_render::ImageColorSpace::Linear,
        };

        if entry.uploaded.format != wgpu::TextureFormat::Rgba8UnormSrgb {
            return false;
        }
        if color_space != fret_render::ImageColorSpace::Srgb {
            return false;
        }

        let is_full_update = rect.x == 0 && rect.y == 0 && rect.w == width && rect.h == height;
        let needs_replace =
            entry.uploaded.size != (width, height) || entry.uploaded.color_space != color_space;
        if needs_replace {
            if !is_full_update {
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::Unsupported,
                            },
                        );
                    }
                }
                return true;
            }

            let uploaded = fret_render::create_rgba8_image_storage(
                &context.device,
                (width, height),
                color_space,
            );
            let view = uploaded
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            if !renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view,
                    size: uploaded.size,
                    format: uploaded.format,
                    color_space: uploaded.color_space,
                    alpha_mode: fret_core::AlphaMode::Opaque,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::UnknownImage,
                            },
                        );
                    }
                }
                return true;
            }
            entry.uploaded = uploaded;
            entry.alpha_mode = fret_core::AlphaMode::Opaque;
            entry.nv12_planes = None;
        }

        if entry.alpha_mode != fret_core::AlphaMode::Opaque {
            if !renderer.update_image(
                image,
                fret_render::ImageDescriptor {
                    view: entry.uploaded.view.clone(),
                    size: entry.uploaded.size,
                    format: entry.uploaded.format,
                    color_space: entry.uploaded.color_space,
                    alpha_mode: fret_core::AlphaMode::Opaque,
                },
            ) {
                self.uploaded_images.remove(&image);
                if self.config.streaming_update_ack_enabled {
                    let target = window
                        .or(self.main_window)
                        .or_else(|| self.windows.keys().next());
                    if let Some(target) = target {
                        self.deliver_window_event_now(
                            target,
                            &Event::ImageUpdateDropped {
                                token,
                                image,
                                reason: fret_core::ImageUpdateDropReason::UnknownImage,
                            },
                        );
                    }
                }
                return true;
            }
            entry.alpha_mode = fret_core::AlphaMode::Opaque;
        }

        if entry
            .nv12_planes
            .as_ref()
            .is_none_or(|p| p.size != (width, height))
        {
            entry.nv12_planes = Some(yuv_gpu::Nv12Planes::new(&context.device, (width, height)));
        }
        if self.nv12_gpu.is_none() {
            self.nv12_gpu = Some(yuv_gpu::Nv12GpuConverter::new(&context.device));
        }

        let Some(planes) = entry.nv12_planes.as_ref() else {
            return false;
        };
        let Some(converter) = self.nv12_gpu.as_ref() else {
            return false;
        };

        let t0 = std::time::Instant::now();
        let Ok(uploaded_bytes) = yuv_gpu::write_nv12_rect(
            &context.queue,
            planes,
            rect,
            y_bytes_per_row,
            y_plane,
            uv_bytes_per_row,
            uv_plane,
        ) else {
            if self.config.streaming_update_ack_enabled {
                let target = window
                    .or(self.main_window)
                    .or_else(|| self.windows.keys().next());
                if let Some(target) = target {
                    self.deliver_window_event_now(
                        target,
                        &Event::ImageUpdateDropped {
                            token,
                            image,
                            reason: fret_core::ImageUpdateDropReason::InvalidPayload,
                        },
                    );
                }
            }
            return true;
        };

        stats.upload_bytes_applied = stats.upload_bytes_applied.saturating_add(uploaded_bytes);

        converter.convert_rect_into(yuv_gpu::Nv12ConvertRectIntoArgs {
            device: &context.device,
            queue: &context.queue,
            dst_view: &entry.uploaded.view,
            rect,
            y_view: &planes.y_view,
            uv_view: &planes.uv_view,
            range: color_info.range,
            matrix: color_info.matrix,
        });

        stats.yuv_conversions_applied = stats.yuv_conversions_applied.saturating_add(1);
        stats.yuv_convert_us = stats
            .yuv_convert_us
            .saturating_add(t0.elapsed().as_micros() as u64);
        stats.yuv_convert_output_bytes = stats
            .yuv_convert_output_bytes
            .saturating_add(rect.w.saturating_mul(rect.h).saturating_mul(4) as u64);

        if self.config.streaming_update_ack_enabled {
            let target = window
                .or(self.main_window)
                .or_else(|| self.windows.keys().next());
            if let Some(target) = target {
                self.deliver_window_event_now(target, &Event::ImageUpdateApplied { token, image });
            }
        }

        if let Some(state) = window.and_then(|w| self.windows.get(w)) {
            state.window.request_redraw();
        } else {
            for (_id, state) in self.windows.iter() {
                state.window.request_redraw();
            }
        }

        true
    }
}
