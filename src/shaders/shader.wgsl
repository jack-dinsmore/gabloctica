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
    
    var normal = vec3(0., 0., 0.);
    if (((vert.data>>12) & 0xf) == 0) { normal.x = 1.; }
    if (((vert.data>>12) & 0xf) == 1) { normal.x = -1.; }
    if (((vert.data>>12) & 0xf) == 2) { normal.y = 1.; }
    if (((vert.data>>12) & 0xf) == 3) { normal.y = -1.; }
    if (((vert.data>>12) & 0xf) == 4) { normal.z = 1.; }
    if (((vert.data>>12) & 0xf) == 5) { normal.z = -1.; }

    out.texpos = vec2(
        f32((vert.data>>16) & 0xf) / TEXTURE_SIZE,
        f32((vert.data>>20) & 0xf) / TEXTURE_SIZE
    );

    let rotation = mat3x3(
        model.model[0].xyz,
        model.model[1].xyz,
        model.model[2].xyz,
    );
    
    out.position = model.model * out.position;
    out.normal = vec4(rotation * normal, 1.);
    out.clip_position = camera.view_proj * out.position;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.texpos);
    var to_light = normalize(lights.pos - in.position);
    var illum = dot(in.normal, to_light);
    illum = max(illum, 0.1);

    color.x *= illum;
    color.y *= illum;
    color.z *= illum;
    return color;
}