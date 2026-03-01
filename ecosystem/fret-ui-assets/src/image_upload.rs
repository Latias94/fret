use std::collections::HashMap;

use fret_core::{AlphaMode, AppWindowId, Event, ImageColorSpace, ImageId, ImageUploadToken};
use fret_runtime::{Effect, EffectSink, GlobalsHost, TimeHost};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageMeta {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default)]
pub struct ImageUploadService {
    pending: HashMap<ImageUploadToken, ImageMeta>,
    ready: HashMap<ImageUploadToken, (ImageId, ImageMeta)>,
    failed: HashMap<ImageUploadToken, String>,
    by_image: HashMap<ImageId, (ImageUploadToken, ImageMeta)>,
}

impl ImageUploadService {
    pub fn request_rgba8<H: TimeHost + EffectSink>(
        &mut self,
        host: &mut H,
        window: AppWindowId,
        width: u32,
        height: u32,
        bytes: Vec<u8>,
        color_space: ImageColorSpace,
    ) -> ImageUploadToken {
        let token = host.next_image_upload_token();
        let meta = ImageMeta { width, height };

        match expected_rgba8_len(width, height) {
            Some(expected) if expected == bytes.len() => {
                self.pending.insert(token, meta);
                self.failed.remove(&token);

                host.push_effect(Effect::ImageRegisterRgba8 {
                    window,
                    token,
                    width,
                    height,
                    bytes,
                    color_info: color_space.to_color_info(),
                    alpha_mode: AlphaMode::Straight,
                });
                host.request_redraw(window);
            }
            Some(expected) => {
                self.failed.insert(
                    token,
                    format!(
                        "invalid rgba8 byte length: expected {expected} for {width}x{height}, got {}",
                        bytes.len()
                    ),
                );
            }
            None => {
                self.failed
                    .insert(token, "invalid rgba8 dimensions (overflow)".to_string());
            }
        }

        token
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::ImageRegistered {
                token,
                image,
                width,
                height,
            } => {
                let meta = ImageMeta {
                    width: *width,
                    height: *height,
                };
                self.pending.remove(token);
                self.failed.remove(token);
                self.ready.insert(*token, (*image, meta));
                self.by_image.insert(*image, (*token, meta));
                true
            }
            Event::ImageRegisterFailed { token, message } => {
                self.pending.remove(token);
                self.failed.insert(*token, message.clone());
                true
            }
            _ => false,
        }
    }

    pub fn image(&self, token: ImageUploadToken) -> Option<ImageId> {
        self.ready.get(&token).map(|(id, _)| *id)
    }

    pub fn image_meta(&self, token: ImageUploadToken) -> Option<ImageMeta> {
        self.ready.get(&token).map(|(_, meta)| *meta)
    }

    pub fn error(&self, token: ImageUploadToken) -> Option<&str> {
        self.failed.get(&token).map(String::as_str)
    }

    pub fn is_pending(&self, token: ImageUploadToken) -> bool {
        self.pending.contains_key(&token)
    }

    pub fn release_image<H: EffectSink>(&mut self, host: &mut H, image: ImageId) -> bool {
        let Some((token, _meta)) = self.by_image.remove(&image) else {
            return false;
        };
        self.ready.remove(&token);
        host.push_effect(Effect::ImageUnregister { image });
        true
    }

    pub fn release_token<H: EffectSink>(&mut self, host: &mut H, token: ImageUploadToken) -> bool {
        if self.pending.remove(&token).is_some() || self.failed.remove(&token).is_some() {
            return true;
        }
        let Some((image, _meta)) = self.ready.remove(&token) else {
            return false;
        };
        self.by_image.remove(&image);
        host.push_effect(Effect::ImageUnregister { image });
        true
    }
}

pub trait ImageUploadServiceHostExt: GlobalsHost {
    fn with_image_upload_service<R>(
        &mut self,
        f: impl FnOnce(&mut ImageUploadService, &mut Self) -> R,
    ) -> R {
        self.with_global_mut(ImageUploadService::default, f)
    }
}

impl<H: GlobalsHost> ImageUploadServiceHostExt for H {}

fn expected_rgba8_len(width: u32, height: u32) -> Option<usize> {
    let pixels = (width as u64).checked_mul(height as u64)?;
    let bytes = pixels.checked_mul(4)?;
    usize::try_from(bytes).ok()
}
