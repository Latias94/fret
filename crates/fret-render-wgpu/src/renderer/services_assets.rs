use super::*;

impl fret_core::SvgService for Renderer {
    fn register_svg(&mut self, bytes: &[u8]) -> fret_core::SvgId {
        self.svg_registry_state.register_svg(bytes)
    }

    fn unregister_svg(&mut self, svg: fret_core::SvgId) -> bool {
        match self.svg_registry_state.unregister_svg(svg) {
            svg::SvgRegistryUnregisterOutcome::Missing => false,
            svg::SvgRegistryUnregisterOutcome::StillReferenced => true,
            svg::SvgRegistryUnregisterOutcome::Removed => {
                self.unregister_svg_rasters(svg);
                true
            }
        }
    }
}

fn catalog_texture_material_supported(
    allowed_usages: wgpu::TextureUsages,
    flags: wgpu::TextureFormatFeatureFlags,
) -> bool {
    let ok_usages = allowed_usages
        .contains(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST);
    let ok_filterable = flags.contains(wgpu::TextureFormatFeatureFlags::FILTERABLE);
    ok_usages && ok_filterable
}

impl fret_core::MaterialService for Renderer {
    fn register_material(
        &mut self,
        desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        // v1: all baseline material kinds are supported by the quad shader on wgpu backends.
        //
        // v2: sampled materials are capability-gated by a fixed, renderer-owned catalog texture
        // binding shape (ADR 0242).
        match desc.binding {
            fret_core::MaterialBindingShape::ParamsOnly => {}
            fret_core::MaterialBindingShape::ParamsPlusCatalogTexture { .. } => {
                let features = self
                    .adapter
                    .get_texture_format_features(wgpu::TextureFormat::Rgba8Unorm);
                if !catalog_texture_material_supported(features.allowed_usages, features.flags) {
                    return Err(fret_core::MaterialRegistrationError::Unsupported);
                }
            }
        }

        Ok(self.material_effect_state.register_material(desc))
    }

    fn unregister_material(&mut self, id: fret_core::MaterialId) -> bool {
        self.material_effect_state.unregister_material(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sampled_material_registration_is_capability_gated() {
        let required_usages = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
        let required_flags = wgpu::TextureFormatFeatureFlags::FILTERABLE;

        assert!(!catalog_texture_material_supported(
            wgpu::TextureUsages::COPY_DST,
            required_flags
        ));
        assert!(!catalog_texture_material_supported(
            wgpu::TextureUsages::TEXTURE_BINDING,
            required_flags
        ));
        assert!(!catalog_texture_material_supported(
            required_usages,
            wgpu::TextureFormatFeatureFlags::empty()
        ));
        assert!(catalog_texture_material_supported(
            required_usages,
            required_flags
        ));
    }
}
