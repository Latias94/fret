pub(crate) use std::{
    any::{Any, TypeId},
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

pub(crate) use fret_core::{
    AppWindowId, CaretAffinity, Event, KeyCode, Modifiers, MouseButton, MouseButtons, Point,
    PointerId, PointerType, Px, Rect, SemanticsRole, Size, TextConstraints, TextMetrics,
    TextService,
};
pub(crate) use fret_runtime::{
    ClipboardToken, CommandRegistry, CommandsHost, DragHost, DragKindId, DragSession,
    DragSessionId, Effect, EffectSink, FrameId, GlobalsHost, KeyChord, ModelHost, ModelId,
    ModelStore, ModelsHost, PlatformCapabilities, ShareSheetToken, TickId, TimeHost, TimerToken,
};
pub(crate) use fret_ui::action::{DismissReason, DismissRequestCx, OnDismissRequest};
pub(crate) use fret_ui::declarative::render_root;
pub(crate) use fret_ui::element::Length;
pub(crate) use fret_ui::tree::PointerOcclusion;
pub(crate) use fret_ui::{ElementContext, GlobalElementId, UiTree};
pub(crate) use fret_ui_kit::OverlayController;
pub(crate) use fret_ui_kit::imui::UiWriterImUiFacadeExt;
pub(crate) use fret_ui_kit::imui::{
    CheckboxOptions, ComboModelOptions, ComboOptions, FloatingAreaOptions, FloatingWindowOptions,
    FloatingWindowResizeOptions, GridOptions, HorizontalOptions, ImUiHoveredFlags, IndentOptions,
    InputTextOptions, ItemFlowOptions, ListBoxOptions, MenuItemOptions, PopupMenuOptions,
    PopupModalOptions, SameLineOptions, ScrollOptions, SelectableOptions, SliderOptions,
    SpacingOptions, SwitchOptions, TableColumn, TableColumnPin, TableOptions, VerticalOptions,
    VirtualListMeasureMode, VirtualListOptions, VirtualListScrollHandle, WindowOptions,
};
pub(crate) use fret_ui_kit::{OverlayPresence, OverlayRequest};

mod events;
mod floating_scenes;
mod frames;
mod host;
mod hover_scenes;
mod lookup;
mod services;

pub(crate) use events::*;
pub(crate) use floating_scenes::*;
pub(crate) use frames::*;
pub(crate) use host::TestHost;
pub(crate) use hover_scenes::*;
pub(crate) use lookup::*;
pub(crate) use services::FakeTextService;
