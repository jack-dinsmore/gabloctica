struct CameraUniform {
    view_proj: mat4x4<f32>,
    shadow_proj: mat4x4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
};
struct LightUniform {
    pos: vec4<f32>,
};

const TEXTURE_SIZE: f32 = 16.; // Blocks per texture

fn rgb_to_hsl(c: vec3<f32>) -> vec3<f32> {
    let maxc = max(c.r, max(c.g, c.b));
    let minc = min(c.r, min(c.g, c.b));
    let d = maxc - minc;

    let l = 0.5 * (maxc + minc);
    let e = 1e-10;

    let s = d / (1.0 - abs(2.0 * l - 1.0) + e);

    var h: f32;
    if (d < e) {
        h = 0.0;
    } else if (maxc == c.r) {
        h = (c.g - c.b) / d;
    } else if (maxc == c.g) {
        h = 2.0 + (c.b - c.r) / d;
    } else {
        h = 4.0 + (c.r - c.g) / d;
    }

    h = fract(h / 6.0);
    return vec3<f32>(h, s, l);
}

fn hue_to_rgb(p: f32, q: f32, t: f32) -> f32 {
    let tt = fract(t);
    if (tt < 1.0 / 6.0) { return p + (q - p) * 6.0 * tt; }
    if (tt < 1.0 / 2.0) { return q; }
    if (tt < 2.0 / 3.0) { return p + (q - p) * (2.0 / 3.0 - tt) * 6.0; }
    return p;
}

fn hsl_to_rgb(c: vec3<f32>) -> vec3<f32> {
    let h = c.x;
    let s = c.y;
    let l = c.z;

    if (s == 0.0) {
        return vec3<f32>(l);
    }

    let q = select(
        l * (1.0 + s),
        l + s - l * s,
        l >= 0.5
    );
    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    return vec3<f32>(r, g, b);
}

fn clamp_color(c: vec4<f32>) -> vec4<f32> {
    var hsl = rgb_to_hsl(c.xyz);
    // hsl = round(hsl*32.) / 32.;
    // hsl.y = 1.;
    return vec4(hsl_to_rgb(hsl), c.w);
}


// ==========================
// CUSTOM CODE
// ==========================

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

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;
@group(2) @binding(0) var<uniform> lights: LightUniform;
@group(3) @binding(0) var t_diffuse: texture_2d<f32>;
@group(3) @binding(1) var s_diffuse: sampler;
@group(3) @binding(2) var t_shadow: texture_depth_2d;
@group(3) @binding(3) var s_shadow: sampler;

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

    if ((vert.data>>24) & 1) != 0 { out.position.x += 16.; }
    if ((vert.data>>25) & 1) != 0 { out.position.y += 16.; }
    if ((vert.data>>26) & 1) != 0 { out.position.z += 16.; }

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
    shadow_coords.y *= -1.;
    var closest_depth = textureSample(t_shadow, s_shadow, shadow_coords.xy * 0.5 + 0.5);
    let shadow = step(closest_depth + 0.0005, shadow_coords.z);

    var illum = (1. - shadow) * dot(in.normal, to_light);

    illum = max(illum, 0.1);

    color.x *= illum;
    color.y *= illum;
    color.z *= illum;

    return color;
    // return clamp_color(color);
}