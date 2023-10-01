struct Uniform {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) // 1.
var<uniform> uniform_: Uniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv_position: vec2<f32>,
};

struct InstanceInput {
    @location(10) model_matrix_0: vec4<f32>,
    @location(11) model_matrix_1: vec4<f32>,
    @location(12) model_matrix_2: vec4<f32>,
    @location(13) model_matrix_3: vec4<f32>,
    @location(14) color: vec4<f32>,
};


struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv_position: vec2<f32>,
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
    out.clip_position = uniform_.view_proj * model_matrix * vec4<f32>(model.position, 1.0);
    out.color = instance.color;
    out.uv_position = model.uv_position;
    return out;
}


@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1)@binding(1)
var sampler_: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, sampler_, in.uv_position) * in.color;
}