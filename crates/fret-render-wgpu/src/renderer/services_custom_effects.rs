use super::shaders::{
    custom_effect_mask_shader_source, custom_effect_masked_shader_source,
    custom_effect_unmasked_shader_source, custom_effect_v2_mask_shader_source,
    custom_effect_v2_masked_shader_source, custom_effect_v2_unmasked_shader_source,
    custom_effect_v3_mask_shader_source, custom_effect_v3_masked_shader_source,
    custom_effect_v3_unmasked_shader_source,
};
use super::*;
use std::sync::Arc;

const MAX_CUSTOM_EFFECT_WGSL_BYTES: usize = 64 * 1024;

fn validate_custom_effect_wgsl_triplet(
    abi: CustomEffectAbi,
    user_source: &str,
    wgsl_unmasked: &str,
    wgsl_masked: &str,
    wgsl_mask: &str,
) -> Result<(), fret_core::CustomEffectRegistrationError> {
    use naga::valid::{Capabilities, ValidationFlags, Validator};

    if user_source.len() > MAX_CUSTOM_EFFECT_WGSL_BYTES {
        return Err(fret_core::CustomEffectRegistrationError::InvalidSource);
    }

    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::empty());
    for (label, src) in [
        ("unmasked", wgsl_unmasked),
        ("masked", wgsl_masked),
        ("mask", wgsl_mask),
    ] {
        let module = naga::front::wgsl::parse_str(src).map_err(|err| {
            tracing::warn!(?err, %label, ?abi, "custom effect wgsl parse failed");
            fret_core::CustomEffectRegistrationError::InvalidSource
        })?;
        validator.validate(&module).map_err(|err| {
            tracing::warn!(?err, %label, ?abi, "custom effect wgsl validation failed");
            fret_core::CustomEffectRegistrationError::InvalidSource
        })?;
    }

    Ok(())
}

fn build_and_validate_custom_effect_wgsl_with_sources(
    abi: CustomEffectAbi,
    user_source: &str,
    unmasked_source: fn(&str) -> String,
    masked_source: fn(&str) -> String,
    mask_source: fn(&str) -> String,
) -> Result<(String, String, String), fret_core::CustomEffectRegistrationError> {
    if user_source.len() > MAX_CUSTOM_EFFECT_WGSL_BYTES {
        return Err(fret_core::CustomEffectRegistrationError::InvalidSource);
    }

    let wgsl_unmasked = unmasked_source(user_source);
    let wgsl_masked = masked_source(user_source);
    let wgsl_mask = mask_source(user_source);

    validate_custom_effect_wgsl_triplet(
        abi,
        user_source,
        &wgsl_unmasked,
        &wgsl_masked,
        &wgsl_mask,
    )?;

    Ok((wgsl_unmasked, wgsl_masked, wgsl_mask))
}

fn build_and_validate_custom_effect_wgsl_v1(
    user_source: &str,
) -> Result<(String, String, String), fret_core::CustomEffectRegistrationError> {
    build_and_validate_custom_effect_wgsl_with_sources(
        CustomEffectAbi::V1,
        user_source,
        custom_effect_unmasked_shader_source,
        custom_effect_masked_shader_source,
        custom_effect_mask_shader_source,
    )
}

fn build_and_validate_custom_effect_wgsl_v2(
    user_source: &str,
) -> Result<(String, String, String), fret_core::CustomEffectRegistrationError> {
    build_and_validate_custom_effect_wgsl_with_sources(
        CustomEffectAbi::V2,
        user_source,
        custom_effect_v2_unmasked_shader_source,
        custom_effect_v2_masked_shader_source,
        custom_effect_v2_mask_shader_source,
    )
}

fn build_and_validate_custom_effect_wgsl_v3(
    user_source: &str,
) -> Result<(String, String, String), fret_core::CustomEffectRegistrationError> {
    build_and_validate_custom_effect_wgsl_with_sources(
        CustomEffectAbi::V3,
        user_source,
        custom_effect_v3_unmasked_shader_source,
        custom_effect_v3_masked_shader_source,
        custom_effect_v3_mask_shader_source,
    )
}

