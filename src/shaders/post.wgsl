struct CameraUniform {
    view_proj: mat4x4<f32>,
    shadow_proj: mat4x4<f32>,
};
struct LightUniform {
    pos: vec4<f32>,
};
struct PostUniform {
    viewport: vec4<f32>,
    fog: vec4<f32>,
    normal: vec4<f32>,
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

@fragment
fn fs_main(@builtin(position) pos : vec4<f32>) -> @location(0) vec4<f32> {
    let uv = vec2(pos.x / post.viewport.x, pos.y / post.viewport.y); // Division
    let depth = textureSample(depth_tex, samp, uv);
    let color = textureSample(color_tex, samp, uv);
    let linear_depth = linearize_depth(depth, post.viewport.z, post.viewport.w);

    // Get the direction of this pixel

    // Get the dot product to the normal and the sun to get the scattering angle and path length.

    let path_length = 1.0 - exp(-linear_depth * post.fog.w);
    let mixed_color =  mix(color.xyz, post.fog.xyz, path_length);

    return vec4(mixed_color, color.w);
}