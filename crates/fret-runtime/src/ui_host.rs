use std::any::{Any, TypeId};

use crate::{ClipboardToken, FrameId, ImageUploadToken, ShareSheetToken, TickId, TimerToken};
use fret_core::{AppWindowId, Point, PointerId};

use crate::{CommandRegistry, DragKindId, DragSession, Effect, ModelHost, ModelId};

pub trait GlobalsHost {
    fn set_global<T: Any>(&mut self, value: T);
    fn global<T: Any>(&self) -> Option<&T>;

    /// Returns a monotonically-increasing token for a global type.
    ///
    /// This is intended for derived-state memoization (e.g. selector deps signatures). Hosts that
    /// track global changes should override this to return a value that changes whenever a tracked
    /// global is updated via `set_global` / `with_global_mut`.
    ///
    /// The default implementation returns `None`, meaning the host does not expose global revision
    /// tokens (callers may fall back to value hashing or manual invalidation).
    #[inline]
    fn global_revision(&self, global: TypeId) -> Option<u64> {
        let _ = global;
        None
    }

    #[inline]
    fn global_revision_of<T: Any>(&self) -> Option<u64> {
        self.global_revision(TypeId::of::<T>())
    }

    fn with_global_mut<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R;

    /// Like [`GlobalsHost::with_global_mut`], but does not participate in the host's "global
    /// changed" tracking mechanism.
    ///
    /// This is intended for frame-local caches/registries that should not schedule redraw or UI
    /// invalidation by themselves. Hosts can override this to implement an actual untracked path.
    #[inline]
    fn with_global_mut_untracked<T: Any, R>(
        &mut self,
        init: impl FnOnce() -> T,
        f: impl FnOnce(&mut T, &mut Self) -> R,
    ) -> R {
        self.with_global_mut(init, f)
    }
}

pub trait ModelsHost: ModelHost {
    /// Returns and clears the list of models that were marked changed since the last call.
    fn take_changed_models(&mut self) -> Vec<ModelId>;
}

pub trait CommandsHost {
    /// Returns the command registry used by this host.
    fn commands(&self) -> &CommandRegistry;
}

pub trait EffectSink {
    /// Requests a redraw for the given window.
    fn request_redraw(&mut self, window: AppWindowId);
    /// Enqueues a runtime effect to be handled by the runner/backend.
    fn push_effect(&mut self, effect: Effect);
}

pub trait TimeHost {
    /// Current tick id (monotonically increasing).
    fn tick_id(&self) -> TickId;
    /// Current frame id (monotonically increasing).
    fn frame_id(&self) -> FrameId;
    /// Allocates the next timer token.
    fn next_timer_token(&mut self) -> TimerToken;
    /// Allocates the next clipboard token.
    fn next_clipboard_token(&mut self) -> ClipboardToken;
    /// Allocates the next share-sheet token.
    fn next_share_sheet_token(&mut self) -> ShareSheetToken;
    /// Allocates the next image-upload token.
    fn next_image_upload_token(&mut self) -> ImageUploadToken;
}

pub trait DragHost {
    /// Returns the drag session associated with the pointer, if any.
    fn drag(&self, pointer_id: PointerId) -> Option<&DragSession>;
    /// Returns a mutable drag session associated with the pointer, if any.
    fn drag_mut(&mut self, pointer_id: PointerId) -> Option<&mut DragSession>;
    /// Cancels a drag session associated with the pointer, if any.
    fn cancel_drag(&mut self, pointer_id: PointerId);

    /// Returns `true` if any active drag session matches the predicate.
    fn any_drag_session(&self, predicate: impl FnMut(&DragSession) -> bool) -> bool;

    /// Finds the first pointer id whose drag session matches the predicate.
    fn find_drag_pointer_id(
        &self,
        predicate: impl FnMut(&DragSession) -> bool,
    ) -> Option<PointerId>;

    /// Cancels all drag sessions matching the predicate and returns their pointer ids.
    fn cancel_drag_sessions(
        &mut self,
        predicate: impl FnMut(&DragSession) -> bool,
    ) -> Vec<PointerId>;

    /// Begins a drag session with a typed payload.
    fn begin_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    );

    /// Begins a cross-window drag session with a typed payload.
    fn begin_cross_window_drag_with_kind<T: Any>(
        &mut self,
        pointer_id: PointerId,
        kind: DragKindId,
        source_window: AppWindowId,
        start: Point,
        payload: T,
    );
}

/// Host services required by the retained UI runtime (`fret-ui`).
///
/// This is intentionally portable: it lives in `fret-runtime` so that third-party engines/editors
/// can embed `fret-ui` without adopting `fret-app`.
///
/// Note: the individual service traits are intentionally split so hosts can implement them
/// independently. `UiHost` remains the single bound used throughout `fret-ui`.
pub trait UiHost:
    GlobalsHost + ModelsHost + CommandsHost + EffectSink + TimeHost + DragHost
{
}

impl<T> UiHost for T where
    T: GlobalsHost + ModelsHost + CommandsHost + EffectSink + TimeHost + DragHost
{
}
