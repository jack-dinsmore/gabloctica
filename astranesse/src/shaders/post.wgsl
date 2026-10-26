struct CameraUniform {
    view_proj: mat4x4<f32>,
    shadow_proj: mat4x4<f32>,
    view_proj_inv: mat4x4<f32>,
};
struct LightUniform {
    pos: vec4<f32>,
};
struct PostUniform {
    planet_model_inv: mat4x4<f32>,
    viewport: vec4<f32>,
    fog: vec4<f32>,
    scale_height: vec4<f32>,
};

/// Returns depth
fn linearize_depth(depth: f32, near: f32, far: f32) -> f32 {
    return (near * far) / (far - depth * (far - near));
}

// ==========================
// CUSTOM CODE
// ==========================

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4<f32> {
    var pos = array<vec2<f32>, 3>(
        vec2(-1.0, -3.0),
        vec2( 3.0,  1.0),
        vec2(-1.0,  1.0)
    );

    return vec4(pos[i], 0.0, 1.0);
}

@group(0) @binding(0) var<uniform> post : PostUniform;
@group(1) @binding(0) var color_tex : texture_2d<f32>;
@group(1) @binding(1) var samp : sampler;
@group(1) @binding(2) var depth_tex : texture_depth_2d;
@group(2) @binding(0) var<uniform> camera: CameraUniform;
@group(3) @binding(0) var<uniform> lights: LightUniform;

fn get_containing_normal(pos: vec3<f32>) -> vec3<f32> {
    var normal = vec3<f32>(0., 0., 0.);
    // Get the normal vector of the final position
    let max = max(max(abs(pos.x), abs(pos.y)), abs(pos.z));
    if max == -pos.x { normal.x = -1.; }
    if max == pos.x { normal.x = 1.; }
    if max == -pos.y { normal.y = -1.; }
    if max == pos.y { normal.y = 1.; }
    if max == -pos.z { normal.z = -1.; }
    if max == pos.z { normal.z = 1.; }

    return normal;
}

fn get_optical_depth(start_cube: vec3<f32>, stop_cube: vec3<f32>) -> f32 {
    let look = stop_cube - start_cube;
    let look_length = length(look);
    let alphas = array<f32, 6>(
        -(start_cube.x+start_cube.y) / (look.x+look.y),
        -(start_cube.x+start_cube.z) / (look.x+look.z),
        -(start_cube.x-start_cube.y) / (look.x-look.y),
        -(start_cube.x-start_cube.z) / (look.x-look.z),
        -(start_cube.y+start_cube.z) / (look.y+look.z),
        -(start_cube.y-start_cube.z) / (look.y-look.z),
    );

    var depth = 0.;
    var start_alpha = 0.;
    for (var stop_index = 0; stop_index < 10; stop_index++) {
        var stop_alpha = 1.;
        for (var i: u32 = 0; i < 6; i++) {
            if alphas[i] < 1. && alphas[i] < stop_alpha && alphas[i] > start_alpha {
                stop_alpha = alphas[i];
            }
        }
        let normal = get_containing_normal(start_cube + look * (start_alpha + stop_alpha) / 2.);
        let ldot = dot(look, normal);
        let adot = dot(start_cube, normal) - 1.;
        let secant = look_length / ldot;

        depth += exp(-(ldot * start_alpha + adot) / post.scale_height.x) * secant;
        depth -= exp(-(ldot * stop_alpha + adot) / post.scale_height.x) * secant;
        if stop_alpha >= 1. {break;}
        start_alpha = stop_alpha;
    }

    return depth * post.fog.w;
}

@fragment
fn fs_main(@builtin(position) pos : vec4<f32>) -> @location(0) vec4<f32> {
    let uv = vec2(pos.x / post.viewport.x, pos.y / post.viewport.y); // Division
    let depth = textureSample(depth_tex, samp, uv);
    let color = textureSample(color_tex, samp, uv);

    let end_space = camera.view_proj_inv * vec4(2. * uv.x - 1., 1. - 2*uv.y, depth, 1.);
    let end_cube = post.planet_model_inv * (end_space / end_space.w);
    let start_cube = post.planet_model_inv * vec4(0., 0., 0., 1.);

    let optical_depth = get_optical_depth(start_cube.xyz, end_cube.xyz);

    let mixed_color =  mix(post.fog.xyz, color.xyz, exp(-optical_depth));

    return vec4(mixed_color, color.w);
}