
use crate::graphics::{Camera, Graphics, Lighting, PostVertex, Renderer, ResourceType, Shader, Uniform, UniformBuffer};
use crate::game::Planet;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PostUniform {
    planet_model_inv: [[f32; 4]; 4],
    viewport: [f32; 4],
    fog: [f32; 4],
}

impl Uniform for PostUniform {
    const TYPE: ResourceType = ResourceType::Post;
}

pub struct PostInfo {
    shader: Shader,
    uniform: PostUniform,
    buffer: UniformBuffer<PostUniform>,
}
impl PostInfo {
    pub fn new(graphics: &mut Graphics, camera: &Camera) -> Self {
        let shader = Shader::new::<PostVertex>(graphics, include_str!("../../shaders/post.wgsl"), vec![
            ResourceType::Post,
            ResourceType::Texture,
            ResourceType::Camera,
            ResourceType::Lighting,
        ], true);

        let buffer = UniformBuffer::new(graphics);
        let size = graphics.window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        let uniform = PostUniform {
            viewport: [width as f32, height as f32, camera.znear, camera.zfar],
            fog: [0.6, 0.09, 0., 1.],
            planet_model_inv: [[0., 0., 0., 0.],[0., 0., 0., 0.],[0., 0., 0., 0.],[0., 0., 0., 0.]],
        };

        Self {
            buffer,
            uniform,
            shader,
        }
    }
    pub fn bind(&mut self, renderer: &mut Renderer) {
        self.buffer.bind(renderer);
    }
    pub fn update_buffer(&mut self, graphics: &Graphics, camera: &Camera, my_planet: Option<&Planet>) {
        match my_planet {
            Some(p) => match p.get_post_params(camera) {
                Some((planet_model_inv, fog)) => {
                    self.uniform.planet_model_inv = planet_model_inv;
                    self.uniform.fog = fog;
                }
                None => self.uniform.fog[3] = 0.
            },
            None => self.uniform.fog[3] = 0.
        }
        self.buffer.write(graphics, self.uniform);
    }

    pub fn render(&mut self, renderer: &mut Renderer, camera: &Camera, lighting: &Lighting){
        renderer.start_post();
        self.shader.bind(renderer);
        self.bind(renderer);
        camera.bind(renderer);
        lighting.bind(renderer);
        renderer.stop_post();
    }
}