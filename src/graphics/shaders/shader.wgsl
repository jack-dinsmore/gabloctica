struct CameraUniform {
    view_proj: mat4x4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
};
struct LightUniform {
    pos: vec4<f32>,
    normal: vec4<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> lights: LightUniform;

@vertex
fn vs_main(
    vert: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.position = model.model * vec4(vert.position, 1.0);
    out.clip_position = camera.view_proj * out.position;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var base_color = vec3(1., 1., 1.);
    var to_light = normalize(lights.pos - in.position);
    var illum = dot(lights.normal, to_light);
    illum = max(illum, 0.);
    return vec4(base_color*illum, 1.);
}