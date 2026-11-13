struct CameraUniform {
    view_proj: mat4x4<f32>,
    view_proj_inv: mat4x4<f32>,
    shadow_proj: mat4x4<f32>,
};
struct LightUniform {
    pos: vec4<f32>,
};
struct PostUniform {
    planet_model_inv: mat4x4<f32>,
    viewport: vec4<f32>,
    fog: vec4<f32>,
};

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
@group(0) @binding(1) var color_tex : texture_multisampled_2d<f32>;
@group(0) @binding(2) var depth_tex : texture_depth_multisampled_2d;
@group(1) @binding(0) var<uniform> camera: CameraUniform;
@group(2) @binding(0) var<uniform> lights: LightUniform;
@group(3) @binding(2) var t_shadow: texture_depth_2d;
@group(3) @binding(3) var s_shadow: sampler_comparison;

fn shadow_map(x: f32) -> f32 {
    return (sqrt(1 + x*x) - 1.) / x;
}

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

fn get_optical_depth(start_cube: vec3<f32>, stop_cube: vec3<f32>) -> vec3<f32> {
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

        depth += exp(-(ldot * start_alpha + adot) / post.fog.w) * secant;
        depth -= exp(-(ldot * stop_alpha + adot) / post.fog.w) * secant;
        if stop_alpha >= 1. {break;}
        start_alpha = stop_alpha;
    }

    return depth * post.fog.xyz;
}

fn get_optical_depth_bright(start_cube: vec3<f32>, end_cube: vec3<f32>, sun_cube: vec3<f32>) -> vec3<f32> {
    let look = end_cube - start_cube;
    let look_length = length(look);
    let alphas = array<f32, 6>(
        -(start_cube.x+start_cube.y) / (look.x+look.y),
        -(start_cube.x-start_cube.y) / (look.x-look.y),
        -(start_cube.x+start_cube.z) / (look.x+look.z),
        -(start_cube.x-start_cube.z) / (look.x-look.z),
        -(start_cube.y+start_cube.z) / (look.y+look.z),
        -(start_cube.y-start_cube.z) / (look.y-look.z),
    );

    var depth = vec3(0.);
    var start_alpha = 0.;
    for (var stop_index = 0; stop_index < 10; stop_index++) {
        var stop_alpha = 1.;
        for (var i: u32 = 0; i < 6; i++) {
            if alphas[i] < 1. && alphas[i] < stop_alpha && alphas[i] > start_alpha {
                stop_alpha = alphas[i];
            }
        }
        var normal = get_containing_normal(start_cube + look * (start_alpha + stop_alpha) / 2.);
        let sdot = dot(sun_cube, normal);
        let ldot = dot(look, normal);
        let adot = dot(start_cube, normal) - 1.;
        let secant_l = look_length / ldot;
        let secant_s = 1. / sdot;

        let a = exp(-(adot + ldot*stop_alpha) / post.fog.w) * secant_s * post.fog.xyz;
        let b = exp(-(adot + ldot*start_alpha) / post.fog.w) * secant_s * post.fog.xyz;
        depth += (exp(-a) - exp(-b)) * secant_l / secant_s;

        if stop_alpha >= 1. {break;}
        start_alpha = stop_alpha;
    }

    return depth;
}

fn get_sky_color(start_space: vec4<f32>, end_space: vec4<f32>, sun_cube: vec3<f32>) -> vec3<f32> {
    // Get the location at which the sight ray enters a shadow
    let start_cube = (post.planet_model_inv * (start_space/start_space.w)).xyz;
    let end_cube = (post.planet_model_inv * (end_space/end_space.w)).xyz;

    return 0.079577 * get_optical_depth_bright(start_cube, end_cube, sun_cube);
}

@fragment
fn fs_main(@builtin(position) pos : vec4<f32>) -> @location(0) vec4<f32> {
    let uv = vec2(pos.x / post.viewport.x, pos.y / post.viewport.y); // Division

    // Get color
    var color = vec3<f32>(0., 0., 0.);
    var depth = 0.;
    let pixel = vec2<i32>(pos.xy);
    let count = i32(textureNumSamples(color_tex));
    for (var i = 0; i < count; i++) {
        color += textureLoad(color_tex, pixel, i).xyz;
        depth += textureLoad(depth_tex, pixel, i);
    }
    color /= f32(count);
    depth /= f32(count);

    if post.fog.w > 0. {
        let end_space = camera.view_proj_inv * vec4(2. * uv.x - 1., 1. - 2*uv.y, depth, 1.);
        let start_space =  vec4(0., 0., 0., 1.);
        let end_cube = post.planet_model_inv * (end_space / end_space.w);
        let start_cube = post.planet_model_inv * start_space;
        let sun_cube = normalize((post.planet_model_inv * lights.pos).xyz);

    
        let optical_depth = get_optical_depth(start_cube.xyz, end_cube.xyz);
        let sky_color = get_sky_color(start_space, end_space, sun_cube);

        color *= exp(-optical_depth);
        color += sky_color;
    }

    return vec4(color, 1.);
}