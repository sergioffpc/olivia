use olivia_math::Vec2;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::window::Fullscreen;

/// What happened to the window since the last update.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WindowEvent {
    /// The window has this logical size.
    Resized(Vec2),
    /// The window gained or lost focus.
    Focus(bool),
    /// The window was minimised or restored. Wayland does not report this, so it never arrives there.
    Minimized(bool),
    /// The player asked to close the window. It stays open until the game quits.
    CloseRequested,
}

/// The game's window. Its state reaches the game only through [`WindowEvent`]s.
#[derive(Debug)]
pub struct Window {
    /// The OS window; a headless window has none.
    raw: Option<winit::window::Window>,
    /// A size applied at once, for which the platform sends no resize event.
    pub(crate) applied_size: Option<Vec2>,
}

impl Window {
    pub(crate) fn new(raw: winit::window::Window) -> Self {
        Self {
            raw: Some(raw),
            applied_size: None,
        }
    }

    /// Creates a window with no OS window behind it, where the setters do nothing.
    #[doc(hidden)]
    #[must_use]
    pub fn headless() -> Self {
        Self {
            raw: None,
            applied_size: None,
        }
    }

    /// Sets the title.
    pub fn set_title(&mut self, title: &str) {
        if let Some(raw) = &self.raw {
            raw.set_title(title);
        }
    }

    /// Enters or leaves fullscreen.
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        if let Some(raw) = &self.raw {
            raw.set_fullscreen(fullscreen.then_some(Fullscreen::Borderless(None)));
        }
    }

    /// Asks for this logical size. [`WindowEvent::Resized`] reports the size the window gets.
    pub fn set_size(&mut self, size: Vec2) {
        if let Some(raw) = &self.raw {
            let applied = raw.request_inner_size(LogicalSize::new(size.x, size.y));
            self.applied_size = applied.map(|size| logical_size(size, raw.scale_factor()));
        }
    }

    /// The underlying `winit` window. Unstable: it changes with `winit`.
    ///
    /// # Panics
    ///
    /// Panics on a headless window.
    #[must_use]
    pub fn raw(&self) -> &winit::window::Window {
        self.raw
            .as_ref()
            .expect("a headless window has no OS window")
    }
}

/// Converts a window size to the logical size the game sees.
pub(crate) fn logical_size(size: PhysicalSize<u32>, scale_factor: f64) -> Vec2 {
    let size: LogicalSize<f32> = size.to_logical(scale_factor);
    Vec2::new(size.width, size.height)
}
