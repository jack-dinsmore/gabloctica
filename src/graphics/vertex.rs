#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    /// Storage spec:
    /// BITS    PURPOSE
    /// 0-3:    x pos
    /// 4-7:    y pos
    /// 8-11:   z pos
    /// 12-14:  normal (0, 1, 2, 3, 4, or 5. Other bytes are unused)
    /// 16-31:  id
    pub data: u32,
}

impl Vertex {
    pub(super) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}