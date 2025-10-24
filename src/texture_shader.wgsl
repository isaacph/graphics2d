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

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32,
            buffer: InstanceBuffer) -> VertexOutput {
    let mat = mat4x4<f32>(buffer.model_matrix_0, buffer.model_matrix_1, buffer.model_matrix_2, buffer.model_matrix_3);
    let x = f32(vertices[in_vertex_index * 2]) - 0.5;
    let y = f32(vertices[in_vertex_index * 2 + 1]) - 0.5;
    var out: VertexOutput;
    out.clip_position = projection * mat * vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2(x + 0.5, y + 0.5);
    return out;
}

@group(0) @binding(1) var diffuse_texture: texture_2d<f32>;
@group(0) @binding(2) var diffuse_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(diffuse_texture, diffuse_sampler, in.tex_coords);
}

