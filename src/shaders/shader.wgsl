struct CameraUniform {
    view_proj: mat4x4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
};
struct LightUniform {
    pos: vec4<f32>,
};

struct VertexInput {
    @location(0) data: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) texpos: vec2<f32>,
}

const TEXTURE_SIZE: f32 = 16.; // Blocks per texture

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> lights: LightUniform;
@group(3) @binding(0) var t_diffuse: texture_2d<f32>;
@group(3) @binding(1) var s_diffuse: sampler;

@vertex
fn vs_main(
    vert: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4(
        f32((vert.data   ) & 0xf),
        f32((vert.data>>4) & 0xf),
        f32((vert.data>>8) & 0xf),
        1.
    );
    
    out.normal = vec4(0., 0., 0., 1.);
    if (((vert.data>>12) & 0xf) == 0) { out.normal.x = 1.; }
    if (((vert.data>>12) & 0xf) == 1) { out.normal.x = -1.; }
    if (((vert.data>>12) & 0xf) == 2) { out.normal.y = 1.; }
    if (((vert.data>>12) & 0xf) == 3) { out.normal.y = -1.; }
    if (((vert.data>>12) & 0xf) == 4) { out.normal.z = 1.; }
    if (((vert.data>>12) & 0xf) == 5) { out.normal.z = -1.; }

    out.texpos = vec2(
        f32((vert.data>>16) & 0xf) / TEXTURE_SIZE,
        f32((vert.data>>20) & 0xf) / TEXTURE_SIZE
    );
    
    out.position = model.model * out.position;
    out.clip_position = camera.view_proj * out.position;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.texpos);
    var to_light = normalize(lights.pos - in.position);
    var illum = dot(in.normal, to_light);
    illum = max(illum, 0.);

    color.x *= illum;
    color.y *= illum;
    color.z *= illum;
    return color;
}