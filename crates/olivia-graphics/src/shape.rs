use lyon::algorithms::aabb::bounding_box;
use lyon::math::{Box2D, Point, Transform, point};
use lyon::path::{Path, Winding};
use lyon::tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
    StrokeVertex, TessellationResult, VertexBuffers,
};
use olivia_math::Vec2;
use tracing::{trace_span, warn};

/// How far a curve strays from its true shape, in logical pixels.
const TOLERANCE: f32 = 0.1;
/// The finest tolerance, however far a camera zooms in.
const MIN_TOLERANCE: f32 = 1e-4;
/// The coarsest tolerance, however far a camera zooms out.
const MAX_TOLERANCE: f32 = 1e6;

fn lyon_point(position: Vec2) -> Point {
    point(position.x, position.y)
}

/// A rectangle of `size` from `(0, 0)`, or nothing if it is not finite, which lyon rejects.
pub(crate) fn rectangle(size: Vec2) -> Path {
    if !size.is_finite() {
        return Path::new();
    }
    let mut builder = Path::builder();
    let rect = Box2D::new(point(0.0, 0.0), lyon_point(size));
    builder.add_rectangle(&rect, Winding::Positive);
    builder.build()
}

/// A circle of `radius` around `(0, 0)`, or nothing if it is negative or not finite.
pub(crate) fn circle(radius: f32) -> Path {
    if !(radius >= 0.0 && radius.is_finite()) {
        return Path::new();
    }
    let mut builder = Path::builder();
    builder.add_circle(point(0.0, 0.0), radius, Winding::Positive);
    builder.build()
}

/// The polygon through `points`, or an open line if it is not `closed`, or nothing if a point is
/// not finite, which lyon rejects.
pub(crate) fn polygon(points: &[Vec2], closed: bool) -> Path {
    if !points.iter().all(|point| point.is_finite()) {
        return Path::new();
    }
    let mut builder = Path::builder();
    if let Some((&first, rest)) = points.split_first() {
        builder.begin(lyon_point(first));
        for &point in rest {
            builder.line_to(lyon_point(point));
        }
        builder.end(closed);
    }
    builder.build()
}

/// `path` scaled around its pivot, which `origin` normalises to its box and which is its `(0, 0)`
/// by default, with the pivot moved to `(0, 0)`. Doing this before tessellating keeps line widths
/// in world units whatever the scale. It is nothing if the result would not be finite.
pub(crate) fn placed(path: Path, origin: Option<Vec2>, scale: Vec2) -> Path {
    let pivot = origin.map_or(Vec2::ZERO, |origin| {
        let bounds = bounding_box(path.iter());
        Vec2::new(bounds.min.x, bounds.min.y) + origin * Vec2::new(bounds.width(), bounds.height())
    });
    if !(scale.is_finite() && pivot.is_finite()) {
        return Path::new();
    }
    let transform = Transform::translation(-pivot.x, -pivot.y).then_scale(scale.x, scale.y);
    path.transformed(&transform)
}

/// The tolerance for a shape seen through a camera of `zoom`.
pub(crate) fn tolerance(zoom: f32) -> f32 {
    let tolerance = TOLERANCE / zoom.abs();
    if tolerance.is_nan() {
        return TOLERANCE;
    }
    tolerance.clamp(MIN_TOLERANCE, MAX_TOLERANCE)
}

/// The corners of the triangles that fill `path`.
pub(crate) fn fill(path: &Path, tolerance: f32) -> Vec<Vec2> {
    let _span = trace_span!("tessellation").entered();
    let options = FillOptions::tolerance(tolerance);
    triangles(|buffers| {
        let corner = |vertex: FillVertex<'_>| vertex.position();
        FillTessellator::new().tessellate_path(
            path,
            &options,
            &mut BuffersBuilder::new(buffers, corner),
        )
    })
}

/// The corners of the triangles that trace `path` with a line `width` wide.
pub(crate) fn stroke(path: &Path, width: f32, tolerance: f32) -> Vec<Vec2> {
    let _span = trace_span!("tessellation").entered();
    if !(width > 0.0 && width.is_finite()) {
        return Vec::new();
    }
    let options = StrokeOptions::tolerance(tolerance).with_line_width(width);
    triangles(|buffers| {
        let corner = |vertex: StrokeVertex<'_, '_>| vertex.position();
        StrokeTessellator::new().tessellate_path(
            path,
            &options,
            &mut BuffersBuilder::new(buffers, corner),
        )
    })
}

/// Expands what `tessellate` builds into plain triangles, or none if it fails, as `draw` cannot.
fn triangles(
    tessellate: impl FnOnce(&mut VertexBuffers<Point, u32>) -> TessellationResult,
) -> Vec<Vec2> {
    let mut buffers = VertexBuffers::new();
    if let Err(error) = tessellate(&mut buffers) {
        warn!(%error, "shape not drawn");
        return Vec::new();
    }
    let corners = buffers.indices.iter().map(|&index| {
        let vertex = buffers.vertices[index as usize];
        Vec2::new(vertex.x, vertex.y)
    });
    corners.collect()
}
