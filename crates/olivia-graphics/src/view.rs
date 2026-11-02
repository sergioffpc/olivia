use std::ops::Range;
use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use olivia_math::{Color, Vec2};
use tracing::field::Empty;
use tracing::trace_span;

use crate::camera::Camera;
use crate::draw::{Blend, Draw, Filter};
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

/// What a frame has drawn so far, in call order.
#[derive(Debug, Default)]
pub(crate) struct DrawCalls {
    vertices: Vec<Vertex>,
    calls: Vec<Call>,
    /// How many views the frame has started; the last one is the one drawing.
    views: usize,
}

/// One draw call.
#[derive(Debug)]
struct Call {
    view: usize,
    depth: f32,
    image: Arc<Image>,
    filter: Filter,
    blend: Blend,
    vertices: Range<usize>,
}

impl DrawCalls {
    /// Sorts the calls by view, then by depth, and batches the result. The sort is stable, so ties
    /// keep call order.
    pub(crate) fn mesh(&self) -> Mesh {
        let span = trace_span!("batching", batches = Empty).entered();
        let mut calls: Vec<&Call> = self.calls.iter().collect();
        calls.sort_by(|a, b| {
            let view = a.view.cmp(&b.view);
            view.then(a.depth.total_cmp(&b.depth))
        });
        let mut mesh = Mesh {
            vertices: Vec::with_capacity(self.vertices.len()),
            batches: Vec::new(),
        };
        for call in calls {
            mesh.push(call, &self.vertices[call.vertices.clone()]);
        }
        span.record("batches", mesh.batches.len());
        mesh
    }

    /// Adds `vertices` as one call through the last view.
    fn push(&mut self, image: &Arc<Image>, draw: Draw, vertices: [Vertex; 6]) {
        let start = self.vertices.len();
        self.vertices.extend(vertices);
        self.calls.push(Call {
            view: self.views,
            depth: draw.depth,
            image: Arc::clone(image),
            filter: draw.filter,
            blend: draw.blend,
            vertices: start..self.vertices.len(),
        });
    }
}

/// A frame ready to submit: its triangles in drawing order, and the batches that cover them.
#[doc(hidden)]
#[derive(Debug)]
pub struct Mesh {
    /// Every vertex, in drawing order.
    pub vertices: Vec<Vertex>,
    /// One entry for each run of calls that share their GPU state.
    pub batches: Vec<Batch>,
}

/// A run of vertices that draw with the same image, filter and blend.
#[doc(hidden)]
#[derive(Debug)]
pub struct Batch {
    pub(crate) image: Arc<Image>,
    /// How the image is sampled.
    pub filter: Filter,
    /// How the vertices combine with what is under them.
    pub blend: Blend,
    /// The vertices of the [`Mesh`] in this batch.
    pub vertices: Range<u32>,
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

    /// Adds `vertices`, joining the last batch if `call` shares its state.
    fn push(&mut self, call: &Call, vertices: &[Vertex]) {
        self.vertices.extend_from_slice(vertices);
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a frame's vertices stay far below 2^32"
        )]
        let end = self.vertices.len() as u32;
        match self.batches.last_mut() {
            Some(batch)
                if Arc::ptr_eq(&batch.image, &call.image)
                    && batch.filter == call.filter
                    && batch.blend == call.blend =>
            {
                batch.vertices.end = end;
            }
            last => {
                let start = last.map_or(0, |batch| batch.vertices.end);
                self.batches.push(Batch {
                    image: Arc::clone(&call.image),
                    filter: call.filter,
                    blend: call.blend,
                    vertices: start..end,
                });
            }
        }
    }
}

/// What a frame draws through one [`Camera`], and the only thing that draws.
///
/// It borrows the frame, so views are created one after another, and they draw in that order.
/// [`depth`](Draw::depth) orders the calls within one view.
#[derive(Debug)]
pub struct View<'a> {
    calls: &'a mut DrawCalls,
    camera: Camera,
    viewport: Vec2,
}

impl<'a> View<'a> {
    pub(crate) fn new(calls: &'a mut DrawCalls, camera: Camera, viewport: Vec2) -> Self {
        calls.views += 1;
        Self {
            calls,
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
        self.calls.push(
            &texture.image,
            draw,
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
