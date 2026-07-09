use fret_core::time::Instant;
use fret_core::{
    AlphaMode, AppWindowId, Event, ImageColorInfo, ImageEncoding, ImageId, ImageUpdateDropReason,
    ImageUpdateToken, ImageUploadToken, RectPx,
};
use fret_render::{ImageColorSpace, ImageDescriptor, upload_rgba8_image};

use crate::runner::streaming_upload::StreamingUploadStats;

use super::streaming_images::{
    StreamingImageUpdateNv12, StreamingImageUpdateRgba8, UploadedImageEntry,
};
use super::{WinitAppDriver, WinitRunner};

impl<D: WinitAppDriver> WinitRunner<D> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn handle_image_register_rgba8(
        &mut self,
        window: AppWindowId,
        token: ImageUploadToken,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        color_info: ImageColorInfo,
        alpha_mode: AlphaMode,
    ) {
        let Some(context) = self.context.as_ref() else {
            self.deliver_window_event_now(
                window,
                &Event::ImageRegisterFailed {
                    token,
                    message: "wgpu not initialized".to_string(),
                },
            );
            return;
        };
        let Some(renderer) = self.renderer.as_mut() else {
            self.deliver_window_event_now(
                window,
                &Event::ImageRegisterFailed {
                    token,
                    message: "renderer not initialized".to_string(),
                },
            );
            return;
        };

        if width == 0 || height == 0 {
            self.deliver_window_event_now(
                window,
                &Event::ImageRegisterFailed {
                    token,
                    message: format!("invalid image size: {width}x{height}"),
                },
            );
            return;
        }

        let expected_len = (width as usize)
            .saturating_mul(height as usize)
            .saturating_mul(4);
        if bytes.len() != expected_len {
            self.deliver_window_event_now(
                window,
                &Event::ImageRegisterFailed {
                    token,
                    message: format!(
                        "invalid rgba8 byte length: got {} expected {}",
                        bytes.len(),
                        expected_len
                    ),
                },
            );
            return;
        }

        let color_space = match color_info.encoding {
            ImageEncoding::Srgb => ImageColorSpace::Srgb,
            ImageEncoding::Linear => ImageColorSpace::Linear,
        };

        let uploaded = upload_rgba8_image(
            &context.device,
            &context.queue,
            (width, height),
            &bytes,
            color_space,
        );

        let view = uploaded
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let image = renderer.register_image(ImageDescriptor {
            view,
            size: uploaded.size,
            format: uploaded.format,
            color_space: uploaded.color_space,
            alpha_mode,
        });
        self.uploaded_images.insert(
            image,
            UploadedImageEntry {
                uploaded,
                stream_generation: 0,
                alpha_mode,
                nv12_planes: None,
            },
        );

        self.deliver_window_event_now(
            window,
            &Event::ImageRegistered {
                token,
                image,
                width,
                height,
            },
        );
        if let Some(state) = self.windows.get(window) {
            state.window.request_redraw();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn handle_image_update_rgba8(
        &mut self,
        stats: &mut StreamingUploadStats,
        window: Option<AppWindowId>,
        token: ImageUpdateToken,
        image: ImageId,
        stream_generation: u64,
        width: u32,
        height: u32,
        update_rect_px: Option<RectPx>,
        bytes_per_row: u32,
        bytes: Vec<u8>,
        color_info: ImageColorInfo,
        alpha_mode: AlphaMode,
    ) {
        self.apply_streaming_image_update_rgba8(
            stats,
            StreamingImageUpdateRgba8 {
                window,
                token,
                image,
                stream_generation,
                width,
                height,
                update_rect_px,
                bytes_per_row,
                bytes: &bytes,
                color_info,
                alpha_mode,
            },
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn handle_image_update_nv12(
        &mut self,
        stats: &mut StreamingUploadStats,
        window: Option<AppWindowId>,
        token: ImageUpdateToken,
        image: ImageId,
        stream_generation: u64,
        width: u32,
        height: u32,
        update_rect_px: Option<RectPx>,
        y_bytes_per_row: u32,
        y_plane: Vec<u8>,
        uv_bytes_per_row: u32,
        uv_plane: Vec<u8>,
        color_info: ImageColorInfo,
    ) {
        stats.yuv_conversions_attempted = stats.yuv_conversions_attempted.saturating_add(1);
        if self.try_apply_streaming_image_update_nv12_gpu(
            stats,
            StreamingImageUpdateNv12 {
                window,
                token,
                image,
                stream_generation,
                width,
                height,
                update_rect_px,
                y_bytes_per_row,
                y_plane: &y_plane,
                uv_bytes_per_row,
                uv_plane: &uv_plane,
                color_info,
            },
        ) {
            return;
        }

        let t0 = Instant::now();
        match crate::runner::yuv::nv12_to_rgba8_rect(crate::runner::yuv::Nv12ToRgba8RectInput {
            width,
            height,
            update_rect_px,
            y_bytes_per_row,
            y_plane: &y_plane,
            uv_bytes_per_row,
            uv_plane: &uv_plane,
            range: color_info.range,
            matrix: color_info.matrix,
        }) {
            Ok((rect, rgba)) => {
                stats.yuv_conversions_applied = stats.yuv_conversions_applied.saturating_add(1);
                stats.yuv_convert_us = stats
                    .yuv_convert_us
                    .saturating_add(t0.elapsed().as_micros() as u64);
                stats.yuv_convert_output_bytes = stats
                    .yuv_convert_output_bytes
                    .saturating_add(rgba.len() as u64);

                self.apply_streaming_image_update_rgba8(
                    stats,
                    StreamingImageUpdateRgba8 {
                        window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px: Some(rect),
                        bytes_per_row: rect.w.saturating_mul(4),
                        bytes: &rgba,
                        color_info: fret_core::ImageColorInfo::srgb_rgba(),
                        alpha_mode: AlphaMode::Opaque,
                    },
                );
            }
            Err(_message) => {
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
                                reason: ImageUpdateDropReason::InvalidPayload,
                            },
                        );
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn handle_image_update_i420(
        &mut self,
        stats: &mut StreamingUploadStats,
        window: Option<AppWindowId>,
        token: ImageUpdateToken,
        image: ImageId,
        stream_generation: u64,
        width: u32,
        height: u32,
        update_rect_px: Option<RectPx>,
        y_bytes_per_row: u32,
        y_plane: Vec<u8>,
        u_bytes_per_row: u32,
        u_plane: Vec<u8>,
        v_bytes_per_row: u32,
        v_plane: Vec<u8>,
        color_info: ImageColorInfo,
    ) {
        stats.yuv_conversions_attempted = stats.yuv_conversions_attempted.saturating_add(1);
        let t0 = Instant::now();
        match crate::runner::yuv::i420_to_rgba8_rect(crate::runner::yuv::I420ToRgba8RectInput {
            width,
            height,
            update_rect_px,
            y_bytes_per_row,
            y_plane: &y_plane,
            u_bytes_per_row,
            u_plane: &u_plane,
            v_bytes_per_row,
            v_plane: &v_plane,
            range: color_info.range,
            matrix: color_info.matrix,
        }) {
            Ok((rect, rgba)) => {
                stats.yuv_conversions_applied = stats.yuv_conversions_applied.saturating_add(1);
                stats.yuv_convert_us = stats
                    .yuv_convert_us
                    .saturating_add(t0.elapsed().as_micros() as u64);
                stats.yuv_convert_output_bytes = stats
                    .yuv_convert_output_bytes
                    .saturating_add(rgba.len() as u64);

                self.apply_streaming_image_update_rgba8(
                    stats,
                    StreamingImageUpdateRgba8 {
                        window,
                        token,
                        image,
                        stream_generation,
                        width,
                        height,
                        update_rect_px: Some(rect),
                        bytes_per_row: rect.w.saturating_mul(4),
                        bytes: &rgba,
                        color_info: fret_core::ImageColorInfo::srgb_rgba(),
                        alpha_mode: AlphaMode::Opaque,
                    },
                );
            }
            Err(_message) => {
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
                                reason: ImageUpdateDropReason::InvalidPayload,
                            },
                        );
                    }
                }
            }
        }
    }

    pub(super) fn handle_image_unregister(&mut self, image: ImageId) {
        let Some(renderer) = self.renderer.as_mut() else {
            return;
        };

        self.uploaded_images.remove(&image);

        if !renderer.unregister_image(image) {
            return;
        }

        for (_id, state) in self.windows.iter() {
            state.window.request_redraw();
        }
    }
}
