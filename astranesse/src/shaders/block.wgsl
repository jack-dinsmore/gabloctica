struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_inv: mat4x4<f32>,
    shadow_proj: mat4x4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
};
struct LightUniform {
    pos: vec4<f32>,
};


// ==========================
// CUSTOM CODE
// ==========================

struct VertexInput {
    @location(0) data: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec4<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) @interpolate(perspective, sample) texpos: vec2<f32>,
    @location(3) shadow_pos: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> lights: LightUniform;
@group(3) @binding(0) var t_diffuse: texture_2d<f32>;
@group(3) @binding(1) var s_diffuse: sampler;
@group(3) @binding(2) var t_shadow: texture_depth_2d;
@group(3) @binding(3) var s_shadow: sampler_comparison;


fn shadow_map(x: f32) -> f32 {
    return (sqrt(1 + x*x) - 1.) / x;
}

@vertex
fn vs_main(
    vert: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4(
        f32((vert.data   ) & 0x1f),
        f32((vert.data>>5) & 0x1f),
        f32((vert.data>>10) & 0x1f),
        1.
    );

    var normal = vec3(0., 0., 0.);
    if (((vert.data>>15) & 0x7) == 0) { normal.x = 1.; }
    if (((vert.data>>15) & 0x7) == 1) { normal.x = -1.; }
    if (((vert.data>>15) & 0x7) == 2) { normal.y = 1.; }
    if (((vert.data>>15) & 0x7) == 3) { normal.y = -1.; }
    if (((vert.data>>15) & 0x7) == 4) { normal.z = 1.; }
    if (((vert.data>>15) & 0x7) == 5) { normal.z = -1.; }

    out.texpos = vec2(
        f32((vert.data>>26) & 0x7) / 7.,
        f32((vert.data>>18) & 0xff) / 255.,
    );

    if ((vert.data>>29) & 1) != 0 { out.position.x += 32.; }
    if ((vert.data>>30) & 1) != 0 { out.position.y += 32.; }
    if ((vert.data>>31) & 1) != 0 { out.position.z += 32.; }

    let rotation = mat3x3(
        model.model[0].xyz,
        model.model[1].xyz,
        model.model[2].xyz,
    );
    
    out.position = model.model * out.position;
    out.normal = rotation * normal;
    out.clip_position = camera.view_proj * out.position;
    out.shadow_pos = camera.shadow_proj * out.position;

    out.shadow_pos /= out.shadow_pos.w;
    out.shadow_pos.x = shadow_map(out.shadow_pos.x);
    out.shadow_pos.y = shadow_map(out.shadow_pos.y);

    return out;
}

@fragment

fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.texpos);
    if color.a == 0 {discard;}
    let to_light = normalize(lights.pos.xyz - in.position.xyz);

    var shadow_coords = in.shadow_pos.xyz / in.shadow_pos.w;
    shadow_coords.y *= -1.;

    let shadow = textureSampleCompare(t_shadow, s_shadow, shadow_coords.xy * 0.5 + 0.5, shadow_coords.z - 0.00005);

    var illum = shadow * dot(in.normal.xyz, to_light);

    illum = max(illum, 0.1);

    color.x *= illum;
    color.y *= illum;
    color.z *= illum;

    return color;
    // return clamp_color(color);
}