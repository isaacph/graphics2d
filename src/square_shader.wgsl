const vertices = array<u32, 6 * 2>(
    0, 0,
    1, 0,
    1, 1,
    1, 1,
    0, 1,
    0, 0,
);

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(vertices[in_vertex_index * 2]) - 0.5;
    let y = f32(vertices[in_vertex_index * 2 + 1]) - 0.5;
    return transform * vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

