use olivia_math::Vec2;

/// Which part of the game's coordinates is seen. Coordinates are logical pixels with Y down.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    /// The point at the centre of the screen.
    pub position: Vec2,
    /// How many logical pixels one unit covers.
    pub zoom: f32,
    /// How far the camera is turned clockwise, in radians.
    pub rotation: f32,
}

impl Camera {
    /// A camera where `(0, 0)` is the top-left corner and one unit is one logical pixel.
    ///
    /// `viewport` is the logical window size from
    /// [`WindowEvent::Resized`](olivia_window::WindowEvent::Resized).
    #[must_use]
    pub fn screen(viewport: Vec2) -> Self {
        Self {
            position: viewport / 2.0,
            zoom: 1.0,
            rotation: 0.0,
        }
    }

    /// Where the window point `point` is in the game's coordinates.
    #[must_use]
    pub fn screen_to_world(&self, point: Vec2, viewport: Vec2) -> Vec2 {
        let turned = (point - viewport / 2.0) / self.zoom;
        self.position + Vec2::from_angle(self.rotation).rotate(turned)
    }

    /// Where the point `point` of the game's coordinates is in the window.
    #[must_use]
    pub fn world_to_screen(&self, point: Vec2, viewport: Vec2) -> Vec2 {
        let turned = Vec2::from_angle(-self.rotation).rotate(point - self.position);
        turned * self.zoom + viewport / 2.0
    }
}