fn custom_effect_sampled_user_image_supported(adapter: &wgpu::Adapter) -> bool {
    let fmt = wgpu::TextureFormat::Rgba8Unorm;
    let f = adapter.get_texture_format_features(fmt);
    f.allowed_usages
        .contains(wgpu::TextureUsages::TEXTURE_BINDING)
        && f.flags
            .contains(wgpu::TextureFormatFeatureFlags::FILTERABLE)
}

type BuildAndValidateFn = fn(
    &str,
) -> Result<(String, String, String), fret_core::CustomEffectRegistrationError>;

fn register_custom_effect_wgsl(
    renderer: &mut Renderer,
    abi: CustomEffectAbi,
    user_source: String,
    build_and_validate: BuildAndValidateFn,
) -> Result<fret_core::EffectId, fret_core::CustomEffectRegistrationError> {
    if user_source.len() > MAX_CUSTOM_EFFECT_WGSL_BYTES {
        return Err(fret_core::CustomEffectRegistrationError::InvalidSource);
    }

    if let Some(id) = renderer
        .material_effect_state
        .find_custom_effect(abi, &user_source)
    {
        let retained = renderer.material_effect_state.retain_custom_effect(id);
        debug_assert!(
            retained,
            "custom-effect registry lookup should not return a missing id"
        );
        return Ok(id);
    }

    let (wgsl_unmasked, wgsl_masked, wgsl_mask) = build_and_validate(&user_source)?;
    Ok(renderer
        .material_effect_state
        .insert_custom_effect(CustomEffectEntry {
            abi,
            raw_source: Arc::from(user_source),
            wgsl_unmasked: Arc::from(wgsl_unmasked),
            wgsl_masked: Arc::from(wgsl_masked),
            wgsl_mask: Arc::from(wgsl_mask),
            refs: 1,
        }))
}

impl fret_core::CustomEffectService for Renderer {
    fn register_custom_effect_v1(
        &mut self,
        desc: fret_core::CustomEffectDescriptorV1,
    ) -> Result<fret_core::EffectId, fret_core::CustomEffectRegistrationError> {
        if desc.language != fret_core::CustomEffectProgramLanguage::WgslUtf8 {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }
        register_custom_effect_wgsl(
            self,
            CustomEffectAbi::V1,
            desc.source,
            build_and_validate_custom_effect_wgsl_v1,
        )
    }

    fn register_custom_effect_v2(
        &mut self,
        desc: fret_core::CustomEffectDescriptorV2,
    ) -> Result<fret_core::EffectId, fret_core::CustomEffectRegistrationError> {
        if desc.language != fret_core::CustomEffectProgramLanguage::WgslUtf8 {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }

        // V2 requires a filterable sampled input texture (the user image input). Gate the feature
        // on a conservative and widely supported format so apps can reliably probe support.
        if !custom_effect_sampled_user_image_supported(&self.adapter) {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }
        register_custom_effect_wgsl(
            self,
            CustomEffectAbi::V2,
            desc.source,
            build_and_validate_custom_effect_wgsl_v2,
        )
    }

    fn register_custom_effect_v3(
        &mut self,
        desc: fret_core::CustomEffectDescriptorV3,
    ) -> Result<fret_core::EffectId, fret_core::CustomEffectRegistrationError> {
        if desc.language != fret_core::CustomEffectProgramLanguage::WgslUtf8 {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }

        // V3 currently keeps the same conservative requirement as V2 for user image inputs:
        // a filterable sampled texture (RGBA8). Backends that cannot provide it should report
        // `Unsupported` deterministically.
        if !custom_effect_sampled_user_image_supported(&self.adapter) {
            return Err(fret_core::CustomEffectRegistrationError::Unsupported);
        }
        register_custom_effect_wgsl(
            self,
            CustomEffectAbi::V3,
            desc.source,
            build_and_validate_custom_effect_wgsl_v3,
        )
    }

