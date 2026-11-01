struct CameraUniform {
    view_proj: mat4x4<f32>,
    shadow_proj: mat4x4<f32>,
    view_proj_inv: mat4x4<f32>,
};
struct ModelUniform {
    model: mat4x4<f32>,
};

// ==========================
// CUSTOM CODE
// ==========================


struct VertexInput {
    @location(0) data: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniform;
@group(1) @binding(0) var<uniform> model: ModelUniform;

@vertex
fn vs_main(
    vert: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;

    var position = vec4(
        f32((vert.data   ) & 0x1f),
        f32((vert.data>>5) & 0x1f),
        f32((vert.data>>10) & 0x1f),
        1.
    );

    if ((vert.data>>29) & 1) != 0 { position.x += 32.; }
    if ((vert.data>>30) & 1) != 0 { position.y += 32.; }
    if ((vert.data>>31) & 1) != 0 { position.z += 32.; }

    out.clip_position = camera.shadow_proj * (model.model * position);
    return out;
}