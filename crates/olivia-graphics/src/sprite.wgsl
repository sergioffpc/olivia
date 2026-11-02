struct Vertex {
    @location(0) position: vec2f,
    @location(1) uv: vec2f,
    @location(2) color: vec4f,
}

struct Fragment {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) color: vec4f,
}

@group(0) @binding(0) var image: texture_2d<f32>;
@group(1) @binding(0) var sampling: sampler;

@vertex
fn vertex(vertex: Vertex) -> Fragment {
    return Fragment(vec4f(vertex.position, 0.0, 1.0), vertex.uv, vertex.color);
}

@fragment
fn fragment(fragment: Fragment) -> @location(0) vec4f {
    return textureSample(image, sampling, fragment.uv) * fragment.color;
}
