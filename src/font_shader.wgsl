// Vertex shader
struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>, 
    @location(9) texture_pos: vec2<f32>,
    @location(10) texture_scale: vec2<f32>,
    @location(11) color: vec4<f32>,
};

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    var out: VertexOutput;
    out.tex_coords = instance.texture_pos +
        vec2(
            model.position.x * instance.texture_scale.x,
            model.position.y * instance.texture_scale.y
        );
    out.clip_position = model_matrix * vec4<f32>(model.position, 0.0, 1.0);
    out.color = instance.color;
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let r = textureSample(t_diffuse, s_diffuse, in.tex_coords).r;
    return in.color * vec4(1.0, 1.0, 1.0, r);
}


