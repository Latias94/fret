use fret_core::AppWindowId;
use slotmap::SlotMap;
use winit::{dpi::LogicalSize, event_loop::ActiveEventLoop, window::Window};

#[derive(Default)]
pub struct WinitWindows {
    windows: SlotMap<AppWindowId, Window>,
}

impl WinitWindows {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_window(
        &mut self,
        event_loop: &ActiveEventLoop,
        title: &str,
        size: (u32, u32),
    ) -> Result<AppWindowId, winit::error::OsError> {
        let window = event_loop.create_window(
            Window::default_attributes()
                .with_title(title)
                .with_inner_size(LogicalSize::new(size.0 as f64, size.1 as f64)),
        )?;

        Ok(self.windows.insert(window))
    }

    pub fn get(&self, id: AppWindowId) -> Option<&Window> {
        self.windows.get(id)
    }
}