    fn unregister_custom_effect(&mut self, id: fret_core::EffectId) -> bool {
        match self.material_effect_state.unregister_custom_effect(id) {
            CustomEffectUnregisterOutcome::Missing => false,
            CustomEffectUnregisterOutcome::StillReferenced => true,
            CustomEffectUnregisterOutcome::Removed => {
                self.pipelines.evict_custom_effect_pipelines(id);
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fret_core::CustomEffectService as _;

    #[test]
    fn custom_effect_wgsl_size_limit_is_enforced() {
        let src = "x".repeat(MAX_CUSTOM_EFFECT_WGSL_BYTES + 1);
        assert!(matches!(
            build_and_validate_custom_effect_wgsl_v1(&src),
            Err(fret_core::CustomEffectRegistrationError::InvalidSource)
        ));
    }

    #[test]
    fn custom_effect_wgsl_empty_is_rejected() {
        assert!(matches!(
            build_and_validate_custom_effect_wgsl_v1(""),
            Err(fret_core::CustomEffectRegistrationError::InvalidSource)
        ));
    }

    #[test]
    fn custom_effect_wgsl_minimal_program_validates() {
        let src = r#"
 fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
   return src;
 }
 "#;
        build_and_validate_custom_effect_wgsl_v1(src).expect("expected valid custom effect WGSL");
    }

    #[test]
    fn custom_effect_v2_wgsl_minimal_program_validates() {
        let src = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;
        build_and_validate_custom_effect_wgsl_v2(src)
            .expect("expected valid custom effect v2 WGSL");
    }

    #[test]
    fn custom_effect_v3_wgsl_minimal_program_validates() {
        let src = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;
        build_and_validate_custom_effect_wgsl_v3(src)
            .expect("expected valid custom effect v3 WGSL");
    }

    #[test]
    fn unregister_custom_effect_evicts_custom_effect_pipelines() {
        let ctx = match pollster::block_on(crate::WgpuContext::new()) {
            Ok(ctx) => ctx,
            Err(_err) => return,
        };
        let mut renderer = crate::Renderer::new(&ctx.adapter, &ctx.device);

        let src = r#"
fn fret_custom_effect(src: vec4<f32>, _uv: vec2<f32>, _pos_px: vec2<f32>, _params: EffectParamsV1) -> vec4<f32> {
  return src;
}
"#;

        let id_v1 = renderer
            .register_custom_effect_v1(fret_core::CustomEffectDescriptorV1::wgsl_utf8(src))
            .expect("custom effect v1 registration must succeed on wgpu backends");
        let id_v2 = renderer
            .register_custom_effect_v2(fret_core::CustomEffectDescriptorV2::wgsl_utf8(src))
            .expect("custom effect v2 registration must succeed on wgpu backends");
        let id_v3 = renderer
            .register_custom_effect_v3(fret_core::CustomEffectDescriptorV3::wgsl_utf8(src))
            .expect("custom effect v3 registration must succeed on wgpu backends");

        renderer.ensure_custom_effect_pipelines(
            &ctx.device,
            wgpu::TextureFormat::Rgba8Unorm,
            id_v1,
        );
        assert!(
            renderer
                .pipelines
                .custom_effect_pipelines
                .contains_key(&id_v1),
            "expected v1 pipelines to be cached for the effect"
        );
        assert!(renderer.unregister_custom_effect(id_v1));
        assert!(
            !renderer
                .pipelines
                .custom_effect_pipelines
                .contains_key(&id_v1),
            "expected v1 pipelines to be evicted when the effect is unregistered"
        );

        renderer.ensure_custom_effect_v2_pipelines(
            &ctx.device,
            wgpu::TextureFormat::Rgba8Unorm,
            id_v2,
        );
        assert!(
            renderer
                .pipelines
                .custom_effect_v2_pipelines
                .contains_key(&id_v2),
            "expected v2 pipelines to be cached for the effect"
        );
        assert!(renderer.unregister_custom_effect(id_v2));
        assert!(
            !renderer
                .pipelines
                .custom_effect_v2_pipelines
                .contains_key(&id_v2),
            "expected v2 pipelines to be evicted when the effect is unregistered"
        );

        renderer.ensure_custom_effect_v3_pipelines(
            &ctx.device,
            wgpu::TextureFormat::Rgba8Unorm,
            id_v3,
        );
        assert!(
            renderer
                .pipelines
                .custom_effect_v3_pipelines
                .contains_key(&id_v3),
            "expected v3 pipelines to be cached for the effect"
        );
        assert!(renderer.unregister_custom_effect(id_v3));
        assert!(
            !renderer
                .pipelines
                .custom_effect_v3_pipelines
                .contains_key(&id_v3),
            "expected v3 pipelines to be evicted when the effect is unregistered"
        );
    }
}
