struct CameraUniform {
    view_proj: mat4x4<f32>,
};struct ModelUniform {
    model: mat4x4<f32>,
};

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) texpos: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(1) texpos: vec2<f32>,
}

const TEXTURE_SIZE: f32 = 16.; // Blocks per texture

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var t_diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;

@vertex
fn vs_main(
    vert: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_proj * model.model * vert.position;
    out.texpos = vert.texpos;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.texpos);
    return color;
}