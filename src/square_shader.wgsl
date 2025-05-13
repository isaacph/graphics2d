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
var<uniform> projection: mat4x4<f32>;

struct InstanceBuffer {
    @location(0) model_matrix_0: vec4<f32>,
    @location(1) model_matrix_1: vec4<f32>,
    @location(2) model_matrix_2: vec4<f32>,
    @location(3) model_matrix_3: vec4<f32>, 
}

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32,
            buffer: InstanceBuffer) -> @builtin(position) vec4<f32> {
    let mat = mat4x4<f32>(buffer.model_matrix_0, buffer.model_matrix_1, buffer.model_matrix_2, buffer.model_matrix_3);
    let x = f32(vertices[in_vertex_index * 2]) - 0.5;
    let y = f32(vertices[in_vertex_index * 2 + 1]) - 0.5;
    return projection * mat * vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

