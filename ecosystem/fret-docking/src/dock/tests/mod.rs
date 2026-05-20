use super::ViewportPanel;
use super::hit_test::hit_test_split_handle;
use super::host_frame::{DockSpaceLayoutSnapshot, panel_root_placements_for_snapshot};
use super::layout::{
    active_panel_content_bounds, compute_layout_map, dock_hint_rects_with_font, dock_space_regions,
    split_tab_bar,
};
use super::prelude_core::*;
use super::prelude_runtime::*;
use super::split_geometry;
use super::{
    DockManager, DockPanel, DockPanelContentService, DockPanelElementRegistry,
    DockPanelElementRegistryService, DockSpaceElementOptions, DockingPolicy, DockingPolicyService,
    dock_panel_element, dock_space_element, dock_space_element_from_registry,
};
use crate::test_host::TestHost;
use fret_core::{
    AppWindowId, Event, InternalDragEvent, InternalDragKind, Modifiers, Point, Px, Scene, SceneOp,
    Size, TextBlobId, TextConstraints, TextInput, TextMetrics, TextService,
};
use fret_runtime::PlatformCapabilities;
use fret_runtime::{DRAG_KIND_DOCK_PANEL, DRAG_KIND_DOCK_TABS};
use fret_ui::UiTree;
use fret_ui::element::{
    AnchoredProps, ContainerProps, InsetEdge, LayoutStyle, PositionStyle, SemanticsProps, SizeStyle,
};
use fret_ui::overlay_placement::{Align, AnchoredPanelLayout, AnchoredPanelOptions, Side};
use std::sync::Arc;

use fret_ui::declarative;
use slotmap::KeyData;

mod dock_space;
mod split;

#[derive(Default)]
struct FakeTextService;

impl TextService for FakeTextService {
    fn prepare(
        &mut self,
        _input: &TextInput,
        _constraints: TextConstraints,
    ) -> (TextBlobId, TextMetrics) {
        (
            TextBlobId::default(),
            TextMetrics {
                size: Size::new(Px(240.0), Px(34.0)),
                baseline: Px(18.0),
            },
        )
    }

    fn release(&mut self, _blob: TextBlobId) {}
}

impl fret_core::PathService for FakeTextService {
    fn prepare(
        &mut self,
        _commands: &[fret_core::PathCommand],
        _style: fret_core::PathStyle,
        _constraints: fret_core::PathConstraints,
    ) -> (fret_core::PathId, fret_core::PathMetrics) {
        (
            fret_core::PathId::default(),
            fret_core::PathMetrics::default(),
        )
    }

    fn release(&mut self, _path: fret_core::PathId) {}
}

impl fret_core::SvgService for FakeTextService {
    fn register_svg(&mut self, _bytes: &[u8]) -> fret_core::SvgId {
        fret_core::SvgId::default()
    }

    fn unregister_svg(&mut self, _svg: fret_core::SvgId) -> bool {
        false
    }
}

impl fret_core::MaterialService for FakeTextService {
    fn register_material(
        &mut self,
        _desc: fret_core::MaterialDescriptor,
    ) -> Result<fret_core::MaterialId, fret_core::MaterialRegistrationError> {
        Err(fret_core::MaterialRegistrationError::Unsupported)
    }

    fn unregister_material(&mut self, _id: fret_core::MaterialId) -> bool {
        true
    }
}
