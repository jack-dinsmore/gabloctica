#import lib.wgsl

struct VertexInput {
    @location(0) data: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) texpos: vec2<f32>,
    @location(3) shadow_pos: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: lib::CameraUniform;
@group(1) @binding(0) var<uniform> model: lib::ModelUniform;
@group(2) @binding(0) var<uniform> lights: lib::LightUniform;
@group(3) @binding(0) var t_diffuse: texture_2d<f32>;
@group(3) @binding(1) var s_diffuse: sampler;
@group(4) @binding(0) var t_shadow: texture_depth_2d;
@group(4) @binding(1) var s_shadow: sampler;

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
        f32((vert.data>>16) & 0xf) / lib::TEXTURE_SIZE,
        f32((vert.data>>20) & 0xf) / lib::TEXTURE_SIZE
    );

    if (vert.data>>24 & 1) != 0 { out.position.x += 16.; }
    if (vert.data>>25 & 1) != 0 { out.position.y += 16.; }
    if (vert.data>>26 & 1) != 0 { out.position.z += 16.; }

    let rotation = mat3x3(
        model.model[0].xyz,
        model.model[1].xyz,
        model.model[2].xyz,
    );
    
    out.position = model.model * out.position;
    out.normal = vec4(rotation * normal, 1.);
    out.clip_position = camera.view_proj * out.position;
    out.shadow_pos = camera.shadow_proj * out.position;
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(t_diffuse, s_diffuse, in.texpos);
    var to_light = normalize(lights.pos - in.position);

    var shadow_coords = in.shadow_pos.xyz / in.shadow_pos.w;
    shadow_coords = shadow_coords * 0.5 + 0.5; 
    let closest_depth = textureSample(t_shadow, s_shadow, shadow_coords.xy);
    let current_depth = shadow_coords.z;

    var shadow = 0.;
    if (current_depth - 0.005 > closest_depth) {shadow = 1.;} // max(current_depth, closest_depth)==current_depth; 

    var illum = (1. - shadow) * dot(in.normal, to_light);
    illum = max(illum, 0.1);

    color.x *= illum;
    color.y *= illum;
    color.z *= illum;

    return lib::clamp_color(color);

    // return color;
}