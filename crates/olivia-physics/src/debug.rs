use olivia_math::Vec2;
use rapier2d::prelude::{DebugColor, DebugRenderBackend, DebugRenderObject};

/// Gathers the segments of one debug render, dropping the colour `rapier2d` suggests.
#[derive(Default)]
pub(crate) struct Lines(pub(crate) Vec<[Vec2; 2]>);

impl DebugRenderBackend for Lines {
    fn draw_line(&mut self, _object: DebugRenderObject<'_>, a: Vec2, b: Vec2, _color: DebugColor) {
        self.0.push([a, b]);
    }
}
