use std::ops::Range;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use olivia_math::{Color, Vec2};

use crate::camera::Camera;
use crate::draw::Draw;
use crate::texture::{Image, Texture};

/// One corner of a triangle.
#[doc(hidden)]
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct Vertex {
    /// Where it is, in logical pixels from the top-left corner of the window.
    pub position: Vec2,
    /// Where it samples its texture, from 0 to 1.
    pub uv: Vec2,
    /// The colour it multiplies its texture by, premultiplied.
    pub color: [f32; 4],
}

/// What a frame has drawn so far, in order.
#[derive(Debug, Default)]
pub(crate) struct Mesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) batches: Vec<Batch>,
}

/// A run of vertices that sample one image.
#[derive(Debug)]
pub(crate) struct Batch {
    pub(crate) image: Arc<Image>,
    pub(crate) vertices: Range<u32>,
}

impl Mesh {
    /// The vertices in clip space, which has Y up, for a window of logical size `viewport`.
    pub(crate) fn clip_vertices(&self, viewport: Vec2) -> Vec<Vertex> {
        let clip = |position: Vec2| {
            let normalised = position / viewport * 2.0 - Vec2::ONE;
            Vec2::new(normalised.x, -normalised.y)
        };
        let vertices = self.vertices.iter().map(|vertex| Vertex {
            position: clip(vertex.position),
            ..*vertex
        });
        vertices.collect()
    }

    /// Adds `vertices`, sampling `image`.
    fn push(&mut self, image: &Arc<Image>, vertices: [Vertex; 6]) {
        self.vertices.extend(vertices);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a frame's vertices stay far below 2^32"
        )]
        let end = self.vertices.len() as u32;
        match self.batches.last_mut() {
            Some(batch) if Arc::ptr_eq(&batch.image, image) => batch.vertices.end = end,
            last => {
                let start = last.map_or(0, |batch| batch.vertices.end);
                self.batches.push(Batch {
                    image: Arc::clone(image),
                    vertices: start..end,
                });
            }
        }
    }
}

/// What a frame draws through one [`Camera`], and the only thing that draws.
///
/// It borrows the frame, so views are created one after another, and they draw in that order.
#[derive(Debug)]
pub struct View<'a> {
    mesh: &'a mut Mesh,
    camera: Camera,
    viewport: Vec2,
}

impl<'a> View<'a> {
    pub(crate) fn new(mesh: &'a mut Mesh, camera: Camera, viewport: Vec2) -> Self {
        Self {
            mesh,
            camera,
            viewport,
        }
    }

    /// Draws `texture` with its pivot at `pos`, one unit to each of its pixels.
    pub fn sprite(&mut self, texture: &Texture, pos: Vec2, draw: Draw) {
        let turn = Vec2::from_angle(draw.rotation);
        let (uv_start, uv_end) = texture.uv();
        let Color { r, g, b, a } = draw.color;
        let color = [r * a, g * a, b * a, a];
        let vertex = |corner: Vec2| Vertex {
            position: self.camera.world_to_screen(
                pos + turn.rotate((corner - draw.origin) * texture.size() * draw.scale),
                self.viewport,
            ),
            uv: uv_start + corner * (uv_end - uv_start),
            color,
        };
        let [top_left, top_right] = [vertex(Vec2::ZERO), vertex(Vec2::X)];
        let [bottom_left, bottom_right] = [vertex(Vec2::Y), vertex(Vec2::ONE)];
        self.mesh.push(
            &texture.image,
            [
                top_left,
                top_right,
                bottom_left,
                bottom_left,
                top_right,
                bottom_right,
            ],
        );
    }
}
