use glam::Vec3;

use super::HandleId;

pub(crate) const HANDLE_GROUP_SHIFT: u32 = 16;
pub(crate) const HANDLE_GROUP_TRANSLATE: u32 = 1;
pub(crate) const HANDLE_GROUP_ROTATE: u32 = 2;
pub(crate) const HANDLE_GROUP_SCALE: u32 = 3;

pub(crate) const fn pack_handle(group: u32, id: u32) -> HandleId {
    let group = group & 0xFFFF;
    let id = id & 0xFFFF;
    HandleId(((group << HANDLE_GROUP_SHIFT) | id) as u64)
}

pub(crate) const fn handle_group(handle: HandleId) -> u32 {
    handle.local() >> HANDLE_GROUP_SHIFT
}

pub(crate) const fn handle_sub_id(handle: HandleId) -> u32 {
    handle.local() & 0xFFFF
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TranslateHandle {
    AxisX,
    AxisY,
    AxisZ,
    PlaneXY,
    PlaneXZ,
    PlaneYZ,
    Screen,
    Depth,
}

impl TranslateHandle {
    pub(super) fn id(self) -> HandleId {
        pack_handle(
            HANDLE_GROUP_TRANSLATE,
            match self {
                TranslateHandle::AxisX => 1,
                TranslateHandle::AxisY => 2,
                TranslateHandle::AxisZ => 3,
                TranslateHandle::PlaneXY => 4,
                TranslateHandle::PlaneXZ => 5,
                TranslateHandle::PlaneYZ => 6,
                TranslateHandle::Screen => 10,
                TranslateHandle::Depth => 11,
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BoundsHandle {
    Corner {
        x_max: bool,
        y_max: bool,
        z_max: bool,
    },
    Face {
        axis: usize,
        max_side: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScaleHandle {
    AxisX,
    AxisY,
    AxisZ,
    PlaneXY,
    PlaneXZ,
    PlaneYZ,
    Uniform,
}

impl ScaleHandle {
    pub(super) fn id(self) -> HandleId {
        pack_handle(
            HANDLE_GROUP_SCALE,
            match self {
                ScaleHandle::AxisX => 1,
                ScaleHandle::AxisY => 2,
                ScaleHandle::AxisZ => 3,
                ScaleHandle::Uniform => 7,
                // Keep plane-scale handle IDs disjoint from translation plane IDs (4/5/6) so Universal
                // can include axis scale without fighting translate planes.
                ScaleHandle::PlaneXY => 14,
                ScaleHandle::PlaneXZ => 15,
                ScaleHandle::PlaneYZ => 16,
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RotateHandle {
    AxisX,
    AxisY,
    AxisZ,
    View,
    Arcball,
}

impl RotateHandle {
    pub(super) fn id(self) -> HandleId {
        pack_handle(
            HANDLE_GROUP_ROTATE,
            match self {
                RotateHandle::AxisX => 1,
                RotateHandle::AxisY => 2,
                RotateHandle::AxisZ => 3,
                RotateHandle::View => 8,
                RotateHandle::Arcball => 9,
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum TranslateConstraint {
    Axis { axis_dir: Vec3 },
    Plane { u: Vec3, v: Vec3, normal: Vec3 },
    Dolly { view_dir: Vec3 },
}
