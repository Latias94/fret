use std::sync::Arc;

use fret_core::{SvgId, UiServices};

#[derive(Debug, Clone)]
pub enum SvgSource {
    Id(SvgId),
    Static(&'static [u8]),
    Bytes(Arc<[u8]>),
}

impl SvgSource {
    pub fn resolve(&self, services: &mut dyn UiServices) -> SvgId {
        match self {
            SvgSource::Id(id) => *id,
            SvgSource::Static(bytes) => services.svg().register_svg(bytes),
            SvgSource::Bytes(bytes) => services.svg().register_svg(bytes),
        }
    }
}
