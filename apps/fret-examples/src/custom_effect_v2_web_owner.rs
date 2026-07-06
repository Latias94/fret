use std::sync::Arc;

use fret_runtime::{Model, ModelStore};

struct CustomEffectV2WebModelOwner<'a> {
    models: &'a mut ModelStore,
}

impl<'a> CustomEffectV2WebModelOwner<'a> {
    fn new(models: &'a mut ModelStore) -> Self {
        Self { models }
    }

    fn set_model<T: std::any::Any>(&mut self, model: &Model<T>, value: T) -> bool {
        self.models
            .update(model, |current| {
                *current = value;
                true
            })
            .unwrap_or(false)
    }

    fn toggle_surface(&mut self, binding: &CustomEffectV2WebControlBinding) -> bool {
        self.models
            .update(binding.show(), |v| {
                *v = !*v;
                true
            })
            .unwrap_or(false)
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CustomEffectV2WebCommonDefaults {
    visible: bool,
    enabled: bool,
    mode: &'static str,
    quality: &'static str,
    sampling: &'static str,
    uv_span: f32,
    debug_input: bool,
}

impl Default for CustomEffectV2WebCommonDefaults {
    fn default() -> Self {
        Self {
            visible: true,
            enabled: true,
            mode: "backdrop",
            quality: "high",
            sampling: "linear",
            uv_span: 1.0,
            debug_input: false,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CustomEffectV2WebControlBinding {
    show: Model<bool>,
    common: CustomEffectV2WebCommonControls,
    defaults: CustomEffectV2WebCommonDefaults,
}

impl CustomEffectV2WebControlBinding {
    pub(crate) fn new(models: &mut ModelStore) -> Self {
        Self::with_defaults(models, CustomEffectV2WebCommonDefaults::default())
    }

    pub(crate) fn with_defaults(
        models: &mut ModelStore,
        defaults: CustomEffectV2WebCommonDefaults,
    ) -> Self {
        Self {
            show: models.insert(defaults.visible),
            common: CustomEffectV2WebCommonControls::new(models, defaults),
            defaults,
        }
    }

    pub(crate) fn show(&self) -> &Model<bool> {
        &self.show
    }

    pub(crate) fn enabled(&self) -> &Model<bool> {
        self.common.enabled()
    }

    pub(crate) fn mode(&self) -> &Model<Option<Arc<str>>> {
        self.common.mode()
    }

    pub(crate) fn mode_open(&self) -> &Model<bool> {
        self.common.mode_open()
    }

    pub(crate) fn quality(&self) -> &Model<Option<Arc<str>>> {
        self.common.quality()
    }

    pub(crate) fn quality_open(&self) -> &Model<bool> {
        self.common.quality_open()
    }

    pub(crate) fn sampling(&self) -> &Model<Option<Arc<str>>> {
        self.common.sampling()
    }

    pub(crate) fn sampling_open(&self) -> &Model<bool> {
        self.common.sampling_open()
    }

    pub(crate) fn uv_span(&self) -> &Model<Vec<f32>> {
        self.common.uv_span()
    }

    pub(crate) fn debug_input(&self) -> &Model<bool> {
        self.common.debug_input()
    }

    pub(crate) fn toggle_surface_in(&self, models: &mut ModelStore) -> bool {
        CustomEffectV2WebModelOwner::new(models).toggle_surface(self)
    }

    pub(crate) fn reset_controls_in<C: CustomEffectV2WebVariantControls>(
        &self,
        models: &mut ModelStore,
        controls: &C,
    ) -> bool {
        let mut owner = CustomEffectV2WebModelOwner::new(models);
        let mut changed = self.reset_common_controls(&mut owner);
        let mut reset = CustomEffectV2WebVariantReset { owner: &mut owner };
        changed = controls.reset_variant_controls(&mut reset) || changed;
        changed
    }

    fn reset_common_controls(&self, owner: &mut CustomEffectV2WebModelOwner<'_>) -> bool {
        self.common.reset(owner, self.defaults)
    }
}

#[derive(Debug, Clone)]
struct CustomEffectV2WebCommonControls {
    enabled: Model<bool>,
    mode: Model<Option<Arc<str>>>,
    mode_open: Model<bool>,
    quality: Model<Option<Arc<str>>>,
    quality_open: Model<bool>,
    sampling: Model<Option<Arc<str>>>,
    sampling_open: Model<bool>,
    uv_span: Model<Vec<f32>>,
    debug_input: Model<bool>,
}

impl CustomEffectV2WebCommonControls {
    fn new(models: &mut ModelStore, defaults: CustomEffectV2WebCommonDefaults) -> Self {
        Self {
            enabled: models.insert(defaults.enabled),
            mode: models.insert(Some(Arc::from(defaults.mode))),
            mode_open: models.insert(false),
            quality: models.insert(Some(Arc::from(defaults.quality))),
            quality_open: models.insert(false),
            sampling: models.insert(Some(Arc::from(defaults.sampling))),
            sampling_open: models.insert(false),
            uv_span: models.insert(vec![defaults.uv_span]),
            debug_input: models.insert(defaults.debug_input),
        }
    }

    fn enabled(&self) -> &Model<bool> {
        &self.enabled
    }

    fn mode(&self) -> &Model<Option<Arc<str>>> {
        &self.mode
    }

    fn mode_open(&self) -> &Model<bool> {
        &self.mode_open
    }

    fn quality(&self) -> &Model<Option<Arc<str>>> {
        &self.quality
    }

    fn quality_open(&self) -> &Model<bool> {
        &self.quality_open
    }

    fn sampling(&self) -> &Model<Option<Arc<str>>> {
        &self.sampling
    }

    fn sampling_open(&self) -> &Model<bool> {
        &self.sampling_open
    }

    fn uv_span(&self) -> &Model<Vec<f32>> {
        &self.uv_span
    }

    fn debug_input(&self) -> &Model<bool> {
        &self.debug_input
    }

    fn reset(
        &self,
        owner: &mut CustomEffectV2WebModelOwner<'_>,
        defaults: CustomEffectV2WebCommonDefaults,
    ) -> bool {
        let mut changed = false;
        changed = owner.set_model(&self.enabled, defaults.enabled) || changed;
        changed = owner.set_model(&self.mode, Some(Arc::from(defaults.mode))) || changed;
        changed = owner.set_model(&self.quality, Some(Arc::from(defaults.quality))) || changed;
        changed = owner.set_model(&self.sampling, Some(Arc::from(defaults.sampling))) || changed;
        changed = owner.set_model(&self.uv_span, vec![defaults.uv_span]) || changed;
        owner.set_model(&self.debug_input, defaults.debug_input) || changed
    }
}

pub(crate) struct CustomEffectV2WebVariantReset<'a, 'models> {
    owner: &'a mut CustomEffectV2WebModelOwner<'models>,
}

impl<'a, 'models> CustomEffectV2WebVariantReset<'a, 'models> {
    pub(crate) fn set_model<T: std::any::Any>(&mut self, model: &Model<T>, value: T) -> bool {
        self.owner.set_model(model, value)
    }
}

pub(crate) trait CustomEffectV2WebVariantControls {
    fn reset_variant_controls(&self, reset: &mut CustomEffectV2WebVariantReset<'_, '_>) -> bool;
}
